use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal;
use pulldown_cmark::{html, Options, Parser};
use ratatui::prelude::*;
use ratatui::widgets::*;
use sigil_core::ChatStreamEvent;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::helpers::load_config;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ChatLine {
    role: String,
    text: String,
    style: Style,
}

// Lazy-loaded syntax and theme sets for markdown code highlighting.
struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
    highlighter: HighlightLines,
}

impl SyntaxHighlighter {
    fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();
        let highlighter = HighlightLines::new(&syntax_set, &theme);
        Self {
            syntax_set,
            theme,
            highlighter,
        }
    }

    fn highlight_code(&self, code: &str, lang: Option<&str>) -> Vec<(String, Style)> {
        let syntax = lang
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut result = Vec::new();
        for (line, style) in self.highlighter.highlight_line(code, syntax) {
            let style = Style::from(style);
            result.push((line.to_string(), style));
        }
        result
    }
}

#[allow(dead_code)]
struct AppState {
    messages: Vec<ChatLine>,
    input: String,
    status_agent: String,
    status_model: String,
    status_tokens: u32,
    status_context_pct: f64, // context usage percentage
    status_cost: f64,
    status_text: String,
    scroll_offset: u16,
    ws_tx: Option<tungstenite::WebSocket<std::net::TcpStream>>,
    should_quit: bool,
    agent_id: Option<String>,
    project: Option<String>,
    slash_mode: bool,
    slash_input: String,
    syntax_highlighter: SyntaxHighlighter,
    activity_spinner: u8, // for animated spinner
}

impl AppState {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            status_agent: "Rei".into(),
            status_model: String::new(),
            status_tokens: 0,
            status_context_pct: 0.0,
            status_cost: 0.0,
            status_text: String::new(),
            scroll_offset: 0,
            ws_tx: None,
            should_quit: false,
            agent_id: None,
            project: None,
            slash_mode: false,
            slash_input: String::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            activity_spinner: 0,
        }
    }

    fn push_user(&mut self, text: &str) {
        self.messages.push(ChatLine {
            role: "You".into(),
            text: text.to_string(),
            style: Style::default().fg(Color::Cyan),
        });
    }

    fn push_assistant_start(&mut self) {
        self.messages.push(ChatLine {
            role: "Rei".into(),
            text: String::new(),
            style: Style::default().fg(Color::Green),
        });
    }

    fn append_assistant_text(&mut self, delta: &str) {
        if let Some(last) = self.messages.last_mut()
            && last.role == "Rei"
        {
            last.text.push_str(delta);
            return;
        }
        // No active assistant line — create one.
        self.push_assistant_start();
        if let Some(last) = self.messages.last_mut() {
            last.text.push_str(delta);
        }
    }

    fn push_system(&mut self, text: String, style: Style) {
        self.messages.push(ChatLine {
            role: String::new(),
            text,
            style,
        });
    }

    /// Update an existing tool line (matched by tool_use_id prefix) in place.
    fn update_tool_line(&mut self, tool_name: &str, replacement: String, style: Style) {
        // Walk backwards to find the matching tool start line.
        for line in self.messages.iter_mut().rev() {
            if line.text.contains(&format!("\u{2699} {tool_name}")) {
                line.text = replacement;
                line.style = style;
                return;
            }
        }
        // Fallback: just append.
        self.push_system(replacement, style);
    }
}

// ---------------------------------------------------------------------------
// WebSocket background thread
// ---------------------------------------------------------------------------

enum WsCommand {
    Send(String),
    Quit,
}

