//! Unified delegate tool — consolidates subagent spawning, dispatch sending,
//! task assignment, and channel posting into a single `delegate` tool with
//! routing determined by the `to` parameter.
//!
//! Response modes:
//! - `origin` — response injected back into the caller's conversation
//! - `perpetual` — response delivered to the caller's perpetual session
//! - `async` — fire-and-forget; caller notified on completion
//! - `department` — response posted to the department channel
//! - `none` — no response expected

use anyhow::Result;
use async_trait::async_trait;
use sigil_core::traits::{Tool, ToolResult, ToolSpec};
use std::sync::Arc;
use tracing::info;

use crate::agent_registry::AgentRegistry;
use crate::message::{Dispatch, DispatchBus, DispatchKind};

// ---------------------------------------------------------------------------
// UnifiedDelegateTool
// ---------------------------------------------------------------------------

/// Unified tool for delegating work to subagents, named agents, or departments.
///
/// Routing is determined by the `to` parameter:
/// - `"subagent"` — delegate to the project-default agent (ephemeral worker)
/// - `"dept:<name>"` — post to a department conversation channel
/// - `<agent_name>` — send a DelegateRequest dispatch to a named agent
pub struct UnifiedDelegateTool {
    dispatch_bus: Arc<DispatchBus>,
    /// The name of the calling agent (used as the "from" field in dispatches).
    agent_name: String,
    /// Optional agent registry for looking up project-default agents.
    agent_registry: Option<Arc<AgentRegistry>>,
    /// Project name for scoping default-agent lookups.
    project_name: Option<String>,
    /// Fallback target when no project-default agent is found (system escalation target).
    fallback_target: Option<String>,
}

impl UnifiedDelegateTool {
    pub fn new(dispatch_bus: Arc<DispatchBus>, agent_name: String) -> Self {
        Self {
            dispatch_bus,
            agent_name,
            agent_registry: None,
            project_name: None,
            fallback_target: None,
        }
    }

    /// Set the agent registry and project context for subagent routing.
    pub fn with_agent_context(
        mut self,
        registry: Arc<AgentRegistry>,
        project_name: Option<String>,
        fallback_target: String,
    ) -> Self {
        self.agent_registry = Some(registry);
        self.project_name = project_name;
        self.fallback_target = Some(fallback_target);
        self
    }

    /// Parse a response mode string, defaulting to "origin".
    fn parse_response_mode(args: &serde_json::Value) -> String {
        args.get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("origin")
            .to_string()
    }

    /// Handle delegation to a named agent via DelegateRequest dispatch.
    async fn delegate_to_agent(
        &self,
        to: &str,
        prompt: &str,
        response_mode: &str,
        create_task: bool,
        skill: Option<String>,
    ) -> Result<ToolResult> {
        let kind = DispatchKind::DelegateRequest {
            prompt: prompt.to_string(),
            response_mode: response_mode.to_string(),
            create_task,
            skill: skill.clone(),
            reply_to: None,
        };

        let dispatch = Dispatch::new_typed(&self.agent_name, to, kind);
        let dispatch_id = dispatch.id.clone();

        info!(
            from = %self.agent_name,
            to = %to,
            response_mode = %response_mode,
            create_task = create_task,
            dispatch_id = %dispatch_id,
            "sending DelegateRequest dispatch"
        );

        self.dispatch_bus.send(dispatch).await;

        let mut msg = format!(
            "Delegation sent to '{to}' (dispatch_id: {dispatch_id}, response_mode: {response_mode})"
        );
        if create_task {
            msg.push_str("\nTask creation requested — target agent will pick up via task queue.");
        }
        if let Some(s) = &skill {
            msg.push_str(&format!("\nSkill hint: {s}"));
        }

        Ok(ToolResult::success(msg))
    }

    /// Resolve the target agent for subagent delegation.
    ///
    /// Tries the agent registry's project-default first, then falls back
    /// to the configured system escalation target.
    async fn resolve_subagent_target(&self) -> Option<String> {
        // Try agent registry lookup first.
        if let Some(ref registry) = self.agent_registry
            && let Ok(Some(agent)) = registry
                .default_for_project(self.project_name.as_deref())
                .await
        {
            info!(
                project = ?self.project_name,
                agent = %agent.name,
                "resolved project-default agent for subagent dispatch"
            );
            return Some(agent.name);
        }

        // Fall back to system escalation target.
        self.fallback_target.clone()
    }

