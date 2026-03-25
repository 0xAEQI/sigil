//! Unified Chat Engine — source-agnostic chat processing for Telegram, web, and future channels.
//!
//! Both Telegram and web chat are thin clients that delegate to this engine.
//! The engine handles: intent detection, conversation history, agent routing,
//! council invocation, task creation, and completion tracking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use anyhow::Result;

use sigil_core::traits::{Memory, MemoryQuery, MemoryScope};

use crate::agent_router::AgentRouter;
use crate::conversation_store::{ChannelInfo, ConversationMessage, ConversationStore};
use crate::registry::ProjectRegistry;

// ── Types ──

/// Source of a chat message.
#[derive(Debug, Clone)]
pub enum ChatSource {
    Telegram { message_id: i64 },
    Web { session_id: String },
    Discord,
    Slack,
}

impl ChatSource {
    pub fn channel_type(&self) -> &str {
        match self {
            ChatSource::Telegram { .. } => "telegram",
            ChatSource::Web { .. } => "web",
            ChatSource::Discord => "discord",
            ChatSource::Slack => "slack",
        }
    }

    pub fn message_id(&self) -> i64 {
        match self {
            ChatSource::Telegram { message_id } => *message_id,
            _ => 0,
        }
    }
}

/// Incoming chat message.
pub struct ChatMessage {
    pub message: String,
    pub chat_id: i64,
    pub sender: String,
    pub source: ChatSource,
    pub project_hint: Option<String>,
}

/// Response from the chat engine (quick path).
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub ok: bool,
    pub context: String,
    pub action: Option<String>,
    pub task: Option<serde_json::Value>,
    pub projects: Option<Vec<serde_json::Value>>,
    pub cost: Option<serde_json::Value>,
    pub workers: Option<u32>,
}

impl ChatResponse {
    pub fn error(msg: &str) -> Self {
        Self {
            ok: false,
            context: msg.to_string(),
            action: None,
            task: None,
            projects: None,
            cost: None,
            workers: None,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let mut v = serde_json::json!({
            "ok": self.ok,
            "context": self.context,
        });
        if let Some(ref action) = self.action {
            v["action"] = serde_json::json!(action);
        }
        if let Some(ref task) = self.task {
            v["task"] = task.clone();
        }
        if let Some(ref projects) = self.projects {
            v["projects"] = serde_json::json!(projects);
        }
        if let Some(ref cost) = self.cost {
            v["cost"] = cost.clone();
        }
        if let Some(workers) = self.workers {
            v["workers"] = serde_json::json!(workers);
        }
        v
    }
}

/// Handle returned when a full (async) chat task is created.
#[derive(Debug, Clone)]
pub struct ChatTaskHandle {
    pub task_id: String,
    pub chat_id: i64,
    pub project: String,
}

/// A pending task that's being processed asynchronously.
pub struct PendingChatTask {
    pub chat_id: i64,
    pub message_id: i64,
    pub source: ChatSource,
    pub created_at: std::time::Instant,
    pub phase1_reaction: Option<String>,
    pub sent_slow_notice: bool,
}

/// Result of a completed chat task.
#[derive(Debug, Clone)]
pub struct ChatCompletion {
    pub task_id: String,
    pub chat_id: i64,
    pub message_id: i64,
    pub source: ChatSource,
    pub status: CompletionStatus,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum CompletionStatus {
    Done,
    Blocked,
    Cancelled,
    TimedOut,
}

// ── Engine ──

/// The unified chat engine.
pub struct ChatEngine {
    pub conversations: Arc<ConversationStore>,
    pub registry: Arc<ProjectRegistry>,
    pub agent_router: Arc<Mutex<AgentRouter>>,
    pub council_advisors: Arc<Vec<sigil_core::config::PeerAgentConfig>>,
    pub leader_name: String,
    pub pending_tasks: Arc<Mutex<HashMap<String, PendingChatTask>>>,
    pub task_notify: Arc<tokio::sync::Notify>,
    /// Per-project memory stores for knowledge-aware chat.
    pub memory_stores: HashMap<String, Arc<dyn Memory>>,
}

impl ChatEngine {
    /// Handle a chat message (quick path): intent detection + status queries.
    /// Returns immediately. For messages that don't match an intent, returns None
    /// to signal the caller should use `handle_message_full` instead.
    pub async fn handle_message(&self, msg: &ChatMessage) -> Option<ChatResponse> {
        if msg.message.is_empty() {
            return Some(ChatResponse::error("message is required"));
        }

        let source_tag = msg.source.channel_type();

        // Register channel.
        let _ = self
            .conversations
            .ensure_channel(msg.chat_id, source_tag, &msg.sender)
            .await;

        let msg_lower = msg.message.to_lowercase();

        // Intent: create task.
        if msg_lower.starts_with("create task")
            || msg_lower.starts_with("new task")
            || msg_lower.starts_with("add task")
            || msg_lower.contains("create a task")
            || msg_lower.contains("add a task")
        {
            return Some(self.handle_create_task(msg).await);
        }

        // Intent: close task.
        if msg_lower.starts_with("close task")
            || msg_lower.starts_with("done with")
            || msg_lower.contains("close task")
            || msg_lower.contains("mark done")
        {
            return Some(self.handle_close_task(msg).await);
        }

        // Intent: blackboard post.
        if msg_lower.starts_with("note:")
            || msg_lower.starts_with("remember:")
            || msg_lower.starts_with("blackboard:")
        {
            return Some(self.handle_blackboard_post(msg).await);
        }

        // No intent matched — caller should use handle_message_full.
        None
    }