fn spawn_ws_thread(
    url: String,
    cmd_rx: mpsc::Receiver<WsCommand>,
    event_tx: mpsc::Sender<ChatStreamEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        use tungstenite::Message;

        let mut ws = match tungstenite::connect(&url) {
            Ok((ws, _)) => ws,
            Err(e) => {
                let _ = event_tx.send(ChatStreamEvent::Error {
                    message: format!("WebSocket connect to {url} failed: {e}"),
                    recoverable: false,
                });
                return;
            }
        };

        // Make the underlying TCP stream non-blocking so we can interleave
        // send commands with incoming reads.
        if let tungstenite::stream::MaybeTlsStream::Plain(tcp) = ws.get_ref() {
            tcp.set_nonblocking(true).ok();
        }

        loop {
            // Check for outbound commands (non-blocking).
            match cmd_rx.try_recv() {
                Ok(WsCommand::Send(text)) => {
                    // Switch to blocking briefly for the send.
                    if let tungstenite::stream::MaybeTlsStream::Plain(tcp) = ws.get_ref() {
                        tcp.set_nonblocking(false).ok();
                    }
                    if ws.send(Message::Text(text.into())).is_err() {
                        break;
                    }
                    if let tungstenite::stream::MaybeTlsStream::Plain(tcp) = ws.get_ref() {
                        tcp.set_nonblocking(true).ok();
                    }
                }
                Ok(WsCommand::Quit) => {
                    if let tungstenite::stream::MaybeTlsStream::Plain(tcp) = ws.get_ref() {
                        tcp.set_nonblocking(false).ok();
                    }
                    let _ = ws.close(None);
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => break,
                Err(mpsc::TryRecvError::Empty) => {}
            }

            // Try to read an incoming message (non-blocking).
            match ws.read() {
                Ok(Message::Text(text)) => {
                    if let Ok(evt) = serde_json::from_str::<ChatStreamEvent>(&text)
                        && event_tx.send(evt).is_err()
                    {
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(tungstenite::Error::Io(ref e)) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Nothing available — sleep briefly so we don't spin.
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
                _ => {} // Ping/Pong/Binary — ignore
            }
        }
    })
}

// ---------------------------------------------------------------------------
// TUI rendering
// ---------------------------------------------------------------------------

fn draw(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // messages
            Constraint::Length(1), // status bar
            Constraint::Length(3), // input
        ])
        .split(frame.area());

    draw_messages(frame, chunks[0], state);
    draw_status(frame, chunks[1], state);
    draw_input(frame, chunks[2], state);
}

