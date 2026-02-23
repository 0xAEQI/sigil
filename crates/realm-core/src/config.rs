use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Master configuration loaded from realm.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmConfig {
    pub realm: RealmMeta,
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub pulse: PulseConfig,
    #[serde(default)]
    pub shadow: ShadowConfig,
    #[serde(default)]
    pub channels: ChannelsConfig,
    #[serde(default)]
    pub domains: Vec<DomainConfig>,
    /// Multi-familiar council members. If empty, auto-generated from [shadow].
    #[serde(default)]
    pub familiars: Vec<FamiliarConfig>,
    /// Council-level settings (cost caps, cooldowns).
    #[serde(default)]
    pub council: CouncilConfig,
    /// Session alarm and progress heartbeat settings.
    #[serde(default)]
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmMeta {
    pub name: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
}

fn default_data_dir() -> String {
    "~/.sigil".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub openrouter: Option<OpenRouterConfig>,
    #[serde(default)]
    pub anthropic: Option<AnthropicConfig>,
    #[serde(default)]
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    #[serde(default = "default_openrouter_model")]
    pub default_model: String,
    #[serde(default)]
    pub fallback_model: Option<String>,
    #[serde(default)]
    pub embedding_model: Option<String>,
}

fn default_openrouter_model() -> String {
    "anthropic/claude-sonnet-4.6".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    #[serde(default = "default_anthropic_model")]
    pub default_model: String,
}

fn default_anthropic_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_url")]
    pub url: String,
    #[serde(default = "default_ollama_model")]
    pub default_model: String,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3.1:8b".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_autonomy")]
    pub autonomy: Autonomy,
    #[serde(default = "default_true")]
    pub workspace_only: bool,
    #[serde(default = "default_max_cost")]
    pub max_cost_per_day_usd: f64,
    #[serde(default)]
    pub secret_store: Option<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            autonomy: Autonomy::Supervised,
            workspace_only: true,
            max_cost_per_day_usd: 10.0,
            secret_store: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Autonomy {
    Readonly,
    Supervised,
    Full,
}

fn default_autonomy() -> Autonomy {
    Autonomy::Supervised
}

fn default_true() -> bool {
    true
}

fn default_max_cost() -> f64 {
    10.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_memory_backend")]
    pub backend: String,
    #[serde(default = "default_embedding_dims")]
    pub embedding_dimensions: usize,
    #[serde(default = "default_vector_weight")]
    pub vector_weight: f64,
    #[serde(default = "default_keyword_weight")]
    pub keyword_weight: f64,
    #[serde(default = "default_decay_halflife")]
    pub temporal_decay_halflife_days: f64,
    #[serde(default = "default_mmr_lambda")]
    pub mmr_lambda: f64,
    #[serde(default = "default_chunk_size")]
    pub chunk_size_tokens: usize,
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap_tokens: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            backend: "sqlite".to_string(),
            embedding_dimensions: 1536,
            vector_weight: 0.6,
            keyword_weight: 0.4,
            temporal_decay_halflife_days: 30.0,
            mmr_lambda: 0.7,
            chunk_size_tokens: 400,
            chunk_overlap_tokens: 80,
        }
    }
}

fn default_memory_backend() -> String { "sqlite".to_string() }
fn default_embedding_dims() -> usize { 1536 }
fn default_vector_weight() -> f64 { 0.6 }
fn default_keyword_weight() -> f64 { 0.4 }
fn default_decay_halflife() -> f64 { 30.0 }
fn default_mmr_lambda() -> f64 { 0.7 }
fn default_chunk_size() -> usize { 400 }
fn default_chunk_overlap() -> usize { 80 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_pulse_interval")]
    pub default_interval_minutes: u32,
    /// Whether the autonomous self-reflection cycle is enabled.
    #[serde(default)]
    pub reflection_enabled: bool,
    /// Interval between reflection cycles in minutes (default: 240 = 4 h).
    #[serde(default = "default_reflection_interval")]
    pub reflection_interval_minutes: u32,
}

impl Default for PulseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_interval_minutes: 30,
            reflection_enabled: false,
            reflection_interval_minutes: 240,
        }
    }
}

fn default_pulse_interval() -> u32 { 30 }
fn default_reflection_interval() -> u32 { 240 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowConfig {
    #[serde(default = "default_fa_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_fa_workers")]
    pub max_workers: u32,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    /// Max agentic turns per Claude Code execution (default: 25).
    #[serde(default)]
    pub max_turns: Option<u32>,
    /// Max budget in USD per Claude Code execution.
    #[serde(default)]
    pub max_budget_usd: Option<f64>,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            prefix: "fa".to_string(),
            model: None,
            max_workers: 2,
            execution_mode: ExecutionMode::Agent,
            max_turns: None,
            max_budget_usd: None,
        }
    }
}

