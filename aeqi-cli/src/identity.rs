use anyhow::Result;
use std::path::Path;

/// Identity files loaded from agent + project directories.
///
/// Two-source loading:
///   - Agent personality (PERSONA, IDENTITY, PREFERENCES, MEMORY) from `agents/{name}/`
///   - Project context (AGENTS, KNOWLEDGE) from `projects/{name}/`
///   - Shared workflow from `agents/shared/WORKFLOW.md`
#[derive(Debug, Clone, Default)]
pub struct Identity {
    /// Core personality and purpose (PERSONA.md — from agent dir).
    pub persona: Option<String>,
    /// Name, style, expertise (IDENTITY.md — from agent dir).
    pub identity: Option<String>,
    /// Operational instructions separated from personality (OPERATIONAL.md — from agent dir).
    pub operational: Option<String>,
    /// Operating instructions (AGENTS.md — from project dir).
    pub agents: Option<String>,
    /// Accumulated evolution patches (EVOLUTION.md — from agent dir, written by lifecycle engine).
    pub evolution: Option<String>,
    /// Persistent memories (MEMORY.md — from agent dir).
    pub memory: Option<String>,
    /// Operational knowledge and learnings (KNOWLEDGE.md — from project dir).
    pub knowledge: Option<String>,
    /// Architect's observed preferences (PREFERENCES.md — from agent dir).
    pub preferences: Option<String>,
    /// Shared workflow from agents/shared/WORKFLOW.md.
    pub shared_workflow: Option<String>,
    /// Skill-specific system prompt (injected at runtime from skill TOML).
    pub skill_prompt: Option<String>,
}

impl Identity {
    /// Load identity files from an agent directory + optional project directory.
    pub fn load(agent_dir: &Path, domain_dir: Option<&Path>) -> Result<Self> {
        let shared_dir = agent_dir.parent().map(|p| p.join("shared"));

        Ok(Self {
            persona: load_optional(agent_dir, "PERSONA.md")?,
            identity: load_optional(agent_dir, "IDENTITY.md")?,
            operational: load_optional(agent_dir, "OPERATIONAL.md")?,
            evolution: load_optional(agent_dir, "EVOLUTION.md")?,
            preferences: load_optional(agent_dir, "PREFERENCES.md")?,
            memory: load_optional(agent_dir, "MEMORY.md")?,
            agents: domain_dir
                .map(|d| load_optional(d, "AGENTS.md"))
                .transpose()?
                .flatten(),
            knowledge: domain_dir
                .map(|d| load_optional(d, "KNOWLEDGE.md"))
                .transpose()?
                .flatten(),
            shared_workflow: shared_dir
                .as_deref()
                .map(|d| load_optional(d, "WORKFLOW.md"))
                .transpose()?
                .flatten(),
            skill_prompt: None,
        })
    }

    /// Load identity from a single directory (loads all files from one dir).
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let shared_dir = dir.parent().map(|p| p.join("shared"));

        Ok(Self {
            persona: load_optional(dir, "PERSONA.md")?,
            identity: load_optional(dir, "IDENTITY.md")?,
            operational: load_optional(dir, "OPERATIONAL.md")?,
            evolution: load_optional(dir, "EVOLUTION.md")?,
            agents: load_optional(dir, "AGENTS.md")?,
            memory: load_optional(dir, "MEMORY.md")?,
            knowledge: load_optional(dir, "KNOWLEDGE.md")?,
            preferences: load_optional(dir, "PREFERENCES.md")?,
            shared_workflow: shared_dir
                .as_deref()
                .map(|d| load_optional(d, "WORKFLOW.md"))
                .transpose()?
                .flatten(),
            skill_prompt: None,
        })
    }

    /// Build the system prompt from identity files.
    pub fn system_prompt(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref shared) = self.shared_workflow {
            parts.push(format!("# Shared Workflow\n\n{shared}"));
        }
        if let Some(ref persona) = self.persona {
            parts.push(format!("# Persona\n\n{persona}"));
        }
        if let Some(ref identity) = self.identity {
            parts.push(format!("# Identity\n\n{identity}"));
        }
        if let Some(ref evolution) = self.evolution {
            parts.push(format!("# Evolution\n\n{evolution}"));
        }
        if let Some(ref operational) = self.operational {
            parts.push(format!("# Operational Instructions\n\n{operational}"));
        }
        if let Some(ref agents) = self.agents {
            parts.push(format!("# Operating Instructions\n\n{agents}"));
        }
        if let Some(ref knowledge) = self.knowledge {
            parts.push(format!("# Project Knowledge\n\n{knowledge}"));
        }
        if let Some(ref skill) = self.skill_prompt {
            parts.push(format!("# Active Skill\n\n{skill}"));
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
}

fn load_optional(dir: &Path, filename: &str) -> Result<Option<String>> {
    let path = dir.join(filename);
    match std::fs::read_to_string(&path) {
        Ok(content) if content.trim().is_empty() => Ok(None),
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow::anyhow!("failed to read {}: {e}", path.display())),
    }
}
