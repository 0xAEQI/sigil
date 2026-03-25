//! Message Queue Middleware — injects externally queued messages between tool calls.
//!
//! External callers (e.g. the supervisor, IPC handlers, or the chat engine) can
//! push messages into the queue at any time. Before each model invocation, this
//! middleware drains the queue and injects any pending messages into the worker
//! context via [`MiddlewareAction::Inject`].

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tracing::debug;

use super::{Middleware, MiddlewareAction, WorkerContext};

/// Message queue middleware — check for externally injected messages.
pub struct MessageQueueMiddleware {
    /// Thread-safe queue of pending messages.
    queue: Arc<Mutex<Vec<String>>>,
}

impl MessageQueueMiddleware {
    /// Create a new message queue middleware with an empty queue.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Push a message into the queue. External callers use this to inject
    /// messages that will be delivered before the next model invocation.
    pub fn push_message(&self, msg: String) {
        let mut q = self.queue.lock().expect("message queue lock poisoned");
        q.push(msg);
    }

    /// Returns the number of pending messages in the queue.
    pub fn pending_count(&self) -> usize {
        let q = self.queue.lock().expect("message queue lock poisoned");
        q.len()
    }
}

impl Default for MessageQueueMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for MessageQueueMiddleware {
    fn name(&self) -> &str {
        "message_queue"
    }

    fn order(&self) -> u32 {
        45
    }

    async fn before_model(&self, ctx: &mut WorkerContext) -> MiddlewareAction {
        let messages: Vec<String> = {
            let mut q = self.queue.lock().expect("message queue lock poisoned");
            if q.is_empty() {
                return MiddlewareAction::Continue;
            }
            q.drain(..).collect()
        };

        debug!(
            task_id = %ctx.task_id,
            count = messages.len(),
            "message queue draining pending messages"
        );

        MiddlewareAction::Inject(messages)
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

    #[tokio::test]
    async fn inject_queued_messages() {
        let mw = MessageQueueMiddleware::new();
        let mut ctx = test_ctx();

        mw.push_message("urgent: stop and re-read requirements".into());
        mw.push_message("note: API changed, see /docs/api.md".into());

        let action = mw.before_model(&mut ctx).await;
        match action {
            MiddlewareAction::Inject(msgs) => {
                assert_eq!(msgs.len(), 2);
                assert!(msgs[0].contains("urgent"));
                assert!(msgs[1].contains("API changed"));
            }
            other => panic!("expected Inject, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_queue_continues() {
        let mw = MessageQueueMiddleware::new();
        let mut ctx = test_ctx();

        let action = mw.before_model(&mut ctx).await;
        assert!(
            matches!(action, MiddlewareAction::Continue),
            "expected Continue for empty queue, got {action:?}"
        );
    }

    #[tokio::test]
    async fn queue_is_drained_after_inject() {
        let mw = MessageQueueMiddleware::new();
        let mut ctx = test_ctx();

        mw.push_message("first".into());
        assert_eq!(mw.pending_count(), 1);

        let _ = mw.before_model(&mut ctx).await;
        assert_eq!(mw.pending_count(), 0);

        // Second call should be no-op.
        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
    }

    #[tokio::test]
    async fn order_is_45() {
        let mw = MessageQueueMiddleware::new();
        assert_eq!(mw.order(), 45);
    }

    #[tokio::test]
    async fn name_is_correct() {
        let mw = MessageQueueMiddleware::new();
        assert_eq!(mw.name(), "message_queue");
    }

    #[test]
    fn default_impl() {
        let mw = MessageQueueMiddleware::default();
        assert_eq!(mw.pending_count(), 0);
    }
}
