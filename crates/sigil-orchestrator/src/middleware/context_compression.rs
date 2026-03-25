//! Context Compression Middleware — compresses middle messages when context grows too large.
//!
//! Inspired by Hermes' context_compressor.py. When the messages buffer exceeds a
//! configurable threshold (percentage of a line budget), the middleware replaces
//! the middle portion of messages with a single compressed summary, preserving the
//! first N and last N messages intact.
//!
//! This prevents workers from hitting context limits on long-running tasks while
//! retaining the most relevant context (initial instructions and recent activity).

use async_trait::async_trait;
use tracing::{debug, info};

use super::{Middleware, MiddlewareAction, WorkerContext};

/// Context compression middleware configuration.
pub struct ContextCompressionMiddleware {
    /// Compress when messages count exceeds `threshold_percent * max_context_lines`.
    /// Default: 0.50 (50% of budget).
    threshold_percent: f32,
    /// Maximum context lines used as the budget reference.
    /// Default: 500.
    max_context_lines: usize,
    /// Number of initial messages to always preserve.
    /// Default: 3.
    protect_first_n: usize,
    /// Number of trailing messages to always preserve.
    /// Default: 5.
    protect_last_n: usize,
}

impl ContextCompressionMiddleware {
    /// Create with default configuration.
    pub fn new() -> Self {
        Self {
            threshold_percent: 0.50,
            max_context_lines: 500,
            protect_first_n: 3,
            protect_last_n: 5,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(
        threshold_percent: f32,
        max_context_lines: usize,
        protect_first_n: usize,
        protect_last_n: usize,
    ) -> Self {
        Self {
            threshold_percent,
            max_context_lines,
            protect_first_n,
            protect_last_n,
        }
    }

    /// Compute the message count threshold that triggers compression.
    fn threshold(&self) -> usize {
        (self.max_context_lines as f32 * self.threshold_percent) as usize
    }

    /// Build a compressed summary from a slice of messages.
    ///
    /// Extracts the first 200 characters of each message and joins them into
    /// a single summary string.
    fn build_summary(messages: &[String]) -> String {
        let count = messages.len();
        let key_points: Vec<String> = messages
            .iter()
            .map(|m| {
                let trimmed = m.trim();
                if trimmed.len() <= 200 {
                    trimmed.to_string()
                } else {
                    format!("{}...", &trimmed[..200])
                }
            })
            .collect();

        format!(
            "[Context compressed: {} messages summarized. Key points: {}]",
            count,
            key_points.join(" | ")
        )
    }

    /// Compress the messages buffer in-place, preserving head and tail.
    fn compress_messages(
        messages: &[String],
        protect_first: usize,
        protect_last: usize,
    ) -> Vec<String> {
        let len = messages.len();
        let protected_total = protect_first + protect_last;

        // Not enough messages to have a compressible middle section.
        if len <= protected_total {
            return messages.to_vec();
        }

        let head = &messages[..protect_first];
        let middle = &messages[protect_first..len - protect_last];
        let tail = &messages[len - protect_last..];

        let summary = Self::build_summary(middle);

        let mut result = Vec::with_capacity(protected_total + 1);
        result.extend_from_slice(head);
        result.push(summary);
        result.extend_from_slice(tail);
        result
    }
}

impl Default for ContextCompressionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for ContextCompressionMiddleware {
    fn name(&self) -> &str {
        "context-compression"
    }

    fn order(&self) -> u32 {
        15
    }

    async fn before_model(&self, ctx: &mut WorkerContext) -> MiddlewareAction {
        let msg_count = ctx.messages.len();
        let threshold = self.threshold();

        if msg_count <= threshold {
            return MiddlewareAction::Continue;
        }

        let original_count = msg_count;
        ctx.messages = Self::compress_messages(
            &ctx.messages,
            self.protect_first_n,
            self.protect_last_n,
        );

        let compressed_count = original_count - ctx.messages.len();
        info!(
            task_id = %ctx.task_id,
            original_messages = original_count,
            compressed_messages = compressed_count,
            remaining_messages = ctx.messages.len(),
            threshold,
            "context compressed — middle messages summarized"
        );
        debug!(
            protect_first = self.protect_first_n,
            protect_last = self.protect_last_n,
            "compression boundaries"
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
        WorkerContext::new("task-1", "test task", "engineer", "sigil")
    }

    fn make_messages(count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("Message {i}: some content here"))
            .collect()
    }

    #[tokio::test]
    async fn below_threshold_no_compression() {
        // threshold = 0.50 * 500 = 250; 10 messages < 250
        let mw = ContextCompressionMiddleware::new();
        let mut ctx = test_ctx();
        ctx.messages = make_messages(10);

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert_eq!(ctx.messages.len(), 10);
        // Verify no compression marker present.
        for msg in &ctx.messages {
            assert!(!msg.contains("[Context compressed"));
        }
    }

    #[tokio::test]
    async fn above_threshold_compresses_middle() {
        // threshold_percent=0.50, max_context_lines=20 → threshold=10
        // protect_first=2, protect_last=2
        let mw = ContextCompressionMiddleware::with_config(0.50, 20, 2, 2);
        let mut ctx = test_ctx();
        ctx.messages = make_messages(15); // 15 > 10 → triggers compression

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));

        // Result: 2 head + 1 summary + 2 tail = 5
        assert_eq!(ctx.messages.len(), 5);

        // Head preserved.
        assert_eq!(ctx.messages[0], "Message 0: some content here");
        assert_eq!(ctx.messages[1], "Message 1: some content here");

        // Summary in the middle.
        assert!(ctx.messages[2].contains("[Context compressed: 11 messages summarized"));

        // Tail preserved.
        assert_eq!(ctx.messages[3], "Message 13: some content here");
        assert_eq!(ctx.messages[4], "Message 14: some content here");
    }

