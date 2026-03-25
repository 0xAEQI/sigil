//! Dangling Tool Patch Middleware — detects interrupted tool calls and injects
//! synthetic error responses.
//!
//! When a tool call is recorded in the worker context but its execution was
//! interrupted (result is empty or missing), this middleware injects a message
//! informing the model that the call failed. This prevents the model from
//! assuming a tool succeeded when it actually never completed.
//!
//! Tracks processed tool calls via a counter in [`WorkerContext::metadata`]
//! under the key `"dangling_patch_processed"`. On each `before_model` call,
//! only newly added tool calls since the last check are inspected.

use std::sync::Mutex;

use async_trait::async_trait;
use tracing::{debug, warn};

use super::{Middleware, MiddlewareAction, WorkerContext};

/// Dangling tool patch middleware — detect interrupted tool calls.
pub struct DanglingToolPatchMiddleware {
    /// Track the number of tool calls we have already inspected.
    processed_count: Mutex<usize>,
}

impl DanglingToolPatchMiddleware {
    /// Create a new dangling tool patch middleware.
    pub fn new() -> Self {
        Self {
            processed_count: Mutex::new(0),
        }
    }
}

impl Default for DanglingToolPatchMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for DanglingToolPatchMiddleware {
    fn name(&self) -> &str {
        "dangling_tool_patch"
    }

    fn order(&self) -> u32 {
        8
    }

    async fn before_model(&self, ctx: &mut WorkerContext) -> MiddlewareAction {
        let mut processed = self
            .processed_count
            .lock()
            .expect("dangling tool patch lock poisoned");

        let history_len = ctx.tool_call_history.len();

        // Nothing new to inspect.
        if *processed >= history_len {
            return MiddlewareAction::Continue;
        }

        let mut patches = Vec::new();

        // Check newly added tool calls for empty/missing input (indicating interruption).
        // A tool call with an empty input string is treated as dangling — normal calls
        // always carry serialized parameters.
        for call in &ctx.tool_call_history[*processed..] {
            if call.input.is_empty() {
                warn!(
                    task_id = %ctx.task_id,
                    tool = %call.name,
                    "dangling tool call detected — injecting synthetic error"
                );
                patches.push(format!(
                    "[DanglingToolPatch] Tool call '{}' was interrupted. \
                     Treating as error: execution interrupted.",
                    call.name
                ));
            }
        }

        // Mark all current history as processed.
        *processed = history_len;

        if patches.is_empty() {
            debug!(task_id = %ctx.task_id, "no dangling tool calls found");
            MiddlewareAction::Continue
        } else {
            debug!(
                task_id = %ctx.task_id,
                count = patches.len(),
                "injecting dangling tool patches"
            );
            MiddlewareAction::Inject(patches)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::ToolCall;

    fn test_ctx() -> WorkerContext {
        WorkerContext::new("task-1", "test task", "engineer", "sigil")
    }

    fn make_call(name: &str, input: &str) -> ToolCall {
        ToolCall {
            name: name.into(),
            input: input.into(),
        }
    }

    #[tokio::test]
    async fn patches_dangling_call() {
        let mw = DanglingToolPatchMiddleware::new();
        let mut ctx = test_ctx();

        // A dangling call has empty input (interrupted before parameters were set).
        ctx.tool_call_history.push(make_call("Bash", ""));

        let action = mw.before_model(&mut ctx).await;
        match action {
            MiddlewareAction::Inject(msgs) => {
                assert_eq!(msgs.len(), 1);
                assert!(msgs[0].contains("[DanglingToolPatch]"));
                assert!(msgs[0].contains("Bash"));
                assert!(msgs[0].contains("interrupted"));
            }
            other => panic!("expected Inject, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn no_op_for_clean_history() {
        let mw = DanglingToolPatchMiddleware::new();
        let mut ctx = test_ctx();

        // Normal call with input present.
        ctx.tool_call_history
            .push(make_call("Read", "/some/file.rs"));

        let action = mw.before_model(&mut ctx).await;
        assert!(
            matches!(action, MiddlewareAction::Continue),
            "expected Continue for clean history, got {action:?}"
        );
    }

    #[tokio::test]
    async fn no_op_for_empty_history() {
        let mw = DanglingToolPatchMiddleware::new();
        let mut ctx = test_ctx();

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
    }

    #[tokio::test]
    async fn processes_only_new_calls() {
        let mw = DanglingToolPatchMiddleware::new();
        let mut ctx = test_ctx();

        // First batch: one clean call.
        ctx.tool_call_history
            .push(make_call("Read", "/some/file.rs"));
        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));

        // Second batch: add a dangling call.
        ctx.tool_call_history.push(make_call("Edit", ""));
        let action = mw.before_model(&mut ctx).await;
        match action {
            MiddlewareAction::Inject(msgs) => {
                assert_eq!(msgs.len(), 1);
                assert!(msgs[0].contains("Edit"));
            }
            other => panic!("expected Inject for new dangling call, got {other:?}"),
        }

        // Third call: nothing new — should be no-op.
        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
    }

    #[tokio::test]
    async fn multiple_dangling_calls() {
        let mw = DanglingToolPatchMiddleware::new();
        let mut ctx = test_ctx();

        ctx.tool_call_history.push(make_call("Bash", ""));
        ctx.tool_call_history.push(make_call("Edit", ""));
        ctx.tool_call_history
            .push(make_call("Read", "/valid/input"));

        let action = mw.before_model(&mut ctx).await;
        match action {
            MiddlewareAction::Inject(msgs) => {
                assert_eq!(msgs.len(), 2);
                assert!(msgs[0].contains("Bash"));
                assert!(msgs[1].contains("Edit"));
            }
            other => panic!("expected Inject with 2 patches, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn order_is_8() {
        let mw = DanglingToolPatchMiddleware::new();
        assert_eq!(mw.order(), 8);
    }

    #[tokio::test]
    async fn name_is_correct() {
        let mw = DanglingToolPatchMiddleware::new();
        assert_eq!(mw.name(), "dangling_tool_patch");
    }

    #[test]
    fn default_impl() {
        let _mw = DanglingToolPatchMiddleware::default();
    }
}