fn default_fa_prefix() -> String { "fa".to_string() }
fn default_fa_workers() -> u32 { 2 }

/// Role of a familiar in the council.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum FamiliarRole {
    /// Lead familiar — primary interface, synthesizes advisor input.
    #[default]
    Lead,
    /// Advisor familiar — provides specialized perspective to the lead.
    Advisor,
}


/// Configuration for a single familiar in the council.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamiliarConfig {
    pub name: String,
    #[serde(default = "default_fa_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub role: FamiliarRole,
    /// Domains this familiar specializes in (for routing classifier).
    #[serde(default)]
    pub domains: Vec<String>,
    /// Max budget per advisor call in USD.
    #[serde(default)]
    pub max_budget_usd: Option<f64>,
    /// Secret store key for this familiar's Telegram bot token.
    /// Each familiar can have its own bot identity in group chats.
    #[serde(default)]
    pub telegram_token_secret: Option<String>,
}

/// Configuration for the multi-familiar council.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CouncilConfig {
    /// Max total cost across all advisors per message, in USD.
    #[serde(default = "default_max_advisor_cost")]
    pub max_advisor_cost_usd: f64,
    /// Cooldown in seconds before same advisor can be re-invoked.
    #[serde(default = "default_advisor_cooldown")]
    pub advisor_cooldown_secs: u64,
}

fn default_max_advisor_cost() -> f64 { 0.50 }
fn default_advisor_cooldown() -> u64 { 60 }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelsConfig {
    #[serde(default)]
    pub telegram: Option<TelegramChannelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChannelConfig {
    pub token_secret: String,
    #[serde(default)]
    pub allowed_chats: Vec<i64>,
    #[serde(default = "default_debounce_window")]
    pub debounce_window_ms: u64,
}

fn default_debounce_window() -> u64 { 3000 }

/// Session alarm and progress heartbeat configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Enable session tracker (default: false).
    #[serde(default)]
    pub enabled: bool,
    /// Interval between active sprint check-ins in minutes (default: 30).
    #[serde(default = "default_checkin_interval")]
    pub checkin_interval_mins: u64,
    /// Interval between idle "get back" alarms in minutes (default: 60).
    #[serde(default = "default_alarm_interval")]
    pub alarm_interval_mins: u64,
    /// Anti-flood floor: minimum minutes between any two messages (default: 30).
    #[serde(default = "default_min_flood_interval")]
    pub min_flood_interval_mins: u64,
    /// Optional session deadline in minutes. Fires a one-shot alarm when elapsed.
    #[serde(default)]
    pub deadline_mins: Option<u64>,
    /// Override chat_id for session notifications. Falls back to first allowed_chat.
    #[serde(default)]
    pub notify_chat_id: Option<i64>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            checkin_interval_mins: 30,
            alarm_interval_mins: 60,
            min_flood_interval_mins: 30,
            deadline_mins: None,
            notify_chat_id: None,
        }
    }
}

fn default_checkin_interval() -> u64 { 30 }
fn default_alarm_interval() -> u64 { 60 }
fn default_min_flood_interval() -> u64 { 30 }

/// How a rig's workers execute beads.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Lightweight internal agent loop (for orchestration/triage).
    #[default]
    Agent,
    /// Spawn Claude Code CLI instance (for real code work).
    ClaudeCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub name: String,
    pub prefix: String,
    pub repo: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_max_workers")]
    pub max_workers: u32,
    #[serde(default)]
    pub worktree_root: Option<String>,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    /// Max agentic turns per Claude Code execution (default: 25).
    #[serde(default = "default_max_turns")]
    pub max_turns: Option<u32>,
    /// Max budget in USD per Claude Code execution.
    #[serde(default)]
    pub max_budget_usd: Option<f64>,
    /// Timeout in seconds for spirit execution. Hung spirits are aborted after this.
    #[serde(default = "default_spirit_timeout")]
    pub spirit_timeout_secs: u64,
    /// Per-domain daily cost ceiling in USD. If set, this domain's spending is
    /// capped independently from the global daily budget.
    #[serde(default)]
    pub max_cost_per_day_usd: Option<f64>,
}

fn default_max_workers() -> u32 { 2 }
fn default_max_turns() -> Option<u32> { Some(25) }
fn default_spirit_timeout() -> u64 { 1800 } // 30 minutes