    #[tokio::test]
    async fn exact_threshold_no_compression() {
        // threshold_percent=0.50, max_context_lines=20 → threshold=10
        let mw = ContextCompressionMiddleware::with_config(0.50, 20, 2, 2);
        let mut ctx = test_ctx();
        ctx.messages = make_messages(10); // exactly 10 = threshold → no compression

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert_eq!(ctx.messages.len(), 10);
    }

    #[tokio::test]
    async fn empty_messages_no_op() {
        let mw = ContextCompressionMiddleware::new();
        let mut ctx = test_ctx();
        // No messages at all.

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert!(ctx.messages.is_empty());
    }

    #[tokio::test]
    async fn few_messages_no_compression() {
        // protect_first=3, protect_last=5 → need > 8 messages to have a middle
        // But threshold also matters: threshold=0.50*500=250
        // With only 7 messages, we're below threshold, so no compression.
        let mw = ContextCompressionMiddleware::new();
        let mut ctx = test_ctx();
        ctx.messages = make_messages(7);

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        assert_eq!(ctx.messages.len(), 7);
    }

    #[tokio::test]
    async fn messages_equal_to_protected_no_middle_to_compress() {
        // threshold_percent=0.50, max_context_lines=6 → threshold=3
        // protect_first=2, protect_last=2 → need > 4 for a middle
        let mw = ContextCompressionMiddleware::with_config(0.50, 6, 2, 2);
        let mut ctx = test_ctx();
        ctx.messages = make_messages(4); // 4 > 3 (threshold), but 4 <= 2+2 (protected)

        let action = mw.before_model(&mut ctx).await;
        assert!(matches!(action, MiddlewareAction::Continue));
        // compress_messages returns original when len <= protected_total
        assert_eq!(ctx.messages.len(), 4);
    }

    #[tokio::test]
    async fn summary_contains_key_points() {
        let mw = ContextCompressionMiddleware::with_config(0.50, 10, 1, 1);
        let mut ctx = test_ctx();
        ctx.messages = make_messages(8); // 8 > 5 → triggers

        mw.before_model(&mut ctx).await;

        // Result: 1 head + 1 summary + 1 tail = 3
        assert_eq!(ctx.messages.len(), 3);

        // Summary should mention the compressed message contents.
        let summary = &ctx.messages[1];
        assert!(summary.contains("Key points:"));
        assert!(summary.contains("Message 1:"));
        assert!(summary.contains("Message 6:"));
    }

    #[tokio::test]
    async fn long_messages_truncated_in_summary() {
        let mw = ContextCompressionMiddleware::with_config(0.50, 4, 1, 1);
        let mut ctx = test_ctx();

        let long_msg = "x".repeat(300);
        ctx.messages = vec![
            "head".into(),
            long_msg.clone(),
            long_msg,
            "tail".into(),
        ];
        // 4 messages, threshold=2, protect 1+1=2, middle=2

        mw.before_model(&mut ctx).await;

        // 1 head + 1 summary + 1 tail = 3
        assert_eq!(ctx.messages.len(), 3);
        let summary = &ctx.messages[1];
        assert!(summary.contains("..."), "long messages should be truncated with ...");
    }

    #[test]
    fn build_summary_format() {
        let messages = vec![
            "First middle message".to_string(),
            "Second middle message".to_string(),
        ];
        let summary = ContextCompressionMiddleware::build_summary(&messages);
        assert!(summary.starts_with("[Context compressed: 2 messages summarized"));
        assert!(summary.contains("First middle message"));
        assert!(summary.contains("Second middle message"));
    }
}
