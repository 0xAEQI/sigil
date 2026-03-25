//! Checkpoint Middleware — periodic state snapshots as a middleware hook.
//!
//! After every N tool calls (configurable via `interval`), records a checkpoint
//! event by storing the current timestamp in [`WorkerContext::metadata`] under
//! the key `"last_checkpoint"`. This provides a lightweight persistence signal
//! that external systems can use to trigger full state serialization.

use async_trait::async_trait;
use tracing::info;

use super::{Middleware, MiddlewareAction, ToolCall, ToolResult, WorkerContext};

/// Checkpoint middleware configuration.
pub struct CheckpointMiddleware {
    /// Checkpoint every N tool calls. Default: 3.
    pub interval: usize,
}

impl CheckpointMiddleware {
    /// Create with default configuration (interval = 3).
    pub fn new() -> Self {
        Self { interval: 3 }
    }

    /// Create with a custom checkpoint interval.
    pub fn with_interval(interval: usize) -> Self {
        Self { interval }
    }

    /// Generate a timestamp string for the checkpoint.
    fn current_timestamp() -> String {
        // Use a simple monotonic representation. In production this would use
        // chrono or std::time::SystemTime, but for deterministic testing we
        // format as epoch-millis-equivalent string.
        use std::time::{SystemTime, UNIX_EPOCH};
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        millis.to_string()
    }
}

impl Default for CheckpointMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CheckpointMiddleware {
    fn name(&self) -> &str {
        "checkpoint"
    }

    fn order(&self) -> u32 {
        90
    }

    async fn after_tool(
        &self,
        ctx: &mut WorkerContext,
        _call: &ToolCall,
        _result: &ToolResult,
    ) -> MiddlewareAction {
        let call_count = ctx.tool_call_history.len();

        // No checkpoint at zero or between intervals.
        if call_count == 0 || !call_count.is_multiple_of(self.interval) {
            return MiddlewareAction::Continue;
        }

        let timestamp = Self::current_timestamp();

        info!(
            task_id = %ctx.task_id,
            call_count,
            interval = self.interval,
            timestamp = %timestamp,
            "checkpoint recorded"
        );

        ctx.metadata
            .insert("last_checkpoint".into(), timestamp);

        MiddlewareAction::Continue
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> WorkerContext {
        WorkerContext::new("task-1", "test task", "engineer", "sigil")
    }

    fn make_call(name: &str) -> ToolCall {
        ToolCall {
            name: name.into(),
            input: "test input".into(),
        }
    }

    fn make_result() -> ToolResult {
        ToolResult {
            success: true,
            output: "ok".into(),
        }
    }

    #[tokio::test]
    async fn checkpoint_at_interval() {
        let mw = CheckpointMiddleware::with_interval(3);
        let mut ctx = test_ctx();
        let call = make_call("Bash");
        let result = make_result();

        // Simulate 3 tool calls in history.
        for _ in 0..3 {
            ctx.tool_call_history.push(make_call("Bash"));
        }

        let action = mw.after_tool(&mut ctx, &call, &result).await;
        assert!(matches!(action, MiddlewareAction::Continue));

        assert!(
            ctx.metadata.contains_key("last_checkpoint"),
            "last_checkpoint should be set at interval"
        );
        // Timestamp should be a numeric string (epoch millis).
        let ts = &ctx.metadata["last_checkpoint"];
        assert!(
            ts.parse::<u128>().is_ok(),
            "timestamp should be numeric, got: {ts}"
        );
    }

    #[tokio::test]
    async fn no_checkpoint_between_intervals() {
        let mw = CheckpointMiddleware::with_interval(3);
        let mut ctx = test_ctx();
        let call = make_call("Bash");
        let result = make_result();

        // Test counts 1, 2, 4, 5 — none should trigger checkpoint.
        for count in [1, 2, 4, 5] {
            ctx.tool_call_history.clear();
            ctx.metadata.remove("last_checkpoint");
            for _ in 0..count {
                ctx.tool_call_history.push(make_call("Bash"));
            }
            let action = mw.after_tool(&mut ctx, &call, &result).await;
            assert!(matches!(action, MiddlewareAction::Continue));
            assert!(
                !ctx.metadata.contains_key("last_checkpoint"),
                "no checkpoint expected at count {count}"
            );
        }
    }

    #[tokio::test]
    async fn checkpoint_at_multiple_intervals() {
        let mw = CheckpointMiddleware::with_interval(3);
        let mut ctx = test_ctx();
        let call = make_call("Bash");
        let result = make_result();

        // Simulate 6 calls — checkpoint at 3 and 6.
        for _ in 0..6 {
            ctx.tool_call_history.push(make_call("Bash"));
        }

        let action = mw.after_tool(&mut ctx, &call, &result).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert!(ctx.metadata.contains_key("last_checkpoint"));
    }

    #[tokio::test]
    async fn empty_history_no_checkpoint() {
        let mw = CheckpointMiddleware::new();
        let mut ctx = test_ctx();
        let call = make_call("Bash");
        let result = make_result();

        let action = mw.after_tool(&mut ctx, &call, &result).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert!(!ctx.metadata.contains_key("last_checkpoint"));
    }

    #[tokio::test]
    async fn order_is_90() {
        let mw = CheckpointMiddleware::new();
        assert_eq!(mw.order(), 90);
    }

    #[tokio::test]
    async fn name_is_correct() {
        let mw = CheckpointMiddleware::new();
        assert_eq!(mw.name(), "checkpoint");
    }

    #[test]
    fn default_interval_is_3() {
        let mw = CheckpointMiddleware::default();
        assert_eq!(mw.interval, 3);
    }
}
