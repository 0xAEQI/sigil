use anyhow::Result;
use async_trait::async_trait;
use sigil_core::traits::{ToolResult, ToolSpec};
use std::path::PathBuf;
use tracing::debug;

pub struct GrepTool {
    workspace: PathBuf,
}

impl GrepTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl sigil_core::traits::Tool for GrepTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'pattern' argument"))?;

        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let glob_filter = args.get("glob").and_then(|v| v.as_str());

        let include_lines = args
            .get("include_lines")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let context = args
            .get("context")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        let search_path = if std::path::Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.workspace.join(path)
        };

        debug!(pattern, path = %search_path.display(), "grep search");

        let mut cmd = tokio::process::Command::new("rg");
        cmd.arg("--no-heading")
            .arg("--color=never")
            .arg("--max-count=1000");

        if include_lines {
            cmd.arg("--line-number");
        } else {
            cmd.arg("--files-with-matches");
        }

        if context > 0 && include_lines {
            cmd.arg(format!("--context={context}"));
        }

        if let Some(g) = glob_filter {
            cmd.arg("--glob").arg(g);
        }

        cmd.arg("--").arg(pattern).arg(&search_path);

        let output = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            cmd.output(),
        )
        .await
        {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => {
                return Ok(ToolResult::error(format!(
                    "failed to run rg: {e}. Is ripgrep installed?"
                )));
            }
            Err(_) => {
                return Ok(ToolResult::error("grep timed out after 30s"));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.is_empty() {
            return Ok(ToolResult::success("no matches found"));
        }

        let lines: Vec<&str> = stdout.lines().collect();
        let total = lines.len();
        let limited: String = lines
            .into_iter()
            .take(max_results as usize)
            .collect::<Vec<_>>()
            .join("\n");

        let suffix = if total > max_results as usize {
            format!("\n\n... ({total} total matches, showing first {max_results})")
        } else {
            String::new()
        };

        Ok(ToolResult::success(format!("{limited}{suffix}")))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "grep".to_string(),
            description: "Search file contents using regex patterns (powered by ripgrep). Returns matching lines with file paths and line numbers.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory or file to search in (default: workspace root)"
                    },
                    "glob": {
                        "type": "string",
                        "description": "Glob pattern to filter files (e.g. '*.rs', '*.{ts,tsx}')"
                    },
                    "include_lines": {
                        "type": "boolean",
                        "description": "Show matching lines (true) or just file paths (false). Default: true"
                    },
                    "context": {
                        "type": "integer",
                        "description": "Lines of context around each match (default: 0)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum result lines to return (default: 50)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    fn name(&self) -> &str {
        "grep"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigil_core::traits::Tool;
    use tempfile::TempDir;

    async fn setup() -> (GrepTool, TempDir) {
        let dir = TempDir::new().unwrap();
        tokio::fs::write(dir.path().join("hello.rs"), "fn main() {\n    println!(\"hello\");\n}\n")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("world.rs"), "fn world() {\n    println!(\"world\");\n}\n")
            .await
            .unwrap();
        let tool = GrepTool::new(dir.path().to_path_buf());
        (tool, dir)
    }

    #[tokio::test]
    async fn finds_pattern() {
        let (tool, _dir) = setup().await;
        let result = tool
            .execute(serde_json::json!({ "pattern": "println" }))
            .await
            .unwrap();
        assert!(!result.is_error, "{}", result.output);
        assert!(result.output.contains("hello.rs"));
        assert!(result.output.contains("world.rs"));
    }

    #[tokio::test]
    async fn glob_filter_works() {
        let (tool, dir) = setup().await;
        tokio::fs::write(dir.path().join("data.txt"), "println in txt\n")
            .await
            .unwrap();
        let result = tool
            .execute(serde_json::json!({
                "pattern": "println",
                "glob": "*.rs"
            }))
            .await
            .unwrap();
        assert!(!result.is_error, "{}", result.output);
        assert!(!result.output.contains("data.txt"));
    }

    #[tokio::test]
    async fn files_only_mode() {
        let (tool, _dir) = setup().await;
        let result = tool
            .execute(serde_json::json!({
                "pattern": "fn main",
                "include_lines": false
            }))
            .await
            .unwrap();
        assert!(!result.is_error, "{}", result.output);
        assert!(result.output.contains("hello.rs"));
        assert!(!result.output.contains("println"));
    }

    #[tokio::test]
    async fn no_matches() {
        let (tool, _dir) = setup().await;
        let result = tool
            .execute(serde_json::json!({ "pattern": "nonexistent_xyz" }))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("no matches"));
    }
}
