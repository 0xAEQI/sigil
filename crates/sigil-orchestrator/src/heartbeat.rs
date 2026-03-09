use anyhow::Result;
use sigil_core::traits::{LogObserver, Observer, Provider, Tool};
use sigil_core::{Agent, AgentConfig, Identity};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::message::{Dispatch, DispatchBus, DispatchKind};

/// Heartbeat: periodic checks driven by HEARTBEAT.md instructions.
/// Each project's supervisor runs a heartbeat on a configurable interval.
/// The agent evaluates checks and reports anomalies.
pub struct Heartbeat {
    pub project_name: String,
    pub interval_secs: u64,
    pub instructions: String,
    pub provider: Arc<dyn Provider>,
    pub tools: Vec<Arc<dyn Tool>>,
    pub identity: Identity,
    pub model: String,
    pub dispatch_bus: Arc<DispatchBus>,
    last_run: Option<std::time::Instant>,
}

impl Heartbeat {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_name: String,
        interval_secs: u64,
        instructions: String,
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn Tool>>,
        identity: Identity,
        model: String,
        dispatch_bus: Arc<DispatchBus>,
    ) -> Self {
        Self {
            project_name,
            interval_secs,
            instructions,
            provider,
            tools,
            identity,
            model,
            dispatch_bus,
            last_run: None,
        }
    }

    /// Check if a heartbeat is due.
    pub fn is_due(&self) -> bool {
        match self.last_run {
            None => true,
            Some(last) => last.elapsed().as_secs() >= self.interval_secs,
        }
    }

    /// Run one heartbeat cycle. Returns the agent's assessment.
    pub async fn run(&mut self) -> Result<String> {
        if self.instructions.is_empty() {
            return Ok("(no heartbeat instructions)".to_string());
        }

        debug!(project = %self.project_name, "running heartbeat");

        let prompt = format!(
            "Run the following periodic health checks. \
             If everything is OK, respond with a brief 'ALL OK' summary. \
             If anything needs attention, describe the issue clearly.\n\n\
             ---\n\n{}",
            self.instructions
        );

        let observer: Arc<dyn Observer> = Arc::new(LogObserver);
        let agent_config = AgentConfig {
            model: self.model.clone(),
            max_iterations: 10,
            name: format!("{}-heartbeat", self.project_name),
            ..Default::default()
        };

        let agent = Agent::new(
            agent_config,
            self.provider.clone(),
            self.tools.clone(),
            observer,
            self.identity.clone(),
        );

        let agent_result = agent.run(&prompt).await?;
        self.last_run = Some(std::time::Instant::now());

        let text = agent_result.text;

        // Determine if there are issues.
        let is_ok = text.to_lowercase().contains("all ok")
            || text.to_lowercase().contains("all clear")
            || text.to_lowercase().contains("no issues");

        if is_ok {
            info!(project = %self.project_name, "heartbeat: all OK");
        } else {
            warn!(project = %self.project_name, "heartbeat: issues detected");
            self.dispatch_bus
                .send(Dispatch::new_typed(
                    &format!("heartbeat-{}", self.project_name),
                    "leader",
                    DispatchKind::HeartbeatAlert {
                        project: self.project_name.clone(),
                        issues: text.clone(),
                    },
                ))
                .await;
        }

        Ok(text)
    }
}
