//! Chamber Mode — Visible council debate orchestration.
//!
//! When triggered (via `/council` command or Aurelia's judgment), the chamber
//! spawns a Telegram thread where each familiar debates visibly before
//! Aurelia synthesizes the final recommendation.

use anyhow::Result;
use realm_core::config::FamiliarConfig;
use realm_core::identity::Identity;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::executor::ClaudeCodeExecutor;

/// A chamber debate session.
#[derive(Debug, Clone)]
pub struct ChamberTopic {
    pub id: String,
    pub message: String,
    pub familiars: Vec<String>,
    pub responses: HashMap<String, String>,
    pub synthesis: Option<String>,
}

/// Manages chamber debate sessions.
pub struct Chamber {
    /// Active debate topics.
    topics: Mutex<HashMap<String, ChamberTopic>>,
    /// Counter for topic IDs.
    next_id: Mutex<u32>,
}

impl Chamber {
    pub fn new() -> Self {
        Self {
            topics: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Create a new chamber topic and return its ID.
    pub async fn open_topic(&self, message: &str, familiars: Vec<String>) -> String {
        let mut id_counter = self.next_id.lock().await;
        let id = format!("chamber-{:03}", *id_counter);
        *id_counter += 1;

        let topic = ChamberTopic {
            id: id.clone(),
            message: message.to_string(),
            familiars,
            responses: HashMap::new(),
            synthesis: None,
        };

        self.topics.lock().await.insert(id.clone(), topic);
        id
    }

    /// Record a familiar's response in a chamber debate.
    pub async fn record_response(&self, topic_id: &str, familiar: &str, response: &str) -> bool {
        let mut topics = self.topics.lock().await;
        if let Some(topic) = topics.get_mut(topic_id) {
            topic.responses.insert(familiar.to_string(), response.to_string());
            // Return true if all familiars have responded.
            topic.responses.len() == topic.familiars.len()
        } else {
            false
        }
    }

    /// Record the lead familiar's synthesis.
    pub async fn record_synthesis(&self, topic_id: &str, synthesis: &str) {
        let mut topics = self.topics.lock().await;
        if let Some(topic) = topics.get_mut(topic_id) {
            topic.synthesis = Some(synthesis.to_string());
        }
    }

    /// Get a topic by ID.
    pub async fn get_topic(&self, topic_id: &str) -> Option<ChamberTopic> {
        self.topics.lock().await.get(topic_id).cloned()
    }

    /// Run a chamber debate: spawn all advisor quests, collect responses,
    /// then have the lead synthesize.
    ///
    /// Returns (advisor_responses, synthesis_text).
    pub async fn run_debate(
        &self,
        message: &str,
        advisor_configs: &[(&FamiliarConfig, PathBuf)],
        lead_identity: &Identity,
        lead_repo: &Path,
        lead_model: &str,
    ) -> Result<(Vec<(String, String)>, String)> {
        let familiar_names: Vec<String> = advisor_configs
            .iter()
            .map(|(c, _)| c.name.clone())
            .collect();
        let topic_id = self.open_topic(message, familiar_names).await;

        info!(topic = %topic_id, "chamber debate opened");

        // Spawn advisor quests in parallel.
        let mut handles = Vec::new();
        for (config, rig_dir) in advisor_configs {
            let advisor_name = config.name.clone();
            let advisor_model = config.model.clone().unwrap_or_else(|| "claude-sonnet-4-6".to_string());
            let advisor_identity = Identity::load(rig_dir).unwrap_or_default();
            let msg = message.to_string();
            let repo = rig_dir.clone();
            let budget = config.max_budget_usd;
            let tid = topic_id.clone();

            let handle = tokio::spawn(async move {
                let quest_context = format!(
                    "## Chamber Debate\n\nThe council has been summoned to debate:\n\n{}\n\n\
                     Respond in character with your specialist perspective. Be concise (2-5 sentences).",
                    msg
                );

                let executor = ClaudeCodeExecutor::new(
                    repo,
                    advisor_model,
                    15, // short turns for advisory
                    budget,
                );

                match executor.execute(&advisor_identity, &quest_context).await {
                    Ok(result) => {
                        info!(
                            familiar = %advisor_name,
                            topic = %tid,
                            cost = result.total_cost_usd,
                            "chamber response received"
                        );
                        Some((advisor_name, result.result_text))
                    }
                    Err(e) => {
                        warn!(
                            familiar = %advisor_name,
                            topic = %tid,
                            error = %e,
                            "chamber advisor failed"
                        );
                        None
                    }
                }
            });

            handles.push(handle);
        }

        // Collect responses with timeout.
        let mut responses = Vec::new();
        for handle in handles {
            match tokio::time::timeout(std::time::Duration::from_secs(120), handle).await {
                Ok(Ok(Some((name, text)))) => {
                    self.record_response(&topic_id, &name, &text).await;
                    responses.push((name, text));
                }
                Ok(Ok(None)) => {} // advisor failed
                Ok(Err(e)) => warn!(error = %e, "chamber task panicked"),
                Err(_) => warn!("chamber advisor timed out"),
            }
        }

        // Build synthesis prompt with all advisor input.
        let mut council_input = String::from("## Council Debate Responses\n\n");
        for (name, text) in &responses {
            council_input.push_str(&format!("### {} says:\n{}\n\n", name, text));
        }

        let synthesis_context = format!(
            "## Chamber Synthesis\n\nThe council debated this topic:\n\n{}\n\n{}\n\n\
             Synthesize the council's input into a unified recommendation. \
             Attribute key insights to the relevant advisor. \
             Present one clear recommendation.",
            message, council_input
        );

        let lead_executor = ClaudeCodeExecutor::new(
            lead_repo.to_path_buf(),
            lead_model.to_string(),
            15,
            None,
        );

        let synthesis = match lead_executor.execute(lead_identity, &synthesis_context).await {
            Ok(result) => {
                info!(topic = %topic_id, cost = result.total_cost_usd, "chamber synthesis complete");
                result.result_text
            }
            Err(e) => {
                warn!(topic = %topic_id, error = %e, "chamber synthesis failed");
                format!("Council debate collected {} responses but synthesis failed: {}", responses.len(), e)
            }
        };

        self.record_synthesis(&topic_id, &synthesis).await;

        Ok((responses, synthesis))
    }
}

impl Default for Chamber {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chamber_topic_lifecycle() {
        let chamber = Chamber::new();

        let topic_id = chamber.open_topic("test question", vec!["kael".into(), "void".into()]).await;
        assert!(topic_id.starts_with("chamber-"));

        // First response — not all familiars responded yet.
        let all_done = chamber.record_response(&topic_id, "kael", "risk analysis here").await;
        assert!(!all_done);

        // Second response — all familiars responded.
        let all_done = chamber.record_response(&topic_id, "void", "architecture note").await;
        assert!(all_done);

        let topic = chamber.get_topic(&topic_id).await.unwrap();
        assert_eq!(topic.responses.len(), 2);
    }
}