    /// Handle a chat message (full path): conversation context + agent routing + council + task creation.
    /// Returns a task handle for async completion tracking.
    pub async fn handle_message_full(
        &self,
        msg: &ChatMessage,
        phase1_reaction: Option<String>,
    ) -> Result<ChatTaskHandle> {
        let source_tag = msg.source.channel_type();

        // Register channel.
        let _ = self
            .conversations
            .ensure_channel(msg.chat_id, source_tag, &msg.sender)
            .await;

        // Evict stale conversations.
        let _ = self.conversations.evict_older_than(2).await;

        // Fetch recent messages for context.
        let recent = self
            .conversations
            .recent(msg.chat_id, 20)
            .await
            .unwrap_or_default();

        // Build conversation context for task description.
        let ctx = self
            .conversations
            .context_string(msg.chat_id, 20)
            .await
            .unwrap_or_default();

        // Build compact context for advisor tasks.
        let conv_context_for_advisors = if recent.is_empty() {
            String::new()
        } else {
            let mut s = String::from("Recent conversation:\n");
            for msg_item in recent
                .iter()
                .rev()
                .take(6)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                let truncated = if msg_item.content.len() > 200 {
                    let mut end = 200;
                    while !msg_item.content.is_char_boundary(end) {
                        end -= 1;
                    }
                    &msg_item.content[..end]
                } else {
                    msg_item.content.as_str()
                };
                s.push_str(&format!("  {}: {}\n", msg_item.role, truncated));
            }
            s
        };

        // Record user message.
        let _ = self
            .conversations
            .record_with_source(msg.chat_id, "User", &msg.message, Some(source_tag))
            .await;

        // Build task description with conversation context.
        let routing = format!(
            "[source: {} | chat_id: {} | reply: auto-delivered by daemon]",
            source_tag, msg.chat_id
        );
        let response_protocol = "**RESPONSE PROTOCOL**: Write your reply directly — in character, in voice. Your output text IS the reply. The daemon delivers it automatically. Do NOT call any tools to send the reply. Do NOT write meta-commentary like \"I've sent your reply\" or \"Done.\".";
        let mut description = if ctx.is_empty() {
            format!("{}\n\n---\n{}\n{}", msg.message, routing, response_protocol)
        } else {
            format!(
                "{}\n## Current Message\n\n{}\n\n---\n{}\n{}",
                ctx, msg.message, routing, response_protocol
            )
        };

        // Inject Phase 1 reaction if available.
        if let Some(ref reaction) = phase1_reaction {
            description = format!(
                "{}\n\n---\n## Your Immediate Reaction (already sent)\n\n\
                 You already reacted with this stage direction:\n\
                 {}\n\n\
                 Continue from this energy. Your full reply should feel like the natural \
                 next beat after this reaction — same emotional tone, same intensity. \
                 Don't repeat or reference the reaction itself, just carry its momentum.\n",
                description, reaction
            );
        }

