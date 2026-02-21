use anyhow::{Context, Result};
use std::path::Path;

/// Identity files loaded from a rig directory.
/// These form the system prompt for agents operating within the rig.
#[derive(Debug, Clone, Default)]
pub struct Identity {
    /// Core personality and purpose (SOUL.md).
    pub soul: Option<String>,
    /// Name, style, expertise (IDENTITY.md).
    pub identity: Option<String>,
    /// Operating instructions (AGENTS.md).
    pub agents: Option<String>,
    /// Periodic check instructions (HEARTBEAT.md).
    pub heartbeat: Option<String>,
    /// Persistent memories (MEMORY.md).
    pub memory: Option<String>,
    /// Operational knowledge and learnings (KNOWLEDGE.md).
    pub knowledge: Option<String>,
}

impl Identity {
    /// Load identity files from a rig directory.
    pub fn load(rig_dir: &Path) -> Result<Self> {
        Ok(Self {
            soul: load_optional(rig_dir, "SOUL.md")?,
            identity: load_optional(rig_dir, "IDENTITY.md")?,
            agents: load_optional(rig_dir, "AGENTS.md")?,
            heartbeat: load_optional(rig_dir, "HEARTBEAT.md")?,
            memory: load_optional(rig_dir, "MEMORY.md")?,
            knowledge: load_optional(rig_dir, "KNOWLEDGE.md")?,
        })
    }

    /// Build the system prompt from identity files.
    pub fn system_prompt(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref soul) = self.soul {
            parts.push(format!("# Soul\n\n{soul}"));
        }

        if let Some(ref identity) = self.identity {
            parts.push(format!("# Identity\n\n{identity}"));
        }

        if let Some(ref agents) = self.agents {
            parts.push(format!("# Operating Instructions\n\n{agents}"));
        }

        if let Some(ref knowledge) = self.knowledge {
            parts.push(format!("# Operational Knowledge\n\n{knowledge}"));
        }

        if let Some(ref memory) = self.memory {
            parts.push(format!("# Persistent Memory\n\n{memory}"));
        }

        if parts.is_empty() {
            "You are a helpful AI agent.".to_string()
        } else {
            parts.join("\n\n---\n\n")
        }
    }

    /// Check if any identity files are loaded.
    pub fn is_loaded(&self) -> bool {
        self.soul.is_some()
            || self.identity.is_some()
            || self.agents.is_some()
    }
}

fn load_optional(dir: &Path, filename: &str) -> Result<Option<String>> {
    let path = dir.join(filename);
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if content.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(content))
        }
    } else {
        Ok(None)
    }
}
