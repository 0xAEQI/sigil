//! Planning Gate Middleware — requires workers to outline their approach before executing.
//!
//! Injects a planning prompt at the start of execution so the worker explicitly
//! states its approach in 3-5 steps and defines a verifiable DONE condition.
//! Can be disabled via the `require_plan` flag for simple or follow-up tasks.

use async_trait::async_trait;
use tracing::debug;

use super::{Middleware, MiddlewareAction, WorkerContext};

/// Planning gate middleware configuration.
pub struct PlanningGateMiddleware {
    /// Whether to require a plan. When false, the middleware is a no-op.
    pub require_plan: bool,
}

impl PlanningGateMiddleware {
    /// Create with default configuration (require_plan = true).
    pub fn new() -> Self {
        Self { require_plan: true }
    }

    /// Create with explicit require_plan setting.
    pub fn with_require_plan(require_plan: bool) -> Self {
        Self { require_plan }
    }
}

impl Default for PlanningGateMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for PlanningGateMiddleware {
    fn name(&self) -> &str {
        "planning_gate"
    }

    fn order(&self) -> u32 {
        10
    }

    async fn on_start(&self, ctx: &mut WorkerContext) -> MiddlewareAction {
        if !self.require_plan {
            debug!(task_id = %ctx.task_id, "planning gate skipped (require_plan=false)");
            return MiddlewareAction::Continue;
        }

        debug!(task_id = %ctx.task_id, "planning gate injecting approach prompt");

        ctx.messages.push(
            "[Planning Gate] Before executing, outline your approach in 3-5 steps. \
             State the DONE condition as a verifiable assertion."
                .to_string(),
        );

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
        WorkerContext::new("task-1", "implement feature X", "engineer", "sigil")
    }

    #[tokio::test]
    async fn injects_planning_message() {
        let mw = PlanningGateMiddleware::new();
        let mut ctx = test_ctx();

        let action = mw.on_start(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert_eq!(ctx.messages.len(), 1);
        assert!(ctx.messages[0].contains("[Planning Gate]"));
        assert!(ctx.messages[0].contains("3-5 steps"));
        assert!(ctx.messages[0].contains("DONE condition"));
    }

    #[tokio::test]
    async fn skipped_when_require_plan_false() {
        let mw = PlanningGateMiddleware::with_require_plan(false);
        let mut ctx = test_ctx();

        let action = mw.on_start(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert!(ctx.messages.is_empty(), "no message should be injected");
    }

    #[tokio::test]
    async fn order_is_10() {
        let mw = PlanningGateMiddleware::new();
        assert_eq!(mw.order(), 10);
    }

    #[tokio::test]
    async fn name_is_correct() {
        let mw = PlanningGateMiddleware::new();
        assert_eq!(mw.name(), "planning_gate");
    }

    #[test]
    fn default_requires_plan() {
        let mw = PlanningGateMiddleware::default();
        assert!(mw.require_plan);
    }
}
