use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::identity::Identity;
use crate::traits::{
    ChatRequest, ChatResponse, ContentPart, Event, Message, MessageContent, Observer, Provider,
    Role, StopReason, Tool, ToolResult, ToolSpec,
};

/// Configuration for an agent loop.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Model to use (provider-specific format).
    pub model: String,
    /// Maximum iterations (LLM round-trips) before stopping.
    pub max_iterations: u32,
    /// Maximum tokens per LLM response.
    pub max_tokens: u32,
    /// Temperature for generation.
    pub temperature: f32,
    /// Name of this agent (for logging).
    pub name: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "anthropic/claude-sonnet-4.6".to_string(),
            max_iterations: 20,
            max_tokens: 4096,
            temperature: 0.0,
            name: "agent".to_string(),
        }
    }
}

/// The agent: a thin loop that sends prompts to an LLM, parses tool calls,
/// executes tools, and repeats until done. Zero Framework Cognition — no
/// heuristics, all decisions delegated to the LLM.
pub struct Agent {
    config: AgentConfig,
    provider: Arc<dyn Provider>,
    tools: Vec<Arc<dyn Tool>>,
    observer: Arc<dyn Observer>,
    identity: Identity,
}

impl Agent {
    pub fn new(
        config: AgentConfig,
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn Tool>>,
        observer: Arc<dyn Observer>,
        identity: Identity,
    ) -> Self {
        Self {
            config,
            provider,
            tools,
            observer,
            identity,
        }
    }

    /// Run the agent with a user prompt. Returns the final text response.
    pub async fn run(&self, prompt: &str) -> anyhow::Result<String> {
        self.observer
            .record(Event::AgentStart {
                agent_name: self.config.name.clone(),
            })
            .await;

        // Build initial messages.
        let system_prompt = self.identity.system_prompt();
        let mut messages = vec![
            Message {
                role: Role::System,
                content: MessageContent::text(&system_prompt),
            },
            Message {
                role: Role::User,
                content: MessageContent::text(prompt),
            },
        ];

        // Collect tool specs.
        let tool_specs: Vec<ToolSpec> = self.tools.iter().map(|t| t.spec()).collect();

        let mut iterations = 0u32;
        let mut final_text = String::new();

        loop {
            iterations += 1;
            if iterations > self.config.max_iterations {
                warn!(
                    agent = %self.config.name,
                    max = self.config.max_iterations,
                    "agent hit max iterations"
                );
                break;
            }

            // Build request.
            let request = ChatRequest {
                model: self.config.model.clone(),
                messages: messages.clone(),
                tools: tool_specs.clone(),
                max_tokens: self.config.max_tokens,
                temperature: self.config.temperature,
            };

            self.observer
                .record(Event::LlmRequest {
                    model: self.config.model.clone(),
                    tokens: 0, // We don't know prompt tokens until response.
                })
                .await;

            // Call provider.
            let response: ChatResponse = self.provider.chat(&request).await?;

            self.observer
                .record(Event::LlmResponse {
                    model: self.config.model.clone(),
                    prompt_tokens: response.usage.prompt_tokens,
                    completion_tokens: response.usage.completion_tokens,
                })
                .await;

            debug!(
                agent = %self.config.name,
                iteration = iterations,
                tool_calls = response.tool_calls.len(),
                stop_reason = ?response.stop_reason,
                "LLM response received"
            );

            // If there's text content, accumulate it.
            if let Some(ref text) = response.content {
                final_text = text.clone();
            }

            // If no tool calls, we're done.
            if response.tool_calls.is_empty() {
                break;
            }

            // Build assistant message with tool use parts.
            let mut assistant_parts: Vec<ContentPart> = Vec::new();
            if let Some(ref text) = response.content {
                assistant_parts.push(ContentPart::Text {
                    text: text.clone(),
                });
            }
            for tc in &response.tool_calls {
                assistant_parts.push(ContentPart::ToolUse {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    input: tc.arguments.clone(),
                });
            }

            messages.push(Message {
                role: Role::Assistant,
                content: MessageContent::Parts(assistant_parts),
            });

            // Execute each tool call and build tool result messages.
            let mut tool_result_parts: Vec<ContentPart> = Vec::new();
            for tc in &response.tool_calls {
                let start = std::time::Instant::now();

                let result = self.execute_tool(&tc.name, tc.arguments.clone()).await;
                let duration_ms = start.elapsed().as_millis() as u64;

                match &result {
                    Ok(tr) => {
                        self.observer
                            .record(Event::ToolCall {
                                tool_name: tc.name.clone(),
                                duration_ms,
                            })
                            .await;

                        tool_result_parts.push(ContentPart::ToolResult {
                            tool_use_id: tc.id.clone(),
                            content: tr.output.clone(),
                            is_error: tr.is_error,
                        });
                    }
                    Err(e) => {
                        self.observer
                            .record(Event::ToolError {
                                tool_name: tc.name.clone(),
                                error: e.to_string(),
                            })
                            .await;

                        tool_result_parts.push(ContentPart::ToolResult {
                            tool_use_id: tc.id.clone(),
                            content: format!("Tool execution error: {e}"),
                            is_error: true,
                        });
                    }
                }
            }

            messages.push(Message {
                role: Role::Tool,
                content: MessageContent::Parts(tool_result_parts),
            });

            // If stop reason is EndTurn (not ToolUse), break.
            if response.stop_reason == StopReason::EndTurn {
                break;
            }
        }

        self.observer
            .record(Event::AgentEnd {
                agent_name: self.config.name.clone(),
                iterations,
            })
            .await;

        info!(
            agent = %self.config.name,
            iterations,
            "agent completed"
        );

        Ok(final_text)
    }

    /// Find and execute a tool by name.
    async fn execute_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        for tool in &self.tools {
            if tool.name() == name {
                return tool.execute(args).await;
            }
        }
        Ok(ToolResult::error(format!("Unknown tool: {name}")))
    }
}
