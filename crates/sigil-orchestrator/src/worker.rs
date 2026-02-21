use anyhow::Result;
use sigil_beads::{Bead, BeadStatus};
use sigil_core::traits::{LogObserver, Observer, Tool};
use sigil_core::{Agent, AgentConfig, Identity};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::executor::ClaudeCodeExecutor;
use crate::hook::Hook;
use crate::mail::{Mail, MailBus};

/// Worker states.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerState {
    Idle,
    Hooked,
    Working,
    Done,
    Failed(String),
}

/// How a worker executes its assigned bead.
pub enum WorkerExecution {
    /// Internal Agent loop (current behavior): LLM API calls with basic tools.
    Agent {
        provider: Arc<dyn sigil_core::traits::Provider>,
        tools: Vec<Arc<dyn Tool>>,
        model: String,
    },
    /// Claude Code CLI subprocess: full Edit, Grep, Glob, context compression.
    ClaudeCode(ClaudeCodeExecutor),
}

/// A Worker is an ephemeral task executor. Each worker runs as a tokio task
/// with its own identity, hook, and tool allowlist.
pub struct Worker {
    pub name: String,
    pub rig_name: String,
    pub state: WorkerState,
    pub hook: Option<Hook>,
    pub execution: WorkerExecution,
    pub identity: Identity,
    pub mail_bus: Arc<MailBus>,
    pub beads: Arc<Mutex<sigil_beads::BeadStore>>,
}

impl Worker {
    /// Create a worker with Agent execution mode (lightweight LLM API loop).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        rig_name: String,
        provider: Arc<dyn sigil_core::traits::Provider>,
        tools: Vec<Arc<dyn Tool>>,
        identity: Identity,
        model: String,
        mail_bus: Arc<MailBus>,
        beads: Arc<Mutex<sigil_beads::BeadStore>>,
    ) -> Self {
        Self {
            name,
            rig_name,
            state: WorkerState::Idle,
            hook: None,
            execution: WorkerExecution::Agent { provider, tools, model },
            identity,
            mail_bus,
            beads,
        }
    }

    /// Create a worker with Claude Code execution mode (full CLI harness).
    pub fn new_claude_code(
        name: String,
        rig_name: String,
        executor: ClaudeCodeExecutor,
        identity: Identity,
        mail_bus: Arc<MailBus>,
        beads: Arc<Mutex<sigil_beads::BeadStore>>,
    ) -> Self {
        Self {
            name,
            rig_name,
            state: WorkerState::Idle,
            hook: None,
            execution: WorkerExecution::ClaudeCode(executor),
            identity,
            mail_bus,
            beads,
        }
    }

    /// Assign a bead to this worker (set hook).
    pub fn assign(&mut self, bead: &Bead) {
        self.hook = Some(Hook::new(bead.id.clone(), bead.subject.clone()));
        self.state = WorkerState::Hooked;
    }

    /// Execute the hooked work. Dispatches to Agent or Claude Code based on execution mode.
    pub async fn execute(&mut self) -> Result<()> {
        let hook = match &self.hook {
            Some(h) => h.clone(),
            None => {
                warn!(worker = %self.name, "no hook assigned, nothing to do");
                return Ok(());
            }
        };

        info!(
            worker = %self.name,
            bead = %hook.bead_id,
            subject = %hook.subject,
            mode = match &self.execution {
                WorkerExecution::Agent { .. } => "agent",
                WorkerExecution::ClaudeCode(_) => "claude_code",
            },
            "starting work"
        );

        self.state = WorkerState::Working;

        // Mark bead as in_progress.
        {
            let mut store = self.beads.lock().await;
            let _ = store.update(&hook.bead_id.0, |b| {
                b.status = BeadStatus::InProgress;
                b.assignee = Some(self.name.clone());
            });
        }

        // Build the prompt from the bead.
        let bead_context = {
            let store = self.beads.lock().await;
            match store.get(&hook.bead_id.0) {
                Some(b) => {
                    let mut ctx = format!("## Task: {}\n\n", b.subject);
                    if !b.description.is_empty() {
                        ctx.push_str(&format!("{}\n\n", b.description));
                    }
                    ctx.push_str(&format!("Bead ID: {}\nPriority: {}\n", b.id, b.priority));
                    ctx
                }
                None => format!("Task: {}", hook.subject),
            }
        };

        // Dispatch based on execution mode.
        let result = match &self.execution {
            WorkerExecution::Agent { provider, tools, model } => {
                self.execute_agent(provider.clone(), tools.clone(), model, &bead_context).await
            }
            WorkerExecution::ClaudeCode(executor) => {
                self.execute_claude_code(executor, &bead_context).await
            }
        };

        match result {
            Ok(result_text) => {
                info!(worker = %self.name, bead = %hook.bead_id, "work completed");

                // Close the bead.
                {
                    let mut store = self.beads.lock().await;
                    let _ = store.close(&hook.bead_id.0, &result_text);
                }

                // Notify witness.
                self.mail_bus
                    .send(Mail::new(
                        &self.name,
                        &format!("witness-{}", self.rig_name),
                        "DONE",
                        &format!("Completed bead {}: {}", hook.bead_id, hook.subject),
                    ))
                    .await;

                self.state = WorkerState::Done;
            }
            Err(e) => {
                warn!(worker = %self.name, bead = %hook.bead_id, error = %e, "work failed");

                // Mark bead back to pending.
                {
                    let mut store = self.beads.lock().await;
                    let _ = store.update(&hook.bead_id.0, |b| {
                        b.status = BeadStatus::Pending;
                        b.assignee = None;
                    });
                }

                // Notify witness of failure.
                self.mail_bus
                    .send(Mail::new(
                        &self.name,
                        &format!("witness-{}", self.rig_name),
                        "FAILED",
                        &format!("Failed bead {}: {}", hook.bead_id, e),
                    ))
                    .await;

                self.state = WorkerState::Failed(e.to_string());
            }
        }

        self.hook = None;
        Ok(())
    }

    /// Execute via internal Agent loop (lightweight LLM API calls).
    async fn execute_agent(
        &self,
        provider: Arc<dyn sigil_core::traits::Provider>,
        tools: Vec<Arc<dyn Tool>>,
        model: &str,
        bead_context: &str,
    ) -> Result<String> {
        let observer: Arc<dyn Observer> = Arc::new(LogObserver);
        let agent_config = AgentConfig {
            model: model.to_string(),
            max_iterations: 20,
            name: self.name.clone(),
            ..Default::default()
        };

        let agent = Agent::new(
            agent_config,
            provider,
            tools,
            observer,
            self.identity.clone(),
        );

        agent.run(bead_context).await
    }

    /// Execute via Claude Code CLI subprocess.
    async fn execute_claude_code(
        &self,
        executor: &ClaudeCodeExecutor,
        bead_context: &str,
    ) -> Result<String> {
        let result = executor.execute(&self.identity, bead_context).await?;

        info!(
            worker = %self.name,
            turns = result.num_turns,
            cost_usd = result.total_cost_usd,
            duration_ms = result.duration_ms,
            "claude code execution completed"
        );

        Ok(result.result_text)
    }
}