    /// Handle delegation to a department channel.
    async fn delegate_to_department(
        &self,
        dept: &str,
        prompt: &str,
        response_mode: &str,
    ) -> Result<ToolResult> {
        // Send a DelegateRequest dispatch addressed to the department.
        // The trigger/routing system will pick it up and deliver to appropriate agents.
        let kind = DispatchKind::DelegateRequest {
            prompt: prompt.to_string(),
            response_mode: response_mode.to_string(),
            create_task: false,
            skill: None,
            reply_to: None,
        };

        let to = format!("dept:{dept}");
        let dispatch = Dispatch::new_typed(&self.agent_name, &to, kind);
        let dispatch_id = dispatch.id.clone();

        info!(
            from = %self.agent_name,
            department = %dept,
            dispatch_id = %dispatch_id,
            "sending DelegateRequest to department"
        );

        self.dispatch_bus.send(dispatch).await;

        Ok(ToolResult::success(format!(
            "Delegation posted to department '{dept}' (dispatch_id: {dispatch_id}, response_mode: {response_mode})"
        )))
    }
}

#[async_trait]
impl Tool for UnifiedDelegateTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required parameter 'to'"))?;
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required parameter 'prompt'"))?;

        let response_mode = Self::parse_response_mode(&args);
        let create_task = args
            .get("create_task")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let skill = args.get("skill").and_then(|v| v.as_str()).map(String::from);

        match to {
            // Pattern 1: Subagent — delegate to project-default agent
            "subagent" => {
                // Resolve the target agent: try project-default via agent registry,
                // then fall back to the system escalation target.
                let target = self.resolve_subagent_target().await;
                let target = match target {
                    Some(name) => name,
                    None => {
                        return Ok(ToolResult::error(
                            "No target agent available for subagent delegation. \
                             Configure a project-default agent or system escalation target.",
                        ));
                    }
                };

                info!(
                    from = %self.agent_name,
                    resolved_target = %target,
                    "subagent request routed to project-default agent"
                );

                // Subagent dispatches always use origin response mode and create a task.
                self.delegate_to_agent(&target, prompt, "origin", true, skill)
                    .await
            }

            // Pattern 3: Department — post to department channel
            dept_target if dept_target.starts_with("dept:") => {
                let dept_name = &dept_target[5..]; // strip "dept:" prefix
                if dept_name.is_empty() {
                    return Ok(ToolResult::error(
                        "Department name cannot be empty. Use 'dept:<name>' format.",
                    ));
                }
                self.delegate_to_department(dept_name, prompt, &response_mode)
                    .await
            }

            // Pattern 2 & 4: Named agent (or fallback for unknown targets)
            agent_name => {
                self.delegate_to_agent(agent_name, prompt, &response_mode, create_task, skill)
                    .await
            }
        }
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "unified_delegate".to_string(),
            description: "Delegate work to subagents, named agents, or departments. \
                Routes based on the 'to' parameter: \
                'subagent' spawns an ephemeral sub-agent, \
                'dept:<name>' posts to a department channel, \
                or any other value sends a delegation request to a named agent. \
                Response mode controls how results are returned: \
                'origin' (inject back into caller), \
                'perpetual' (deliver to perpetual session), \
                'async' (fire-and-forget with notification), \
                'department' (post to department channel), \
                'none' (no response expected)."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Target: 'subagent' for ephemeral agent, 'dept:<name>' for department, or an agent name"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "The task or message to delegate"
                    },
                    "response": {
                        "type": "string",
                        "enum": ["origin", "perpetual", "async", "department", "none"],
                        "default": "origin",
                        "description": "How the response should be routed back"
                    },
                    "create_task": {
                        "type": "boolean",
                        "default": false,
                        "description": "Whether to also create a tracked task for this delegation"
                    },
                    "skill": {
                        "type": "string",
                        "description": "Optional skill hint for the target agent"
                    },
                    "tools": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional tool allowlist for subagent mode"
                    }
                },
                "required": ["to", "prompt"]
            }),
        }
    }

    fn name(&self) -> &str {
        "unified_delegate"
    }

    fn is_concurrent_safe(&self, _input: &serde_json::Value) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool() -> UnifiedDelegateTool {
        let bus = Arc::new(DispatchBus::new());
        UnifiedDelegateTool::new(bus, "test-agent".to_string())
    }

    #[test]
    fn test_parse_response_mode_default() {
        let args = serde_json::json!({});
        assert_eq!(UnifiedDelegateTool::parse_response_mode(&args), "origin");
    }

    #[test]
    fn test_parse_response_mode_explicit() {
        let args = serde_json::json!({"response": "async"});
        assert_eq!(UnifiedDelegateTool::parse_response_mode(&args), "async");
    }

    #[test]
    fn test_spec_has_required_fields() {
        let tool = make_tool();
        let spec = tool.spec();
        assert_eq!(spec.name, "unified_delegate");
        let required = spec.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("to")));
        assert!(required.contains(&serde_json::json!("prompt")));
    }

    #[test]
    fn test_name() {
        let tool = make_tool();
        assert_eq!(tool.name(), "unified_delegate");
    }

    #[tokio::test]
    async fn test_subagent_no_target_returns_error() {
        // Without agent_registry or fallback_target, subagent should error.
        let tool = make_tool();
        let args = serde_json::json!({
            "to": "subagent",
            "prompt": "do something"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.is_error);
        assert!(result.output.contains("No target agent available"));
    }

    #[tokio::test]
    async fn test_subagent_with_fallback_target() {
        let bus = Arc::new(DispatchBus::new());
        let mut tool = UnifiedDelegateTool::new(bus.clone(), "caller".to_string());
        tool.fallback_target = Some("leader".to_string());

        let args = serde_json::json!({
            "to": "subagent",
            "prompt": "handle this task",
            "skill": "code-review"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("leader"));
        assert!(result.output.contains("dispatch_id"));
        assert!(result.output.contains("Task creation requested"));
        assert!(result.output.contains("code-review"));

        // Verify the dispatch was sent to the fallback target.
        let messages = bus.read("leader").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "caller");
        assert_eq!(messages[0].to, "leader");
        assert_eq!(messages[0].kind.subject_tag(), "DELEGATE_REQUEST");
    }

    #[tokio::test]
    async fn test_department_mode_detection() {
        let tool = make_tool();
        let args = serde_json::json!({
            "to": "dept:engineering",
            "prompt": "review this PR"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("engineering"));
        assert!(result.output.contains("dispatch_id"));
    }

    #[tokio::test]
    async fn test_department_empty_name_rejected() {
        let tool = make_tool();
        let args = serde_json::json!({
            "to": "dept:",
            "prompt": "review this PR"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.is_error);
        assert!(result.output.contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_named_agent_dispatch() {
        let tool = make_tool();
        let args = serde_json::json!({
            "to": "researcher",
            "prompt": "find the auth bug",
            "response": "async",
            "create_task": true,
            "skill": "code-review"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("researcher"));
        assert!(result.output.contains("dispatch_id"));
        assert!(result.output.contains("Task creation requested"));
        assert!(result.output.contains("code-review"));
    }

    #[tokio::test]
    async fn test_missing_to_param() {
        let tool = make_tool();
        let args = serde_json::json!({
            "prompt": "do something"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_prompt_param() {
        let tool = make_tool();
        let args = serde_json::json!({
            "to": "researcher"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dispatch_actually_sent() {
        let bus = Arc::new(DispatchBus::new());
        let tool = UnifiedDelegateTool::new(bus.clone(), "sender".to_string());

        let args = serde_json::json!({
            "to": "receiver",
            "prompt": "hello agent"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(!result.is_error);

        // Verify the dispatch landed in the bus.
        let messages = bus.read("receiver").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "sender");
        assert_eq!(messages[0].to, "receiver");
        assert_eq!(messages[0].kind.subject_tag(), "DELEGATE_REQUEST");
    }

    #[tokio::test]
    async fn test_department_dispatch_sent() {
        let bus = Arc::new(DispatchBus::new());
        let tool = UnifiedDelegateTool::new(bus.clone(), "leader".to_string());

        let args = serde_json::json!({
            "to": "dept:ops",
            "prompt": "check server health"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(!result.is_error);

        // Verify dispatch was sent to "dept:ops".
        let messages = bus.read("dept:ops").await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "leader");
        assert_eq!(messages[0].kind.subject_tag(), "DELEGATE_REQUEST");
    }
}
