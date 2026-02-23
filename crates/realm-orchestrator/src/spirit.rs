use anyhow::Result;
use chrono::Utc;
use realm_quests::{Checkpoint, Quest, QuestStatus};
use realm_core::traits::{
    ChatRequest, LogObserver, Memory, MemoryCategory, MemoryScope, Message, MessageContent,
    Observer, Provider, Role, Tool,
};
use realm_core::{Agent, AgentConfig, Identity};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::checkpoint::SpiritCheckpoint;
use crate::executor::{ClaudeCodeExecutor, QuestOutcome};
use crate::bond::Bond;
use crate::whisper::{Whisper, WhisperBus, WhisperKind};

/// Spirit states.
#[derive(Debug, Clone, PartialEq)]
pub enum SpiritState {
    Idle,
    Hooked,
    Working,
    Done,
    Failed(String),
}

/// How a worker executes its assigned bead.
pub enum SpiritExecution {
    /// Internal Agent loop (current behavior): LLM API calls with basic tools.
    Agent {
        provider: Arc<dyn realm_core::traits::Provider>,
        tools: Vec<Arc<dyn Tool>>,
        model: String,
    },
    /// Claude Code CLI subprocess: full Edit, Grep, Glob, context compression.
    ClaudeCode(ClaudeCodeExecutor),
}

/// A Spirit is an ephemeral task executor. Each worker runs as a tokio task
/// with its own identity, hook, and tool allowlist.
pub struct Spirit {
    pub name: String,
    pub domain_name: String,
    pub state: SpiritState,
    pub hook: Option<Bond>,
    pub execution: SpiritExecution,
    pub identity: Identity,
    pub whisper_bus: Arc<WhisperBus>,
    pub beads: Arc<Mutex<realm_quests::QuestBoard>>,
    pub memory: Option<Arc<dyn Memory>>,
    pub reflect_provider: Option<Arc<dyn Provider>>,
    pub reflect_model: String,
    /// Rig directory path for checkpoint storage.
    pub rig_dir: Option<PathBuf>,
}