impl RealmConfig {
    /// Load config from a TOML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;
        Self::parse(&content)
    }

    /// Parse config from TOML string.
    pub fn parse(content: &str) -> Result<Self> {
        let mut config: Self = toml::from_str(content)
            .context("failed to parse realm.toml")?;

        // Resolve environment variables in API keys.
        if let Some(ref mut or) = config.providers.openrouter {
            or.api_key = resolve_env(&or.api_key);
        }
        if let Some(ref mut a) = config.providers.anthropic {
            a.api_key = resolve_env(&a.api_key);
        }

        // Backward-compat: if no [[familiars]] configured, generate a single
        // lead familiar from [shadow].
        if config.familiars.is_empty() {
            config.familiars.push(FamiliarConfig {
                name: "aurelia".to_string(),
                prefix: config.shadow.prefix.clone(),
                model: config.shadow.model.clone(),
                role: FamiliarRole::Lead,
                domains: Vec::new(),
                max_budget_usd: config.shadow.max_budget_usd,
                telegram_token_secret: None,
            });
        }

        // Expand ~ in paths.
        config.realm.data_dir = expand_tilde(&config.realm.data_dir);
        for domain in &mut config.domains {
            domain.repo = expand_tilde(&domain.repo);
            if let Some(ref mut wt) = domain.worktree_root {
                *wt = expand_tilde(wt);
            }
        }

        // Validate and warn (non-fatal — partial configs allowed during dev).
        let issues = config.validate();
        for issue in &issues {
            warn!(issue = %issue, "config validation warning");
        }

        Ok(config)
    }

    /// Find config by searching upward from cwd, then ~/.sigil/, then /etc/sigil/.
    pub fn discover() -> Result<(Self, PathBuf)> {
        // 1. Check REALM_CONFIG env var.
        if let Ok(path) = std::env::var("REALM_CONFIG") {
            let path = PathBuf::from(path);
            return Ok((Self::load(&path)?, path));
        }

        // 2. Walk up from cwd looking for realm.toml or config/realm.toml.
        if let Ok(cwd) = std::env::current_dir() {
            let mut dir = cwd.as_path();
            loop {
                let candidate = dir.join("realm.toml");
                if candidate.exists() {
                    return Ok((Self::load(&candidate)?, candidate));
                }
                let candidate = dir.join("config/realm.toml");
                if candidate.exists() {
                    return Ok((Self::load(&candidate)?, candidate));
                }
                match dir.parent() {
                    Some(parent) => dir = parent,
                    None => break,
                }
            }
        }

        // 3. Check ~/.sigil/realm.toml.
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join(".sigil/realm.toml");
            if candidate.exists() {
                return Ok((Self::load(&candidate)?, candidate));
            }
        }

        anyhow::bail!("No realm.toml found. Run `rm init` to create one.")
    }

    /// Get rig config by name.
    pub fn domain(&self, name: &str) -> Option<&DomainConfig> {
        self.domains.iter().find(|r| r.name == name)
    }

    /// Get the default model for a rig, falling back to provider default.
    pub fn model_for_domain(&self, domain_name: &str) -> String {
        // Check familiar config first.
        if domain_name == "familiar"
            && let Some(ref m) = self.shadow.model {
                return m.clone();
            }

        self.domain(domain_name)
            .and_then(|r| r.model.clone())
            .or_else(|| {
                self.providers
                    .openrouter
                    .as_ref()
                    .map(|or| or.default_model.clone())
            })
            .unwrap_or_else(|| "anthropic/claude-sonnet-4.6".to_string())
    }

    /// Resolve the data directory path.
    pub fn data_dir(&self) -> PathBuf {
        PathBuf::from(&self.realm.data_dir)
    }

    /// Get the lead familiar config.
    pub fn lead_familiar(&self) -> Option<&FamiliarConfig> {
        self.familiars.iter().find(|f| f.role == FamiliarRole::Lead)
    }

    /// Get all advisor familiars.
    pub fn advisor_familiars(&self) -> Vec<&FamiliarConfig> {
        self.familiars.iter().filter(|f| f.role == FamiliarRole::Advisor).collect()
    }

    /// Get a familiar config by name.
    pub fn familiar(&self, name: &str) -> Option<&FamiliarConfig> {
        self.familiars.iter().find(|f| f.name == name)
    }

    /// Validate config for logical errors that serde can't catch.
    /// Returns collected error messages. Empty vec = valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.realm.name.is_empty() {
            errors.push("realm.name is empty".to_string());
        }

        // Domain validation.
        let mut seen_names = std::collections::HashSet::new();
        let mut seen_prefixes = std::collections::HashSet::new();
        for d in &self.domains {
            if d.name.is_empty() {
                errors.push("domain with empty name".to_string());
            }
            if d.prefix.is_empty() {
                errors.push(format!("domain '{}' has empty prefix", d.name));
            }
            if !seen_names.insert(&d.name) {
                errors.push(format!("duplicate domain name: '{}'", d.name));
            }
            if !seen_prefixes.insert(&d.prefix) {
                errors.push(format!("duplicate domain prefix: '{}'", d.prefix));
            }
            if d.spirit_timeout_secs == 0 {
                errors.push(format!("domain '{}' has zero spirit_timeout_secs", d.name));
            }
            if d.max_workers == 0 {
                errors.push(format!("domain '{}' has zero max_workers", d.name));
            }
        }

        // Familiar validation.
        let lead_count = self.familiars.iter().filter(|f| f.role == FamiliarRole::Lead).count();
        if lead_count == 0 {
            errors.push("no lead familiar configured".to_string());
        } else if lead_count > 1 {
            errors.push(format!("{lead_count} lead familiars configured — expected exactly 1"));
        }
        let mut seen_familiar_names = std::collections::HashSet::new();
        for f in &self.familiars {
            if f.name.is_empty() {
                errors.push("familiar with empty name".to_string());
            }
            if !seen_familiar_names.insert(&f.name) {
                errors.push(format!("duplicate familiar name: '{}'", f.name));
            }
        }

        // Memory weights.
        let weight_sum = self.memory.vector_weight + self.memory.keyword_weight;
        if (weight_sum - 1.0).abs() > 0.01 {
            errors.push(format!(
                "memory weights sum to {weight_sum:.2} (expected ~1.0): vector={}, keyword={}",
                self.memory.vector_weight, self.memory.keyword_weight
            ));
        }
        if self.memory.chunk_overlap_tokens >= self.memory.chunk_size_tokens {
            errors.push(format!(
                "chunk_overlap_tokens ({}) >= chunk_size_tokens ({})",
                self.memory.chunk_overlap_tokens, self.memory.chunk_size_tokens
            ));
        }

        // Budget sanity.
        if self.security.max_cost_per_day_usd <= 0.0 {
            errors.push("max_cost_per_day_usd must be positive".to_string());
        }

        errors
    }
}

