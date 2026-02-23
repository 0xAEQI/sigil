//! Familiar Router — classifies incoming messages to determine which council
//! advisors should be consulted alongside the lead familiar.
//!
//! Uses a cheap Gemini Flash call (~$0.001, ~100ms) to classify message intent,
//! then maps to relevant advisor familiars.

use anyhow::Result;
use realm_core::config::{FamiliarConfig, FamiliarRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

/// Classification result from the router.
#[derive(Debug, Clone)]
pub struct RouteDecision {
    /// Names of advisor familiars to invoke (empty = lead-only).
    pub advisors: Vec<String>,
    /// Classification category for logging.
    pub category: String,
    /// Time taken for classification.
    pub classify_ms: u64,
}

/// Tracks cooldowns to prevent re-invoking the same advisor too quickly.
pub struct FamiliarRouter {
    /// OpenRouter API key for cheap classifier calls.
    api_key: String,
    /// Shared HTTP client (reuses connection pool across calls).
    client: reqwest::Client,
    /// Map of familiar name → last invocation time.
    last_invoked: HashMap<String, Instant>,
    /// Cooldown in seconds before same advisor can be re-invoked.
    cooldown_secs: u64,
}

/// The classifier's JSON output.
#[derive(Debug, Deserialize, Serialize)]
struct ClassifierOutput {
    category: String,
    advisors: Vec<String>,
}

impl FamiliarRouter {
    pub fn new(api_key: String, cooldown_secs: u64) -> Self {
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("failed to build reqwest client"),
            last_invoked: HashMap::new(),
            cooldown_secs,
        }
    }

    /// Classify a message and determine which advisors to invoke.
    ///
    /// Returns a `RouteDecision` with the list of advisor names that should
    /// be consulted for this message, filtered by cooldown and availability.
    pub async fn classify(
        &mut self,
        message: &str,
        available_advisors: &[&FamiliarConfig],
    ) -> Result<RouteDecision> {
        let start = Instant::now();

        // Build advisor descriptions for the classifier.
        let advisor_descriptions: Vec<String> = available_advisors
            .iter()
            .map(|a| {
                let domains = if a.domains.is_empty() {
                    "general".to_string()
                } else {
                    a.domains.join(", ")
                };
                format!("- {}: domains=[{}]", a.name, domains)
            })
            .collect();

        let system_prompt = format!(
            r#"You are a message classifier for an AI council system. Given a user message, determine which specialist advisors should be consulted.

Available advisors:
{}

Classification rules:
- "casual": Simple chat, greetings, personal talk → no advisors needed
- "financial": Trading, fees, costs, PnL, risk analysis → kael
- "product": UX, features, marketplace, engagement, branding → mira
- "technical": Architecture, code, performance, bugs, infrastructure → void
- "strategic": Major decisions spanning multiple concerns → all relevant advisors

Respond with ONLY a JSON object (no markdown, no code fences):
{{"category": "<category>", "advisors": ["<name1>", "<name2>"]}}

Use empty array for "casual" messages. Only include advisors whose expertise is relevant."#,
            advisor_descriptions.join("\n")
        );

        let body = serde_json::json!({
            "model": "google/gemini-2.0-flash-001",
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": message}
            ],
            "max_tokens": 80,
            "temperature": 0.0
        });

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        let classify_ms = start.elapsed().as_millis() as u64;

        let decision = match response {
            Ok(resp) => {
                match resp.json::<serde_json::Value>().await {
                    Ok(v) => {
                        let text = v
                            .pointer("/choices/0/message/content")
                            .and_then(|c| c.as_str())
                            .unwrap_or("")
                            .trim();

                        // Strip markdown code fences if present.
                        let clean = text
                            .strip_prefix("```json")
                            .or_else(|| text.strip_prefix("```"))
                            .unwrap_or(text)
                            .strip_suffix("```")
                            .unwrap_or(text)
                            .trim();

                        match serde_json::from_str::<ClassifierOutput>(clean) {
                            Ok(parsed) => {
                                info!(
                                    category = %parsed.category,
                                    advisors = ?parsed.advisors,
                                    ms = classify_ms,
                                    "message classified"
                                );
                                parsed
                            }
                            Err(e) => {
                                warn!(error = %e, raw = %text, "classifier parse failed, defaulting to casual");
                                ClassifierOutput {
                                    category: "casual".to_string(),
                                    advisors: Vec::new(),
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "classifier response parse failed");
                        ClassifierOutput {
                            category: "casual".to_string(),
                            advisors: Vec::new(),
                        }
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "classifier request failed, defaulting to lead-only");
                ClassifierOutput {
                    category: "casual".to_string(),
                    advisors: Vec::new(),
                }
            }
        };

        // Filter by cooldown.
        let now = Instant::now();
        let valid_advisor_names: Vec<String> = available_advisors
            .iter()
            .filter(|a| a.role == FamiliarRole::Advisor)
            .map(|a| a.name.clone())
            .collect();

        let advisors: Vec<String> = decision
            .advisors
            .into_iter()
            .filter(|name| {
                // Must be a known advisor.
                if !valid_advisor_names.contains(name) {
                    return false;
                }
                // Check cooldown.
                if let Some(last) = self.last_invoked.get(name)
                    && now.duration_since(*last).as_secs() < self.cooldown_secs {
                        info!(familiar = %name, "advisor on cooldown, skipping");
                        return false;
                    }
                true
            })
            .collect();

        // Update last-invoked timestamps.
        for name in &advisors {
            self.last_invoked.insert(name.clone(), now);
        }

        Ok(RouteDecision {
            advisors,
            category: decision.category,
            classify_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classifier_output_parse() {
        let json = r#"{"category": "financial", "advisors": ["kael"]}"#;
        let parsed: ClassifierOutput = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.category, "financial");
        assert_eq!(parsed.advisors, vec!["kael"]);
    }

    #[test]
    fn test_empty_advisors() {
        let json = r#"{"category": "casual", "advisors": []}"#;
        let parsed: ClassifierOutput = serde_json::from_str(json).unwrap();
        assert!(parsed.advisors.is_empty());
    }
}
