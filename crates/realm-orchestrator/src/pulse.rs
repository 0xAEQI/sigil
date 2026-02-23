use anyhow::Result;
use realm_core::traits::{LogObserver, Observer, Provider, Tool};
use realm_core::{Agent, AgentConfig, Identity};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::whisper::{Whisper, WhisperBus, WhisperKind};

/// Pulse: periodic checks driven by HEARTBEAT.md instructions.
/// Each rig's witness runs a pulse on a configurable interval.
/// The agent evaluates checks and reports anomalies.
pub struct Pulse {
    pub domain_name: String,
    pub interval_secs: u64,
    pub instructions: String,
    pub provider: Arc<dyn Provider>,
    pub tools: Vec<Arc<dyn Tool>>,
    pub identity: Identity,
    pub model: String,
    pub whisper_bus: Arc<WhisperBus>,
    last_run: Option<std::time::Instant>,
}

impl Pulse {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_name: String,
        interval_secs: u64,
        instructions: String,
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn Tool>>,
        identity: Identity,
        model: String,
        whisper_bus: Arc<WhisperBus>,
    ) -> Self {
        Self {
            domain_name,
            interval_secs,
            instructions,
            provider,
            tools,
            identity,
            model,
            whisper_bus,
            last_run: None,
        }
    }

    /// Check if a pulse is due.
    pub fn is_due(&self) -> bool {
        match self.last_run {
            None => true,
            Some(last) => last.elapsed().as_secs() >= self.interval_secs,
        }
    }

    /// Run one pulse cycle. Returns the agent's assessment.
    pub async fn run(&mut self) -> Result<String> {
        if self.instructions.is_empty() {
            return Ok("(no pulse instructions)".to_string());
        }

        debug!(domain = %self.domain_name, "running pulse");

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
            name: format!("{}-pulse", self.domain_name),
            ..Default::default()
        };

        let agent = Agent::new(
            agent_config,
            self.provider.clone(),
            self.tools.clone(),
            observer,
            self.identity.clone(),
        );

        let result = agent.run(&prompt).await?;
        self.last_run = Some(std::time::Instant::now());

        // Determine if there are issues.
        let is_ok = result.to_lowercase().contains("all ok")
            || result.to_lowercase().contains("all clear")
            || result.to_lowercase().contains("no issues");

        if is_ok {
            info!(domain = %self.domain_name, "pulse: all OK");
        } else {
            warn!(domain = %self.domain_name, "pulse: issues detected");
            self.whisper_bus
                .send(Whisper::new_typed(
                    &format!("pulse-{}", self.domain_name),
                    "familiar",
                    WhisperKind::PulseAlert {
                        domain: self.domain_name.clone(),
                        issues: result.clone(),
                    },
                ))
                .await;
        }

        Ok(result)
    }
}