        // === Council: classify and gather advisor input ===
        let is_council = msg.message.starts_with("/council");
        let clean_text = if is_council {
            msg.message
                .strip_prefix("/council")
                .unwrap_or(&msg.message)
                .trim()
                .to_string()
        } else {
            msg.message.clone()
        };

        let advisors_to_invoke = self
            .classify_advisors(&clean_text, is_council, msg.chat_id)
            .await;

        // Gather council input (parallel advisor invocation).
        let council_input = if !advisors_to_invoke.is_empty() {
            self.gather_council_input(
                &advisors_to_invoke,
                &clean_text,
                &conv_context_for_advisors,
                msg.chat_id,
                source_tag,
            )
            .await
        } else {
            Vec::new()
        };

        // Append council input to description.
        if !council_input.is_empty() {
            description.push_str("\n\n## Council Input\n\n");
            for (name, text) in &council_input {
                description.push_str(&format!("### {} advises:\n{}\n\n", name, text));
            }
            description.push_str(
                "Synthesize the council's input into your response. Attribute key insights where relevant.\n",
            );
        }

        // Create the task.
        let subject = format!("[{}] {} ({})", source_tag, msg.sender, msg.chat_id);
        let task = self
            .registry
            .assign(&self.leader_name, &subject, &description)
            .await?;
        let task_id = task.id.0.clone();

        // Register pending task for completion tracking.
        self.pending_tasks.lock().await.insert(
            task_id.clone(),
            PendingChatTask {
                chat_id: msg.chat_id,
                message_id: msg.source.message_id(),
                source: msg.source.clone(),
                created_at: std::time::Instant::now(),
                phase1_reaction,
                sent_slow_notice: false,
            },
        );

        Ok(ChatTaskHandle {
            task_id,
            chat_id: msg.chat_id,
            project: self.leader_name.clone(),
        })
    }

    /// Check pending tasks for completions. Returns completed tasks and removes them from pending.
    pub async fn check_completions(&self) -> Vec<ChatCompletion> {
        let mut completions = Vec::new();
        let mut map = self.pending_tasks.lock().await;
        let task_ids: Vec<String> = map.keys().cloned().collect();

        for qid in task_ids {
            let status = {
                if let Some(rig) = self.registry.get_project(&self.leader_name).await {
                    let store = rig.tasks.lock().await;
                    store.get(&qid).map(|b| (b.status, b.closed_reason.clone()))
                } else {
                    None
                }
            };

            let Some(pq) = map.get_mut(&qid) else {
                continue;
            };
            let elapsed = pq.created_at.elapsed();

            match status {
                Some((sigil_tasks::TaskStatus::Done, reason)) => {
                    let reply_text = reason
                        .filter(|r| !r.trim().is_empty())
                        .unwrap_or_else(|| "Done.".to_string());
                    let _ = self
                        .conversations
                        .record_with_source(
                            pq.chat_id,
                            &self.leader_name,
                            &reply_text,
                            Some(pq.source.channel_type()),
                        )
                        .await;
                    completions.push(ChatCompletion {
                        task_id: qid.clone(),
                        chat_id: pq.chat_id,
                        message_id: pq.message_id,
                        source: pq.source.clone(),
                        status: CompletionStatus::Done,
                        text: reply_text,
                    });
                    map.remove(&qid);
                }
                Some((sigil_tasks::TaskStatus::Blocked, reason)) => {
                    let text = reason.unwrap_or_else(|| "Blocked — needs input.".to_string());
                    completions.push(ChatCompletion {
                        task_id: qid.clone(),
                        chat_id: pq.chat_id,
                        message_id: pq.message_id,
                        source: pq.source.clone(),
                        status: CompletionStatus::Blocked,
                        text: format!("Blocked: {}", text),
                    });
                    map.remove(&qid);
                }
                Some((sigil_tasks::TaskStatus::Cancelled, reason)) => {
                    let text = reason.unwrap_or_else(|| "Task cancelled.".to_string());
                    completions.push(ChatCompletion {
                        task_id: qid.clone(),
                        chat_id: pq.chat_id,
                        message_id: pq.message_id,
                        source: pq.source.clone(),
                        status: CompletionStatus::Cancelled,
                        text: format!("Failed: {}", text),
                    });
                    map.remove(&qid);
                }
                _ => {
                    // Still pending/in-progress — check for hard timeout.
                    if elapsed > std::time::Duration::from_secs(1800) {
                        warn!(task = %qid, "chat task hard-timed out after 30min");
                        completions.push(ChatCompletion {
                            task_id: qid.clone(),
                            chat_id: pq.chat_id,
                            message_id: pq.message_id,
                            source: pq.source.clone(),
                            status: CompletionStatus::TimedOut,
                            text:
                                "Sorry, this one took too long. Try again or simplify the request."
                                    .to_string(),
                        });
                        map.remove(&qid);
                    }
                }
            }
        }

        completions
    }