/// Resolve ${ENV_VAR} patterns in strings.
fn resolve_env(s: &str) -> String {
    if s.starts_with("${") && s.ends_with('}') {
        let var_name = &s[2..s.len() - 1];
        std::env::var(var_name).unwrap_or_default()
    } else {
        s.to_string()
    }
}

/// Expand ~ to home directory.
fn expand_tilde(s: &str) -> String {
    if s.starts_with('~')
        && let Some(home) = dirs::home_dir() {
            return s.replacen('~', &home.to_string_lossy(), 1);
        }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[realm]
name = "test"

[[domains]]
name = "test-domain"
prefix = "td"
repo = "/tmp/test"
"#;
        let config = RealmConfig::parse(toml).unwrap();
        assert_eq!(config.realm.name, "test");
        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.domains[0].name, "test-domain");
    }

    #[test]
    fn test_resolve_env() {
        // SAFETY: test runs single-threaded, no concurrent env access.
        unsafe { std::env::set_var("TEST_SIGIL_VAR", "hello") };
        assert_eq!(resolve_env("${TEST_SIGIL_VAR}"), "hello");
        assert_eq!(resolve_env("plain"), "plain");
        unsafe { std::env::remove_var("TEST_SIGIL_VAR") };
    }

    #[test]
    fn test_validate_valid_config() {
        let toml = r#"
[realm]
name = "test"

[[domains]]
name = "alpha"
prefix = "al"
repo = "/tmp/alpha"

[[domains]]
name = "beta"
prefix = "bt"
repo = "/tmp/beta"
"#;
        let config = RealmConfig::parse(toml).unwrap();
        let issues = config.validate();
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn test_validate_duplicate_prefix() {
        let toml = r#"
[realm]
name = "test"

[[domains]]
name = "alpha"
prefix = "ab"
repo = "/tmp/alpha"

[[domains]]
name = "beta"
prefix = "ab"
repo = "/tmp/beta"
"#;
        let config = RealmConfig::parse(toml).unwrap();
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("duplicate domain prefix")), "expected duplicate prefix: {issues:?}");
    }

    #[test]
    fn test_validate_bad_memory_weights() {
        let toml = r#"
[realm]
name = "test"

[memory]
vector_weight = 0.9
keyword_weight = 0.9
"#;
        let config = RealmConfig::parse(toml).unwrap();
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("weights sum")), "expected weight warning: {issues:?}");
    }

    #[test]
    fn test_validate_chunk_overlap_too_large() {
        let toml = r#"
[realm]
name = "test"

[memory]
chunk_size_tokens = 100
chunk_overlap_tokens = 150
"#;
        let config = RealmConfig::parse(toml).unwrap();
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("chunk_overlap_tokens")), "expected overlap warning: {issues:?}");
    }
}