fn draw_messages(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut all_lines: Vec<Line> = Vec::new();

    for msg in &state.messages {
        // Role prefix
        let prefix = if msg.role.is_empty() {
            "  ".to_string()
        } else {
            format!("{}: ", msg.role)
        };

        // Render message text as markdown
        let rendered = render_markdown(&msg.text, &state.syntax_highlighter);
        
        // Add prefix to first line only
        if let Some(first) = rendered.first_mut() {
            first.spans.insert(0, Span::styled(prefix, msg.style));
        }

        all_lines.extend(rendered);
        all_lines.push(Line::from("")); // blank line between messages
    }

    // Auto-scroll: show the tail.
    let visible = area.height.saturating_sub(2) as usize; // block borders
    let total_lines = all_lines.len();
    let scroll = if total_lines > visible {
        (total_lines - visible) as u16
    } else {
        0
    };

    let paragraph = Paragraph::new(all_lines)
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let tokens_fmt = format_number(state.status_tokens);
    let cost_fmt = format!("${:.2}", state.status_cost);
    let ctx_fmt = format!("{:.0}%", state.status_context_pct);

    let model_display = if state.status_model.is_empty() {
        String::new()
    } else {
        format!(" | {}", state.status_model)
    };

    let extra = if state.status_text.is_empty() {
        String::new()
    } else {
        format!(" | {}", state.status_text)
    };

    // Animated spinner frames
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = if state.status_text.is_empty() {
        ""
    } else {
        spinner_frames[state.activity_spinner as usize]
    };

    let text = format!(
        "{} {}{} | {} tokens | {} ctx | {}{}",
        spinner, state.status_agent, model_display, tokens_fmt, ctx_fmt, cost_fmt, extra,
    );

    let bar = Paragraph::new(text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

fn draw_input(frame: &mut Frame, area: Rect, state: &AppState) {
    let display = format!("> {}", state.input);
    let input = Paragraph::new(display)
        .block(Block::default().borders(Borders::ALL).title(" Input "))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, area);

    // Place cursor after the input text.
    let cursor_x = area.x + 1 + 2 + state.input.len() as u16; // border + "> " prefix
    let cursor_y = area.y + 1; // inside border
    frame.set_cursor_position((
        cursor_x.min(area.x + area.width.saturating_sub(2)),
        cursor_y,
    ));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn textwrap(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![s.to_string()];
    }
    let mut result = Vec::new();
    for line in s.split('\n') {
        if line.is_empty() {
            result.push(String::new());
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        for chunk in chars.chunks(width) {
            result.push(chunk.iter().collect());
        }
    }
    result
}

fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ---------------------------------------------------------------------------
// Event processing
// ---------------------------------------------------------------------------

fn process_ws_event(state: &mut AppState, evt: ChatStreamEvent) {
    match evt {
        ChatStreamEvent::TextDelta { text } => {
            state.append_assistant_text(&text);
        }
        ChatStreamEvent::ToolStart { tool_name, .. } => {
            state.push_system(
                format!("  \u{2699} {tool_name}..."),
                Style::default().fg(Color::DarkGray),
            );
        }
        ChatStreamEvent::ToolComplete {
            tool_name,
            duration_ms,
            success,
            ..
        } => {
            let icon = if success { "\u{2713}" } else { "\u{2717}" };
            let style = if success {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Red)
            };
            state.update_tool_line(
                &tool_name,
                format!("  {icon} {tool_name} ({duration_ms}ms)"),
                style,
            );
        }
        ChatStreamEvent::TurnStart { turn, model } => {
            state.push_system(
                format!("  \u{21bb} Turn {turn}..."),
                Style::default().fg(Color::DarkGray),
            );
            state.status_model = model;
            // Start a new assistant message for this turn's text.
            state.push_assistant_start();
        }
        ChatStreamEvent::TurnComplete {
            prompt_tokens,
            completion_tokens,
            ..
        } => {
            state.status_tokens = prompt_tokens + completion_tokens;
        }
        ChatStreamEvent::Complete {
            total_prompt_tokens,
            total_completion_tokens,
            cost_usd,
            ..
        } => {
            state.status_tokens = total_prompt_tokens + total_completion_tokens;
            state.status_cost = cost_usd;
            state.status_text.clear();
        }
        ChatStreamEvent::Status { message } => {
            state.status_text = message;
        }
        ChatStreamEvent::Error { message, .. } => {
            state.push_system(
                format!("  ERROR: {message}"),
                Style::default().fg(Color::Red),
            );
        }
        ChatStreamEvent::DelegateStart {
            worker_name,
            task_subject,
        } => {
            state.push_system(
                format!("  \u{2192} Delegating to {worker_name}: {task_subject}"),
                Style::default().fg(Color::Magenta),
            );
        }
        ChatStreamEvent::DelegateComplete {
            worker_name,
            outcome,
        } => {
            state.push_system(
                format!("  \u{2190} {worker_name}: {outcome}"),
                Style::default().fg(Color::Magenta),
            );
        }
        ChatStreamEvent::MemoryActivity {
            action,
            key,
            preview,
        } => {
            let icon = if action == "recalled" {
                "\u{1f4d6}"
            } else {
                "\u{1f4be}"
            };
            state.push_system(
                format!("  {icon} {action} [{key}]: {}", truncate(&preview, 60)),
                Style::default().fg(Color::DarkGray),
            );
        }
        ChatStreamEvent::Compacted {
            original_messages,
            remaining_messages,
            ..
        } => {
            state.push_system(
                format!(
                    "  \u{267b} Compacted {original_messages} \u{2192} {remaining_messages} messages"
                ),
                Style::default().fg(Color::DarkGray),
            );
        }
        ChatStreamEvent::ToolProgress { .. } => {
            // Silently ignore progress deltas for now.
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

// ---------------------------------------------------------------------------
// Slash commands
// ---------------------------------------------------------------------------

fn handle_slash_command(input: &str, state: &mut AppState, cmd_tx: &mpsc::Sender<WsCommand>) -> bool {
    let parts: Vec<&str> = input.trim_start_matches('/').splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = parts.get(1).copied().unwrap_or("");

    match cmd.as_str() {
        "exit" | "quit" => {
            state.should_quit = true;
            return true;
        }
        "new" => {
            state.messages.clear();
            state.status_text.clear();
            return true;
        }
        "model" => {
            if !args.is_empty() {
                let msg = serde_json::json!({
                    "message": format!("/model {}", args),
                });
                let _ = cmd_tx.send(WsCommand::Send(msg.to_string()));
            }
            return true;
        }
        "compress" => {
            let msg = serde_json::json!({
                "message": "/compress",
            });
            let _ = cmd_tx.send(WsCommand::Send(msg.to_string()));
            return true;
        }
        "status" => {
            let msg = serde_json::json!({
                "message": "/status",
            });
            let _ = cmd_tx.send(WsCommand::Send(msg.to_string()));
            return true;
        }
        "skills" => {
            let msg = serde_json::json!({
                "message": "/skills",
            });
            let _ = cmd_tx.send(WsCommand::Send(msg.to_string()));
            return true;
        }
        "help" => {
            state.push_system(
                "Slash commands: /new, /model <name>, /compress, /status, /skills, /exit".to_string(),
                Style::default().fg(Color::Cyan),
            );
            return true;
        }
        _ => {
            state.push_system(
                format!("Unknown command: /{}", cmd),
                Style::default().fg(Color::Red),
            );
            return true;
        }
    }
}

// ---------------------------------------------------------------------------
// Markdown rendering
// ---------------------------------------------------------------------------

fn render_markdown(text: &str, highlighter: &SyntaxHighlighter) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut current_style = Style::default();
    let mut in_code_block = false;
    let mut code_content = String::new();
    let mut code_lang: Option<String> = None;

    let opts = Options::empty();
    let parser = Parser::new_ext(text, opts);

    for event in parser {
        match event {
            pulldown_cmark::Event::Start(tag) => match tag {
                pulldown_cmark::Tag::Paragraph => {}
                pulldown_cmark::Tag::Heading(level, _, _) => {
                    let prefix = match level {
                        pulldown_cmark::HeadingLevel::H1 => "# ",
                        pulldown_cmark::HeadingLevel::H2 => "## ",
                        pulldown_cmark::HeadingLevel::H3 => "### ",
                        pulldown_cmark::HeadingLevel::H4 => "#### ",
                        pulldown_cmark::HeadingLevel::H5 => "##### ",
                        pulldown_cmark::HeadingLevel::H6 => "###### ",
                    };
                    lines.push(Line::from(Span::styled(
                        prefix.to_string(),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )));
                    current_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
                }
                pulldown_cmark::Tag::BlockQuote => {
                    lines.push(Line::from(Span::styled(
                        "> ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    )));
                    current_style = Style::default().fg(Color::DarkGray);
                }
                pulldown_cmark::Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_content.clear();
                    if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                        code_lang = Some(lang.to_string());
                    } else {
                        code_lang = None;
                    }
                }
                pulldown_cmark::Tag::List(_) => {}
                pulldown_cmark::Tag::Item => {
                    lines.push(Line::from(Span::styled(
                        "  • ".to_string(),
                        Style::default().fg(Color::Cyan),
                    )));
                    current_style = Style::default();
                }
                pulldown_cmark::Tag::Emphasis => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                pulldown_cmark::Tag::Strong => {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                }
                pulldown_cmark::Tag::Link(_link_type, _dest, _title) => {
                    current_style = current_style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED);
                }
                pulldown_cmark::Tag::Image(_link_type, _dest, _title) => {
                    current_style = current_style.fg(Color::Magenta);
                }
                pulldown_cmark::Tag::Table(_) => {}
                pulldown_cmark::Tag::TableHead => {}
                pulldown_cmark::Tag::TableRow => {}
                pulldown_cmark::Tag::TableCell => {}
            },
            pulldown_cmark::Event::End(tag) => match tag {
                pulldown_cmark::Tag::Paragraph => {
                    lines.push(Line::from(""));
                }
                pulldown_cmark::Tag::Heading(_, _, _) => {
                    lines.push(Line::from(""));
                    current_style = Style::default();
                }
                pulldown_cmark::Tag::BlockQuote => {
                    lines.push(Line::from(""));
                }
                pulldown_cmark::Tag::CodeBlock(_) => {
                    in_code_block = false;
                    if !code_content.is_empty() {
                        // Render code block with syntax highlighting.
                        for (line, style) in highlighter.highlight_code(&code_content, code_lang.as_deref()) {
                            lines.push(Line::from(Span::styled(line, style)));
                        }
                        lines.push(Line::from(""));
                    }
                    code_content.clear();
                    code_lang = None;
                    current_style = Style::default();
                }
                pulldown_cmark::Tag::List(_) => {}
                pulldown_cmark::Tag::Item => {
                    current_style = Style::default();
                }
                pulldown_cmark::Tag::Emphasis => {
                    current_style = current_style.remove_modifier(Modifier::ITALIC);
                }
                pulldown_cmark::Tag::Strong => {
                    current_style = current_style.remove_modifier(Modifier::BOLD);
                }
                pulldown_cmark::Tag::Link(_, _, _) => {
                    current_style = current_style.remove_modifier(Modifier::UNDERLINED);
                }
                pulldown_cmark::Tag::Image(_, _, _) => {
                    current_style = Style::default();
                }
                pulldown_cmark::Tag::Table(_) => {}
                pulldown_cmark::Tag::TableHead => {}
                pulldown_cmark::Tag::TableRow => {}
                pulldown_cmark::Tag::TableCell => {}
            },
            pulldown_cmark::Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else {
                    for ch in text.chars() {
                        if ch == '\n' {
                            lines.push(Line::from(""));
                        } else {
                            // Accumulate characters into the current line.
                            if lines.is_empty() {
                                lines.push(Line::from(Span::styled(ch.to_string(), current_style)));
                            } else {
                                let last = lines.last_mut().unwrap();
                                last.spans.last_mut().unwrap().content.push(ch);
                            }
                        }
                    }
                }
            }
            pulldown_cmark::Event::Code(text) => {
                let span = Span::styled(
                    text.to_string(),
                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
                );
                if lines.is_empty() {
                    lines.push(Line::from(span));
                } else {
                    lines.last_mut().unwrap().spans.push(span);
                }
            }
            pulldown_cmark::Event::Html(_html) => {}
            pulldown_cmark::Event::SoftBreak => {
                lines.push(Line::from(""));
            }
            pulldown_cmark::Event::HardBreak => {
                lines.push(Line::from(""));
            }
            pulldown_cmark::Event::Rule => {
                for _ in 0..2 {
                    lines.push(Line::from(Span::styled(
                        "─".repeat(80),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            pulldown_cmark::Event::FootnoteReference(_name) => {}
            pulldown_cmark::Event::TaskListMarker(true) => {
                if let Some(last) = lines.last_mut() {
                    let check = Span::styled("[x] ", Style::default().fg(Color::Green));
                    last.spans.push(check);
                }
            }
            pulldown_cmark::Event::TaskListMarker(false) => {
                if let Some(last) = lines.last_mut() {
                    let check = Span::styled("[ ] ", Style::default().fg(Color::DarkGray));
                    last.spans.push(check);
                }
            }
        }
    }

    // Clean up empty trailing lines.
    while lines.last().map(|l| l.spans.is_empty() && l.is_empty()) == Some(true) {
        lines.pop();
    }

    lines
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Terminal chat interface using ratatui + crossterm + tungstenite.
pub(crate) async fn cmd_chat_tui(
    config_path: &Option<PathBuf>,
    agent_name: Option<&str>,
    project: Option<&str>,
) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let data_dir = config.data_dir();

    // Resolve the persistent agent to chat with.
    let registry = sigil_orchestrator::agent_registry::AgentRegistry::open(&data_dir)?;
    let agent = if let Some(name) = agent_name {
        registry.get_active_by_name(name).await?
    } else {
        registry.default_for_project(project).await?
    };

    let (agent_display, agent_id) = match &agent {
        Some(a) => (
            a.display_name.as_deref().unwrap_or(&a.name).to_string(),
            Some(a.id.clone()),
        ),
        None => (
            config
                .leader_agent()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Rei".into()),
            None,
        ),
    };

    // Extract port from the bind address (e.g. "0.0.0.0:8400" -> 8400).
    let bind = &config.web.bind;
    let port = bind
        .rsplit(':')
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8400);
    let ws_url = format!("ws://127.0.0.1:{port}/api/chat/stream");

    // Channels: ws events -> main thread, main thread -> ws commands.
    let (event_tx, event_rx) = mpsc::channel::<ChatStreamEvent>();
    let (cmd_tx, cmd_rx) = mpsc::channel::<WsCommand>();

    let ws_handle = spawn_ws_thread(ws_url, cmd_rx, event_tx);

    // Enter terminal raw mode.
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut term = ratatui::Terminal::new(backend)?;

    let mut state = AppState::new();
    state.status_agent = agent_display;
    // Store agent_id and project in state for inclusion in WebSocket messages.
    state.agent_id = agent_id;
    state.project = project.map(|s| s.to_string());

    // Main event loop.
    let result = run_loop(&mut term, &mut state, &event_rx, &cmd_tx);

    // Cleanup: always restore terminal.
    let _ = cmd_tx.send(WsCommand::Quit);
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    // Wait for ws thread to finish (with timeout).
    let _ = ws_handle.join();

    result
}

fn run_loop(
    term: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    event_rx: &mpsc::Receiver<ChatStreamEvent>,
    cmd_tx: &mpsc::Sender<WsCommand>,
) -> Result<()> {
    loop {
        term.draw(|f| draw(f, state))?;

        // Drain all pending WebSocket events.
        while let Ok(evt) = event_rx.try_recv() {
            process_ws_event(state, evt);
        }

        // Poll crossterm events with 50ms timeout for responsiveness.
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Esc => {
                    if state.slash_mode {
                        state.slash_mode = false;
                        state.slash_input.clear();
                    } else {
                        state.should_quit = true;
                    }
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.should_quit = true;
                }
                KeyCode::Enter => {
                    let input = if state.slash_mode {
                        format!("/{}", state.slash_input)
                    } else {
                        state.input.trim().to_string()
                    };
                    
                    if !input.is_empty() {
                        if state.slash_mode {
                            // Handle slash command
                            let _ = handle_slash_command(&input, state, cmd_tx);
                            state.slash_mode = false;
                            state.slash_input.clear();
                        } else if input.starts_with('/') {
                            // Enter slash mode with the command (without leading /)
                            state.slash_mode = true;
                            state.slash_input = input[1..].to_string();
                        } else {
                            // Regular message
                            state.push_user(&input);
                            state.input.clear();

                            let mut msg = serde_json::json!({
                                "message": input,
                            });
                            if let Some(ref aid) = state.agent_id {
                                msg["agent_id"] = serde_json::json!(aid);
                            }
                            if let Some(ref p) = state.project {
                                msg["project"] = serde_json::json!(p);
                            }
                            let _ = cmd_tx.send(WsCommand::Send(msg.to_string()));
                        }
                    } else if state.slash_mode {
                        state.slash_mode = false;
                        state.slash_input.clear();
                    }
                }
                KeyCode::Backspace => {
                    if state.slash_mode {
                        state.slash_input.pop();
                    } else {
                        state.input.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if state.slash_mode {
                        state.slash_input.push(c);
                    } else {
                        state.input.push(c);
                    }
                }
                _ => {}
            }
        }

        if state.should_quit {
            break;
        }
    }

    Ok(())
}