    /// Get pending tasks that need a slow-progress notice (elapsed > 2min).
    pub async fn get_slow_tasks(&self) -> Vec<(String, i64, i64, ChatSource)> {
        let mut slow = Vec::new();
        let mut map = self.pending_tasks.lock().await;
        for (qid, pq) in map.iter_mut() {
            let elapsed = pq.created_at.elapsed();
            if elapsed > std::time::Duration::from_secs(120) && !pq.sent_slow_notice {
                pq.sent_slow_notice = true;
                slow.push((qid.clone(), pq.chat_id, pq.message_id, pq.source.clone()));
            }
        }
        slow
    }

    /// Poll a specific task for completion.
    pub async fn poll_completion(&self, task_id: &str) -> Option<ChatCompletion> {
        let status = {
            if let Some(rig) = self.registry.get_project(&self.leader_name).await {
                let store = rig.tasks.lock().await;
                store
                    .get(task_id)
                    .map(|b| (b.status, b.closed_reason.clone()))
            } else {
                None
            }
        };

        let mut map = self.pending_tasks.lock().await;
        let pq = map.get(task_id)?;

        match status {
            Some((sigil_tasks::TaskStatus::Done, reason)) => {
                let text = reason
                    .filter(|r| !r.trim().is_empty())
                    .unwrap_or_else(|| "Done.".to_string());
                let completion = ChatCompletion {
                    task_id: task_id.to_string(),
                    chat_id: pq.chat_id,
                    message_id: pq.message_id,
                    source: pq.source.clone(),
                    status: CompletionStatus::Done,
                    text,
                };
                map.remove(task_id);
                Some(completion)
            }
            Some((sigil_tasks::TaskStatus::Blocked, reason)) => {
                let text = reason.unwrap_or_else(|| "Blocked.".to_string());
                let completion = ChatCompletion {
                    task_id: task_id.to_string(),
                    chat_id: pq.chat_id,
                    message_id: pq.message_id,
                    source: pq.source.clone(),
                    status: CompletionStatus::Blocked,
                    text,
                };
                map.remove(task_id);
                Some(completion)
            }
            Some((sigil_tasks::TaskStatus::Cancelled, reason)) => {
                let text = reason.unwrap_or_else(|| "Cancelled.".to_string());
                let completion = ChatCompletion {
                    task_id: task_id.to_string(),
                    chat_id: pq.chat_id,
                    message_id: pq.message_id,
                    source: pq.source.clone(),
                    status: CompletionStatus::Cancelled,
                    text,
                };
                map.remove(task_id);
                Some(completion)
            }
            _ => None,
        }
    }