impl Spirit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        domain_name: String,
        provider: Arc<dyn realm_core::traits::Provider>,
        tools: Vec<Arc<dyn Tool>>,
        identity: Identity,
        model: String,
        whisper_bus: Arc<WhisperBus>,
        beads: Arc<Mutex<realm_quests::QuestBoard>>,
    ) -> Self {
        let reflect_model = model.clone();
        Self {
            name,
            domain_name,
            state: SpiritState::Idle,
            hook: None,
            execution: SpiritExecution::Agent { provider, tools, model },
            identity,
            whisper_bus,
            beads,
            memory: None,
            reflect_provider: None,
            reflect_model,
            rig_dir: None,
        }
    }

    pub fn new_claude_code(
        name: String,
        domain_name: String,
        executor: ClaudeCodeExecutor,
        identity: Identity,
        whisper_bus: Arc<WhisperBus>,
        beads: Arc<Mutex<realm_quests::QuestBoard>>,
    ) -> Self {
        let rig_dir = Some(executor.workdir().to_path_buf());
        Self {
            name,
            domain_name,
            state: SpiritState::Idle,
            hook: None,
            execution: SpiritExecution::ClaudeCode(executor),
            identity,
            whisper_bus,
            beads,
            memory: None,
            reflect_provider: None,
            reflect_model: String::new(),
            rig_dir,
        }
    }

    pub fn with_memory(mut self, memory: Arc<dyn Memory>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn with_reflect(mut self, provider: Arc<dyn Provider>, model: String) -> Self {
        self.reflect_provider = Some(provider);
        self.reflect_model = model;
        self
    }

    pub fn with_rig_dir(mut self, rig_dir: PathBuf) -> Self {
        self.rig_dir = Some(rig_dir);
        self
    }

    /// Get the working directory for this spirit (from executor or rig_dir).
    fn workdir(&self) -> Option<&std::path::Path> {
        match &self.execution {
            SpiritExecution::ClaudeCode(executor) => Some(executor.workdir()),
            SpiritExecution::Agent { .. } => self.rig_dir.as_deref(),
        }
    }

    /// Capture an external checkpoint by inspecting git state in the spirit's workdir.
    /// Saves the checkpoint to the rig's `.sigil/checkpoints/` directory.
    fn capture_and_save_checkpoint(&self, quest_id: &str, progress_notes: Option<&str>) {
        let Some(workdir) = self.workdir() else {
            debug!(worker = %self.name, "no workdir — skipping checkpoint capture");
            return;
        };

        let rig_dir = self.rig_dir.as_deref().unwrap_or(workdir);

        match SpiritCheckpoint::capture(workdir) {
            Ok(checkpoint) => {
                let checkpoint: SpiritCheckpoint = checkpoint
                    .with_quest_id(quest_id)
                    .with_spirit_name(&self.name);

                let checkpoint = if let Some(notes) = progress_notes {
                    checkpoint.with_progress_notes(notes)
                } else {
                    checkpoint
                };

                let cp_path = SpiritCheckpoint::path_for_quest(rig_dir, quest_id);
                if let Err(e) = checkpoint.write(&cp_path) {
                    warn!(
                        worker = %self.name,
                        quest = %quest_id,
                        error = %e,
                        "failed to write checkpoint"
                    );
                } else {
                    info!(
                        worker = %self.name,
                        quest = %quest_id,
                        files = checkpoint.modified_files.len(),
                        "external checkpoint captured"
                    );
                }
            }
            Err(e) => {
                warn!(
                    worker = %self.name,
                    quest = %quest_id,
                    error = %e,
                    "failed to capture git checkpoint"
                );
            }
        }
    }

    /// Assign a bead to this worker (set hook).
    pub fn assign(&mut self, bead: &Quest) {
        self.hook = Some(Bond::new(bead.id.clone(), bead.subject.clone()));
        self.state = SpiritState::Hooked;
    }

    /// Save a checkpoint recording this spirit's progress on a quest.
    async fn save_checkpoint(&self, quest_id: &str, progress: &str, cost: f64, turns: u32) {
        let mut store = self.beads.lock().await;
        if let Err(e) = store.update(quest_id, |q| {
            q.checkpoints.push(Checkpoint {
                timestamp: Utc::now(),
                spirit: self.name.clone(),
                progress: progress.to_string(),
                cost_usd: cost,
                turns_used: turns,
            });
        }) {
            warn!(quest_id, error = %e, "failed to save checkpoint to quest store");
        }
    }

    /// Execute the hooked work. Dispatches to Agent or Claude Code based on execution mode.
    /// Returns (outcome, cost_usd, turns_used) for the Scout to record.
    pub async fn execute(&mut self) -> Result<(QuestOutcome, f64, u32)> {
        let hook = match &self.hook {
            Some(h) => h.clone(),
            None => {
                warn!(worker = %self.name, "no hook assigned, nothing to do");
                return Ok((QuestOutcome::Done("no work assigned".to_string()), 0.0, 0));
            }
        };

        info!(
            worker = %self.name,
            bead = %hook.quest_id,
            subject = %hook.subject,
            mode = match &self.execution {
                SpiritExecution::Agent { .. } => "agent",
                SpiritExecution::ClaudeCode(_) => "claude_code",
            },
            "starting work"
        );

        self.state = SpiritState::Working;

        // Mark bead as in_progress.
        {
            let mut store = self.beads.lock().await;
            if let Err(e) = store.update(&hook.quest_id.0, |b| {
                b.status = QuestStatus::InProgress;
                b.assignee = Some(self.name.clone());
            }) {
                warn!(bead = %hook.quest_id, error = %e, "failed to mark quest in_progress");
            }
        }

        // Build the prompt from the bead (including any previous checkpoints).
        let quest_context = {
            let store = self.beads.lock().await;
            match store.get(&hook.quest_id.0) {
                Some(b) => {
                    let mut ctx = format!("## Task: {}\n\n", b.subject);
                    if !b.description.is_empty() {
                        ctx.push_str(&format!("{}\n\n", b.description));
                    }
                    ctx.push_str(&format!("Quest ID: {}\nPriority: {}\n", b.id, b.priority));

                    // Include budgeted checkpoints from previous attempts.
                    if !b.checkpoints.is_empty() {
                        let budget = crate::context_budget::ContextBudget::default();
                        ctx.push_str(&budget.budget_checkpoints(&b.checkpoints));
                        ctx.push_str("Review the above before starting. Skip work that's already done.\n\n");
                    }

                    // Include acceptance criteria if defined.
                    if let Some(ref criteria) = b.acceptance_criteria {
                        ctx.push_str(&format!(
                            "\n## Acceptance Criteria\n\n{}\n\n\
                             Verify your work meets these criteria before marking as DONE.\n\n",
                            criteria
                        ));
                    }

                    ctx
                }
                None => format!("Task: {}", hook.subject),
            }
        };

        // Inject recalled memories into quest context for richer execution.
        let quest_context = if let Some(ref mem) = self.memory {
            let query = realm_core::traits::MemoryQuery::new(&quest_context, 5)
                .with_scope(realm_core::traits::MemoryScope::Domain);
            match mem.search(&query).await {
                Ok(entries) if !entries.is_empty() => {
                    let ctx = entries
                        .iter()
                        .map(|e| format!("[{}] {}: {}", e.scope, e.key, e.content))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{quest_context}\n## Recalled Memory\n{ctx}\n")
                }
                _ => quest_context,
            }
        } else {
            quest_context
        };

        // Dispatch based on execution mode. Returns (text, cost_usd, turns_used).
        let raw_result = match &self.execution {
            SpiritExecution::Agent { provider, tools, model } => {
                self.execute_agent(provider.clone(), tools.clone(), model, &quest_context)
                    .await
                    .map(|agent_result| {
                        let cost = realm_providers::estimate_cost(
                            &agent_result.model,
                            agent_result.total_prompt_tokens,
                            agent_result.total_completion_tokens,
                        );
                        info!(
                            worker = %self.name,
                            model = %agent_result.model,
                            prompt_tokens = agent_result.total_prompt_tokens,
                            completion_tokens = agent_result.total_completion_tokens,
                            cost_usd = cost,
                            iterations = agent_result.iterations,
                            "agent execution cost calculated"
                        );
                        (agent_result.text, cost, agent_result.iterations)
                    })
            }
            SpiritExecution::ClaudeCode(executor) => {
                self.execute_claude_code(executor, &quest_context).await
            }
        };

        // Parse into structured outcome.
        let (outcome, cost, turns) = match raw_result {
            Ok((result_text, cost, turns)) => (QuestOutcome::parse(&result_text), cost, turns),
            Err(e) => (QuestOutcome::Failed(e.to_string()), 0.0, 0),
        };

        // Process outcome: save checkpoint, update bead status, notify scout.
        match &outcome {
            QuestOutcome::Done(result_text) => {
                info!(worker = %self.name, bead = %hook.quest_id, "work completed");
                // Capture external checkpoint from git state before recording completion.
                self.capture_and_save_checkpoint(
                    &hook.quest_id.0,
                    Some(&format!("DONE: {}", result_text)),
                );
                self.save_checkpoint(
                    &hook.quest_id.0,
                    &format!("DONE: {}", result_text),
                    cost,
                    turns,
                ).await;
                {
                    let mut store = self.beads.lock().await;
                    let _ = store.close(&hook.quest_id.0, result_text);
                }
                self.whisper_bus
                    .send(Whisper::new_typed(
                        &self.name,
                        &format!("witness-{}", self.domain_name),
                        WhisperKind::QuestDone {
                            quest_id: hook.quest_id.to_string(),
                            summary: format!("{}: {}", hook.subject, result_text),
                        },
                    ))
                    .await;
                self.state = SpiritState::Done;
            }

            QuestOutcome::Blocked { question, full_text } => {
                info!(
                    worker = %self.name,
                    bead = %hook.quest_id,
                    question = %question,
                    "worker blocked — needs input"
                );
                // Capture external checkpoint from git state before recording block.
                self.capture_and_save_checkpoint(
                    &hook.quest_id.0,
                    Some(&format!("BLOCKED: {}\n\nWork so far:\n{}", question, full_text)),
                );
                self.save_checkpoint(
                    &hook.quest_id.0,
                    &format!("BLOCKED on: {}\n\nWork done so far:\n{}", question, full_text),
                    cost,
                    turns,
                ).await;
                // Mark bead as Blocked and preserve the question for Scout resolution.
                {
                    let mut store = self.beads.lock().await;
                    if let Err(e) = store.update(&hook.quest_id.0, |b| {
                        b.status = QuestStatus::Blocked;
                        b.assignee = None;
                        b.closed_reason = Some(question.clone());
                    }) {
                        warn!(bead = %hook.quest_id, error = %e, "failed to mark quest blocked");
                    }
                }
                self.whisper_bus
                    .send(Whisper::new_typed(
                        &self.name,
                        &format!("witness-{}", self.domain_name),
                        WhisperKind::QuestBlocked {
                            quest_id: hook.quest_id.to_string(),
                            question: question.clone(),
                            context: full_text.clone(),
                        },
                    ))
                    .await;
                self.state = SpiritState::Done; // Spirit is done; bead is blocked.
            }

            QuestOutcome::Handoff { checkpoint } => {
                info!(worker = %self.name, bead = %hook.quest_id, "spirit handing off — context exhaustion");
                // Capture external checkpoint from git state before recording handoff.
                self.capture_and_save_checkpoint(
                    &hook.quest_id.0,
                    Some(&format!("HANDOFF: {}", checkpoint)),
                );
                self.save_checkpoint(
                    &hook.quest_id.0,
                    &format!("HANDOFF: {}", checkpoint),
                    cost,
                    turns,
                ).await;
                {
                    let mut store = self.beads.lock().await;
                    if let Err(e) = store.update(&hook.quest_id.0, |b| {
                        b.status = QuestStatus::Pending;
                        b.assignee = None;
                    }) {
                        warn!(bead = %hook.quest_id, error = %e, "failed to re-queue quest after handoff");
                    }
                }
                self.whisper_bus
                    .send(Whisper::new_typed(
                        &self.name,
                        &format!("witness-{}", self.domain_name),
                        WhisperKind::QuestBlocked {
                            quest_id: hook.quest_id.to_string(),
                            question: "Context exhaustion handoff — re-queued automatically".to_string(),
                            context: checkpoint.clone(),
                        },
                    ))
                    .await;
                self.state = SpiritState::Done;
            }

            QuestOutcome::Failed(error_text) => {
                warn!(worker = %self.name, bead = %hook.quest_id, "work failed");
                // Capture external checkpoint from git state before recording failure.
                self.capture_and_save_checkpoint(
                    &hook.quest_id.0,
                    Some(&format!("FAILED: {}", error_text)),
                );
                self.save_checkpoint(
                    &hook.quest_id.0,
                    &format!("FAILED: {}", error_text),
                    cost,
                    turns,
                ).await;
                {
                    let mut store = self.beads.lock().await;
                    if let Err(e) = store.update(&hook.quest_id.0, |b| {
                        b.status = QuestStatus::Pending;
                        b.assignee = None;
                    }) {
                        warn!(bead = %hook.quest_id, error = %e, "failed to re-queue quest after failure");
                    }
                }
                self.whisper_bus
                    .send(Whisper::new_typed(
                        &self.name,
                        &format!("witness-{}", self.domain_name),
                        WhisperKind::QuestFailed {
                            quest_id: hook.quest_id.to_string(),
                            error: error_text.clone(),
                        },
                    ))
                    .await;
                self.state = SpiritState::Failed(error_text.to_string());
            }
        }

        self.hook = None;
        Ok((outcome, cost, turns))
    }

    async fn execute_agent(
        &self,
        provider: Arc<dyn realm_core::traits::Provider>,
        tools: Vec<Arc<dyn Tool>>,
        model: &str,
        quest_context: &str,
    ) -> Result<realm_core::AgentResult> {
        let observer: Arc<dyn Observer> = Arc::new(LogObserver);
        let agent_config = AgentConfig {
            model: model.to_string(),
            max_iterations: 20,
            name: self.name.clone(),
            ..Default::default()
        };

        let mut agent = Agent::new(
            agent_config,
            provider,
            tools,
            observer,
            self.identity.clone(),
        );

        if let Some(ref mem) = self.memory {
            agent = agent.with_memory(mem.clone());
        }

        agent.run(quest_context).await
    }

    /// Execute via Claude Code CLI subprocess. Returns (text, cost_usd, turns_used).
    async fn execute_claude_code(
        &self,
        executor: &ClaudeCodeExecutor,
        quest_context: &str,
    ) -> Result<(String, f64, u32)> {
        let result = executor.execute(&self.identity, quest_context).await?;

        info!(
            worker = %self.name,
            turns = result.num_turns,
            cost_usd = result.total_cost_usd,
            duration_ms = result.duration_ms,
            "claude code execution completed"
        );

        self.reflect_on_result(quest_context, &result.result_text).await;

        Ok((result.result_text, result.total_cost_usd, result.num_turns))
    }

    async fn reflect_on_result(&self, quest_context: &str, result_text: &str) {
        let Some(ref mem) = self.memory else { return };
        let Some(ref provider) = self.reflect_provider else { return };

        let transcript = format!("User: {}\n\nAssistant: {}", quest_context, result_text);
        if transcript.len() < 100 {
            return;
        }

        let max_len = 8000;
        let truncated = if transcript.len() > max_len {
            &transcript[..max_len]
        } else {
            &transcript
        };

        let reflection_prompt = format!(
            "You are a memory extraction system. Analyze this conversation and extract ONLY \
             genuinely important insights worth remembering long-term. Output NOTHING if the \
             conversation is trivial.\n\n\
             For each insight, output exactly one line in this format:\n\
             SCOPE CATEGORY: key-slug | The insight content\n\n\
             Scopes (choose the most appropriate):\n\
             - DOMAIN: Technical facts about this specific project/codebase\n\
             - REALM: Insights about the Emperor (preferences, decisions, patterns that span domains)\n\
             - SELF: Your own observations, reflections, learnings as a companion\n\n\
             Categories:\n\
             - FACT: Factual information (technical details, architecture decisions, numbers)\n\
             - PROCEDURE: How something works or should be done\n\
             - PREFERENCE: User preferences, opinions, behavioral patterns\n\
             - CONTEXT: Decisions made, strategic shifts, project state changes\n\n\
             Rules:\n\
             - Maximum 5 insights per conversation\n\
             - Each insight must be self-contained\n\
             - key-slug: 2-4 lowercase hyphenated words\n\
             - Content: one concise sentence\n\
             - If nothing is worth remembering, output exactly: NONE\n\n\
             ## Conversation\n\n{}",
            truncated
        );

        let request = ChatRequest {
            model: self.reflect_model.clone(),
            messages: vec![Message {
                role: Role::User,
                content: MessageContent::text(&reflection_prompt),
            }],
            tools: vec![],
            max_tokens: 512,
            temperature: 0.0,
        };

        match provider.chat(&request).await {
            Ok(response) => {
                if let Some(text) = response.content {
                    self.store_routed_insights(&text, mem).await;
                }
            }
            Err(e) => warn!(worker = %self.name, "reflection failed: {e}"),
        }
    }

    async fn store_routed_insights(&self, text: &str, mem: &Arc<dyn Memory>) {
        for line in text.lines() {
            let line = line.trim();
            if line == "NONE" || line.is_empty() {
                continue;
            }

            let (scope, rest) = if let Some(r) = line.strip_prefix("DOMAIN ") {
                (MemoryScope::Domain, r)
            } else if let Some(r) = line.strip_prefix("REALM ") {
                (MemoryScope::Realm, r)
            } else if let Some(r) = line.strip_prefix("SELF ") {
                (MemoryScope::Companion, r)
            } else if let Some((cat_str, _rest)) = line.split_once(':') {
                let cat_str = cat_str.trim();
                if matches!(cat_str, "FACT" | "PROCEDURE" | "PREFERENCE" | "CONTEXT") {
                    (MemoryScope::Domain, line)
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let Some((cat_str, rest)) = rest.split_once(':') else {
                continue;
            };
            let Some((key, content)) = rest.split_once('|') else {
                continue;
            };

            let category = match cat_str.trim().to_uppercase().as_str() {
                "FACT" => MemoryCategory::Fact,
                "PROCEDURE" => MemoryCategory::Procedure,
                "PREFERENCE" => MemoryCategory::Preference,
                "CONTEXT" => MemoryCategory::Context,
                _ => continue,
            };

            let key = key.trim();
            let content = content.trim();
            if key.is_empty() || content.is_empty() {
                continue;
            }

            let companion_id = if scope == MemoryScope::Companion {
                Some(self.name.as_str())
            } else {
                None
            };

            match mem.store(key, content, category, scope, companion_id).await {
                Ok(id) => {
                    debug!(worker = %self.name, id = %id, key = %key, scope = %scope, "insight stored")
                }
                Err(e) => {
                    warn!(worker = %self.name, key = %key, "failed to store insight: {e}")
                }
            }
        }
    }
}
