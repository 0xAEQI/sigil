use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Instant;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Result of a Claude Code CLI execution.
#[derive(Debug)]
pub struct ExecutionResult {
    /// The assistant's final response text.
    pub result_text: String,
    /// Session ID (if returned).
    pub session_id: Option<String>,
    /// Number of agentic turns used.
    pub num_turns: u32,
    /// Total cost in USD.
    pub total_cost_usd: f64,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

/// Spawns Claude Code CLI instances for bead execution.
///
/// Each execution is ephemeral: no session persistence, no interactive mode.
/// The worker's identity is injected via `--append-system-prompt` and the
/// repo's CLAUDE.md is auto-discovered from the working directory.
pub struct ClaudeCodeExecutor {
    /// Working directory (rig's repo path).
    workdir: PathBuf,
    /// Claude Code model (e.g., "claude-sonnet-4-6").
    model: String,
    /// Max agentic turns per execution.
    max_turns: u32,
    /// Max budget in USD per execution (None = unlimited).
    max_budget_usd: Option<f64>,
    /// Tool allowlist (Claude Code tool names).
    allowed_tools: Vec<String>,
}

/// Default tool allowlist for code-working rigs.
const DEFAULT_CODE_TOOLS: &[&str] = &[
    "Bash", "Read", "Write", "Edit", "Grep", "Glob", "Task",
];

impl ClaudeCodeExecutor {
    pub fn new(
        workdir: PathBuf,
        model: String,
        max_turns: u32,
        max_budget_usd: Option<f64>,
    ) -> Self {
        Self {
            workdir,
            model,
            max_turns,
            max_budget_usd,
            allowed_tools: DEFAULT_CODE_TOOLS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Execute a bead via Claude Code CLI.
    ///
    /// Spawns `claude -p "<bead_context>"` with the rig's identity as
    /// `--append-system-prompt`. Returns the parsed result from JSON output.
    pub async fn execute(
        &self,
        identity: &sigil_core::Identity,
        bead_context: &str,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();

        let mut cmd = Command::new("claude");

        // Core flags.
        cmd.arg("-p").arg(bead_context);
        cmd.arg("--output-format").arg("json");
        cmd.arg("--permission-mode").arg("bypassPermissions");
        cmd.arg("--model").arg(&self.model);
        cmd.arg("--max-turns").arg(self.max_turns.to_string());
        cmd.arg("--no-session-persistence");

        // Budget cap if configured.
        if let Some(budget) = self.max_budget_usd {
            cmd.arg("--max-budget-usd").arg(budget.to_string());
        }

        // Tool allowlist.
        if !self.allowed_tools.is_empty() {
            cmd.arg("--allowedTools").arg(self.allowed_tools.join(","));
        }

        // Identity as system prompt appendage.
        let system_prompt = identity.system_prompt();
        if !system_prompt.is_empty() {
            cmd.arg("--append-system-prompt").arg(&system_prompt);
        }

        // Working directory.
        cmd.current_dir(&self.workdir);

        // CRITICAL: Unset CLAUDECODE env var to avoid nested-session block.
        cmd.env_remove("CLAUDECODE");
        // Also unset CLAUDE_CODE to be safe.
        cmd.env_remove("CLAUDE_CODE");

        debug!(
            workdir = %self.workdir.display(),
            model = %self.model,
            max_turns = self.max_turns,
            "spawning claude code"
        );

        let output = cmd
            .output()
            .await
            .context("failed to spawn claude CLI — is it installed?")?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            warn!(
                exit_code = ?output.status.code(),
                stderr = %stderr,
                "claude code exited with error"
            );
            anyhow::bail!(
                "claude code failed (exit {}): {}",
                output.status.code().unwrap_or(-1),
                if stderr.is_empty() { &stdout } else { &stderr },
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_json_output(&stdout, duration_ms)
    }

    /// Parse the `--output-format json` response from Claude Code.
    ///
    /// The JSON output has this shape:
    /// ```json
    /// {
    ///   "type": "result",
    ///   "result": "the assistant's response text",
    ///   "session_id": "...",
    ///   "num_turns": 5,
    ///   "total_cost_usd": 0.12,
    ///   ...
    /// }
    /// ```
    fn parse_json_output(stdout: &str, duration_ms: u64) -> Result<ExecutionResult> {
        let v: serde_json::Value = serde_json::from_str(stdout)
            .context("failed to parse claude code JSON output")?;

        let result_text = v.get("result")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        let session_id = v.get("session_id")
            .and_then(|s| s.as_str())
            .map(String::from);

        let num_turns = v.get("num_turns")
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as u32;

        let total_cost_usd = v.get("total_cost_usd")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0);

        info!(
            turns = num_turns,
            cost_usd = total_cost_usd,
            duration_ms = duration_ms,
            result_len = result_text.len(),
            "claude code execution complete"
        );

        Ok(ExecutionResult {
            result_text,
            session_id,
            num_turns,
            total_cost_usd,
            duration_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_output() {
        let json = r#"{
            "type": "result",
            "result": "I fixed the bug in main.rs",
            "session_id": "abc-123",
            "num_turns": 3,
            "total_cost_usd": 0.08
        }"#;

        let result = ClaudeCodeExecutor::parse_json_output(json, 5000).unwrap();
        assert_eq!(result.result_text, "I fixed the bug in main.rs");
        assert_eq!(result.session_id, Some("abc-123".to_string()));
        assert_eq!(result.num_turns, 3);
        assert!((result.total_cost_usd - 0.08).abs() < f64::EPSILON);
        assert_eq!(result.duration_ms, 5000);
    }

    #[test]
    fn test_parse_minimal_json() {
        let json = r#"{"type": "result", "result": "done"}"#;
        let result = ClaudeCodeExecutor::parse_json_output(json, 100).unwrap();
        assert_eq!(result.result_text, "done");
        assert_eq!(result.num_turns, 0);
        assert_eq!(result.total_cost_usd, 0.0);
    }
}
