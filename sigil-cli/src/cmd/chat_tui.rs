use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal;
use ratatui::prelude::*;
use ratatui::widgets::*;
use sigil_core::ChatStreamEvent;

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

#[allow(dead_code)]
struct AppState {
    messages: Vec<ChatLine>,
    input: String,
    status_agent: String,
    status_model: String,
    status_tokens: u32,
    status_cost: f64,
    status_text: String,
    scroll_offset: u16,
    ws_tx: Option<tungstenite::WebSocket<std::net::TcpStream>>,
    should_quit: bool,
    agent_id: Option<String>,
    project: Option<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            status_agent: "Rei".into(),
            status_model: String::new(),
            status_tokens: 0,
            status_cost: 0.0,
            status_text: String::new(),
            scroll_offset: 0,
            ws_tx: None,
            should_quit: false,
            agent_id: None,
            project: None,
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
    let mut lines: Vec<Line> = Vec::new();

    for msg in &state.messages {
        let prefix = if msg.role.is_empty() {
            "  ".to_string()
        } else {
            format!("{}: ", msg.role)
        };

        // Split text into wrapped display lines.
        let full = format!("{prefix}{}", msg.text);
        let width = area.width.saturating_sub(2) as usize; // account for block border
        if width == 0 {
            lines.push(Line::from(Span::styled(full, msg.style)));
            continue;
        }

        for chunk in textwrap(full.as_str(), width) {
            lines.push(Line::from(Span::styled(chunk, msg.style)));
        }
    }

    // Auto-scroll: show the tail.
    let visible = area.height.saturating_sub(2) as usize; // block borders
    let scroll = if lines.len() > visible {
        (lines.len() - visible) as u16
    } else {
        0
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let tokens_fmt = format_number(state.status_tokens);
    let cost_fmt = format!("${:.2}", state.status_cost);

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

    let text = format!(
        "\u{25cf} {}{} | {} tokens | {}{}",
        state.status_agent, model_display, tokens_fmt, cost_fmt, extra,
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
                    state.should_quit = true;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.should_quit = true;
                }
                KeyCode::Enter => {
                    let text = state.input.trim().to_string();
                    if !text.is_empty() {
                        state.push_user(&text);
                        state.input.clear();

                        let mut msg = serde_json::json!({
                            "message": text,
                        });
                        if let Some(ref aid) = state.agent_id {
                            msg["agent_id"] = serde_json::json!(aid);
                        }
                        if let Some(ref p) = state.project {
                            msg["project"] = serde_json::json!(p);
                        }
                        let _ = cmd_tx.send(WsCommand::Send(msg.to_string()));
                    }
                }
                KeyCode::Backspace => {
                    state.input.pop();
                }
                KeyCode::Char(c) => {
                    state.input.push(c);
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