    /// Get conversation history.
    pub async fn get_history(
        &self,
        chat_id: i64,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ConversationMessage>> {
        self.conversations
            .recent_with_offset(chat_id, limit, offset)
            .await
    }

    /// List all known channels.
    pub async fn list_channels(&self) -> Result<Vec<ChannelInfo>> {
        self.conversations.list_channels().await
    }

    /// Build a status response enriched with relevant memories.
    pub async fn status_response(
        &self,
        project_hint: Option<&str>,
        query: Option<&str>,
    ) -> ChatResponse {
        // Search memory for relevant context if we have a query.
        let memory_context = if let (Some(project), Some(q)) = (project_hint, query) {
            self.build_memory_context(project, q).await
        } else if let Some(q) = query {
            // Global query — search all projects.
            let mut all_ctx = Vec::new();
            for (name, mem) in &self.memory_stores {
                let mq = MemoryQuery::new(q, 3).with_scope(MemoryScope::Domain);
                if let Ok(results) = mem.search(&mq).await {
                    for entry in results {
                        all_ctx.push(format!("  • [{}] {}: {}", name, entry.key, entry.content));
                    }
                }
            }
            if all_ctx.is_empty() {
                None
            } else {
                Some(format!("Relevant knowledge:\n{}", all_ctx.join("\n")))
            }
        } else {
            None
        };

        let summaries = self.registry.list_project_summaries().await;
        let (spent, budget, remaining) = self.registry.cost_ledger.budget_status();
        let worker_count = self.registry.total_max_workers().await;

        let recent_audit = match &self.registry.audit_log {
            Some(audit) => audit.query_recent(5).unwrap_or_default(),
            None => Vec::new(),
        };

        let project_summaries: Vec<_> = if let Some(p) = project_hint {
            summaries.iter().filter(|s| s.name == p).collect()
        } else {
            summaries.iter().collect()
        };

        let mut context = String::new();

        if let Some(p) = project_hint {
            if let Some(s) = project_summaries.first() {
                context.push_str(&format!(
                    "{}: {} open tasks ({} pending, {} in progress, {} done), {} missions\n",
                    s.name,
                    s.open_tasks,
                    s.pending_tasks,
                    s.in_progress_tasks,
                    s.done_tasks,
                    s.active_missions
                ));
                if let Some(t) = &s.team {
                    context.push_str(&format!(
                        "Team: {} (lead), agents: {}\n",
                        t.leader,
                        t.agents.join(", ")
                    ));
                }
                if !s.departments.is_empty() {
                    context.push_str("Departments:\n");
                    for d in &s.departments {
                        context.push_str(&format!(
                            "  {} — lead: {}, agents: {}\n",
                            d.name,
                            d.lead.as_deref().unwrap_or("-"),
                            d.agents.join(", ")
                        ));
                    }
                }
            } else {
                context.push_str(&format!("Project '{}' not found.\n", p));
            }
        } else {
            for s in &project_summaries {
                context.push_str(&format!(
                    "{}: {} open/{} total tasks, {} missions\n",
                    s.name, s.open_tasks, s.total_tasks, s.active_missions
                ));
            }
        }

        context.push_str(&format!(
            "\nWorkers: {}, Cost: ${:.3}/${:.2}, Remaining: ${:.3}\n",
            worker_count, spent, budget, remaining
        ));

        if !recent_audit.is_empty() {
            context.push_str("\nRecent:\n");
            for e in &recent_audit {
                context.push_str(&format!(
                    "  [{}] {} — {}\n",
                    e.project,
                    e.decision_type,
                    e.reasoning.chars().take(80).collect::<String>()
                ));
            }
        }

        // Prepend memory context if available.
        if let Some(ref mem_ctx) = memory_context {
            context = format!("{}\n\n{}", mem_ctx, context);
        }

        ChatResponse {
            ok: true,
            context: context.trim().to_string(),
            action: None,
            task: None,
            projects: Some(
                project_summaries
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "name": s.name,
                            "open_tasks": s.open_tasks,
                            "total_tasks": s.total_tasks,
                            "active_missions": s.active_missions,
                        })
                    })
                    .collect(),
            ),
            cost: Some(serde_json::json!({
                "spent": spent,
                "budget": budget,
                "remaining": remaining,
            })),
            workers: Some(worker_count),
        }
    }

    /// Search memory for context relevant to a query in a specific project.
    pub async fn build_memory_context(&self, project: &str, query: &str) -> Option<String> {
        let mem = self.memory_stores.get(project)?;
        let mq = MemoryQuery::new(query, 5).with_scope(MemoryScope::Domain);
        let results = mem.search(&mq).await.ok()?;
        if results.is_empty() {
            return None;
        }
        let mut ctx = String::from("Relevant knowledge:\n");
        for entry in &results {
            ctx.push_str(&format!("  • {}: {}\n", entry.key, entry.content));
        }
        Some(ctx)
    }

    /// Store a note to the project's memory.
    pub async fn store_note(&self, project: &str, key: &str, content: &str) -> Result<String> {
        let mem = self
            .memory_stores
            .get(project)
            .ok_or_else(|| anyhow::anyhow!("no memory store for project: {project}"))?;
        let id = mem
            .store(
                key,
                content,
                sigil_core::traits::MemoryCategory::Fact,
                MemoryScope::Domain,
                None,
            )
            .await?;
        Ok(id)
    }

    // ── Private helpers ──

    async fn handle_create_task(&self, msg: &ChatMessage) -> ChatResponse {
        let msg_lower = msg.message.to_lowercase();

        let project = if let Some(p) = &msg.project_hint {
            p.clone()
        } else {
            let mut found = String::new();
            for s in self.registry.list_project_summaries().await {
                if msg_lower.contains(&s.name.to_lowercase()) {
                    found = s.name.clone();
                    break;
                }
            }
            if found.is_empty() {
                self.registry
                    .list_project_summaries()
                    .await
                    .first()
                    .map(|s| s.name.clone())
                    .unwrap_or_default()
            } else {
                found
            }
        };

        let subject = msg_lower
            .replace("create a task", "")
            .replace("create task", "")
            .replace("new task", "")
            .replace("add a task", "")
            .replace("add task", "")
            .replace(&format!("in {}", project.to_lowercase()), "")
            .replace(&format!("for {}", project.to_lowercase()), "")
            .replace(" to ", " ")
            .trim()
            .trim_start_matches(':')
            .trim()
            .to_string();

        let subject = if subject.is_empty() {
            msg.message.clone()
        } else {
            let start = msg.message.to_lowercase().find(&subject).unwrap_or(0);
            if start + subject.len() <= msg.message.len() {
                msg.message[start..start + subject.len()].to_string()
            } else {
                subject
            }
        };

        match self.registry.assign(&project, &subject, "").await {
            Ok(task) => ChatResponse {
                ok: true,
                context: format!(
                    "Done. Created task {} in {} — \"{}\"",
                    task.id, project, subject
                ),
                action: Some("task_created".to_string()),
                task: Some(serde_json::json!({
                    "id": task.id.0,
                    "subject": task.subject,
                    "project": project,
                })),
                projects: None,
                cost: None,
                workers: None,
            },
            Err(e) => ChatResponse::error(&format!("Failed to create task: {}", e)),
        }
    }

    async fn handle_close_task(&self, msg: &ChatMessage) -> ChatResponse {
        let task_id: String = msg
            .message
            .split_whitespace()
            .find(|w| w.contains('-') && w.chars().any(|c| c.is_ascii_digit()))
            .unwrap_or("")
            .to_string();

        if task_id.is_empty() {
            return ChatResponse::error("I need a task ID to close (e.g., 'close task as-001').");
        }

        for name in self.registry.project_names().await {
            if let Some(board) = self.registry.get_task_board(&name).await {
                let mut board = board.lock().await;
                if board.get(&task_id).is_some() && board.close(&task_id, "closed via chat").is_ok()
                {
                    return ChatResponse {
                        ok: true,
                        context: format!("Done. Task {} is now closed.", task_id),
                        action: Some("task_closed".to_string()),
                        task: None,
                        projects: None,
                        cost: None,
                        workers: None,
                    };
                }
            }
        }

        ChatResponse::error(&format!("Couldn't find task {}.", task_id))
    }

    async fn handle_blackboard_post(&self, msg: &ChatMessage) -> ChatResponse {
        let content = msg
            .message
            .split_once(':')
            .map(|x| x.1)
            .unwrap_or("")
            .trim();
        let project = msg.project_hint.as_deref().unwrap_or("*");
        let key = format!("chat-note-{}", chrono::Utc::now().timestamp());

        // Store to memory (permanent knowledge).
        let memory_result = if project != "*" {
            self.store_note(project, &key, content).await.ok()
        } else {
            None
        };

        // Also store to blackboard (shared ephemeral knowledge).
        match &self.registry.blackboard {
            Some(bb) => {
                match bb.post(
                    &key,
                    content,
                    &self.leader_name,
                    project,
                    &[],
                    crate::blackboard::EntryDurability::Durable,
                ) {
                    Ok(_) => {
                        let stored_where = if memory_result.is_some() {
                            format!("Noted. Stored as knowledge in {}.", project)
                        } else {
                            format!("Noted. Saved to blackboard for {}.", project)
                        };
                        ChatResponse {
                            ok: true,
                            context: stored_where,
                            action: Some("knowledge_stored".to_string()),
                            task: None,
                            projects: None,
                            cost: None,
                            workers: None,
                        }
                    }
                    Err(e) => ChatResponse::error(&format!("Failed to save note: {}", e)),
                }
            }
            None => ChatResponse::error("Blackboard not initialized."),
        }
    }

    async fn classify_advisors(
        &self,
        clean_text: &str,
        is_council: bool,
        chat_id: i64,
    ) -> Vec<String> {
        if self.council_advisors.is_empty() {
            return Vec::new();
        }
        let advisor_refs: Vec<&sigil_core::config::PeerAgentConfig> =
            self.council_advisors.iter().collect();
        let route = {
            let mut r = self.agent_router.lock().await;
            r.classify(clean_text, &advisor_refs, chat_id).await
        };
        match route {
            Ok(decision) => {
                if is_council && decision.advisors.is_empty() {
                    self.council_advisors
                        .iter()
                        .map(|c| c.name.clone())
                        .collect()
                } else {
                    decision.advisors
                }
            }
            Err(e) => {
                warn!(error = %e, "classifier failed");
                Vec::new()
            }
        }
    }

    async fn gather_council_input(
        &self,
        advisors: &[String],
        clean_text: &str,
        conv_context: &str,
        chat_id: i64,
        source_tag: &str,
    ) -> Vec<(String, String)> {
        info!(advisors = ?advisors, "invoking council advisors");

        let mut handles = Vec::new();
        for advisor_name in advisors {
            let project_name = advisor_name.clone();
            let adv_name = advisor_name.clone();
            let adv_msg = clean_text.to_string();
            let adv_history = conv_context.to_string();
            let reg = self.registry.clone();

            let handle = tokio::spawn(async move {
                let task_subject = "[council] Advisor input requested".to_string();
                let task_desc = if adv_history.is_empty() {
                    format!(
                        "The user said:\n\n{}\n\n\
                         Provide your specialist perspective on this in character. \
                         Be concise (2-5 sentences). Focus on your domain expertise.",
                        adv_msg
                    )
                } else {
                    format!(
                        "{}\n\nThe user now says:\n\n{}\n\n\
                         Provide your specialist perspective on this in character. \
                         Be concise (2-5 sentences). Focus on your domain expertise.",
                        adv_history, adv_msg
                    )
                };

                let task_id = match reg.assign(&project_name, &task_subject, &task_desc).await {
                    Ok(b) => b.id.0.clone(),
                    Err(e) => {
                        warn!(agent = %adv_name, error = %e, "failed to create advisor task");
                        return None;
                    }
                };

                let notify = reg
                    .get_project(&project_name)
                    .await
                    .map(|d| d.task_notify.clone());
                let timeout = tokio::time::sleep(std::time::Duration::from_secs(60));
                tokio::pin!(timeout);
                loop {
                    tokio::select! {
                        _ = async {
                            match &notify {
                                Some(n) => n.notified().await,
                                None => std::future::pending::<()>().await,
                            }
                        } => {}
                        _ = &mut timeout => {
                            warn!(agent = %adv_name, "advisor task timed out");
                            return None;
                        }
                    }
                    let done = {
                        if let Some(rig) = reg.get_project(&project_name).await {
                            let store = rig.tasks.lock().await;
                            store.get(&task_id).map(|b| {
                                (
                                    b.status == sigil_tasks::TaskStatus::Done,
                                    b.closed_reason.clone(),
                                )
                            })
                        } else {
                            None
                        }
                    };
                    if let Some((true, reason)) = done {
                        let text = reason.unwrap_or_default();
                        return Some((adv_name, text));
                    }
                }
            });
            handles.push(handle);
        }

        // Record advisor responses in conversation history.
        let mut responses = Vec::new();
        for handle in handles {
            if let Ok(Some((name, text))) = handle.await
                && !text.trim().is_empty()
            {
                let capitalized = {
                    let mut c = name.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().to_string() + c.as_str(),
                    }
                };
                let _ = self
                    .conversations
                    .record_with_source(chat_id, &capitalized, text.trim(), Some(source_tag))
                    .await;
                responses.push((name, text.trim().to_string()));
            }
        }

        responses
    }
}
