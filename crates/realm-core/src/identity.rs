use anyhow::{Context, Result};
use std::path::Path;

/// Identity files loaded from a rig directory.
/// These form the system prompt for agents operating within the rig.
///
/// Loading order: shared templates (rigs/shared/) → rig-specific files.
/// Rig-specific files override shared ones when both exist.
#[derive(Debug, Clone, Default)]
pub struct Identity {
    /// Core personality and purpose (SOUL.md).
    pub soul: Option<String>,
    /// Name, style, expertise (IDENTITY.md).
    pub identity: Option<String>,
    /// Operating instructions (AGENTS.md).
    pub agents: Option<String>,
    /// Periodic check instructions (HEARTBEAT.md).
    pub pulse: Option<String>,
    /// Persistent memories (MEMORY.md).
    pub memory: Option<String>,
    /// Operational knowledge and learnings (KNOWLEDGE.md).
    pub knowledge: Option<String>,
    /// Architect's observed preferences — updated by Aurelia across interactions (PREFERENCES.md).
    pub preferences: Option<String>,
    /// Shared workflow from rigs/shared/WORKFLOW.md.
    pub shared_workflow: Option<String>,
}

impl Identity {
    /// Load identity files from a rig directory.
    ///
    /// Also loads shared templates from `rigs/shared/` (sibling of domain_dir's parent).
    pub fn load(domain_dir: &Path) -> Result<Self> {
        let shared_dir = domain_dir.parent().map(|p| p.join("shared"));

        Ok(Self {
            soul: load_optional(domain_dir, "SOUL.md")?,
            identity: load_optional(domain_dir, "IDENTITY.md")?,
            agents: load_optional(domain_dir, "AGENTS.md")?,
            pulse: load_optional(domain_dir, "HEARTBEAT.md")?,
            memory: load_optional(domain_dir, "MEMORY.md")?,
            knowledge: load_optional(domain_dir, "KNOWLEDGE.md")?,
            preferences: load_optional(domain_dir, "PREFERENCES.md")?,
            shared_workflow: shared_dir
                .as_deref()
                .map(|d| load_optional(d, "WORKFLOW.md"))
                .transpose()?
                .flatten(),
        })
    }

    /// Build the system prompt from identity files.
    ///
    /// Order: shared workflow → soul → identity → agents → knowledge → memory.
    /// Shared workflow provides the base (git workflow, code standards, R→D→R pipeline).
    /// Rig-specific files layer on top with domain context.
    pub fn system_prompt(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref shared) = self.shared_workflow {
            parts.push(format!("# Shared Workflow\n\n{shared}"));
        }

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
            parts.push(format!("# Domain Knowledge\n\n{knowledge}"));
        }

        if let Some(ref preferences) = self.preferences {
            parts.push(format!("# Architect Preferences\n\n{preferences}"));
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
