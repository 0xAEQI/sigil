use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sigil_core::traits::{LogObserver, Observer, Provider, Tool};
use sigil_core::{Agent, AgentConfig, Identity, SecretStore, SigilConfig};
use sigil_providers::OpenRouterProvider;
use sigil_tools::{FileReadTool, FileWriteTool, ListDirTool, ShellTool};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "sg", version, about = "Sigil — Multi-Agent Orchestration")]
struct Cli {
    /// Path to sigil.toml config file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a one-shot agent with a prompt.
    Run {
        /// The prompt/task for the agent.
        prompt: String,

        /// Rig to run in (loads rig identity files).
        #[arg(short, long)]
        rig: Option<String>,

        /// Model override.
        #[arg(short, long)]
        model: Option<String>,

        /// Maximum iterations.
        #[arg(long, default_value = "20")]
        max_iterations: u32,
    },

    /// Initialize Sigil in the current directory.
    Init,

    /// Manage encrypted secrets.
    Secrets {
        #[command(subcommand)]
        action: SecretsAction,
    },

    /// Run diagnostics.
    Doctor,

    /// Show system status.
    Status,
}

#[derive(Subcommand)]
enum SecretsAction {
    /// Set a secret value.
    Set {
        /// Secret name.
        name: String,
        /// Secret value.
        value: String,
    },
    /// Get a secret value.
    Get {
        /// Secret name.
        name: String,
    },
    /// List all secrets.
    List,
    /// Delete a secret.
    Delete {
        /// Secret name.
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level)),
        )
        .with_target(false)
        .init();

    match cli.command {
        Commands::Run {
            prompt,
            rig,
            model,
            max_iterations,
        } => cmd_run(&cli.config, &prompt, rig.as_deref(), model.as_deref(), max_iterations).await,
        Commands::Init => cmd_init().await,
        Commands::Secrets { action } => cmd_secrets(&cli.config, action).await,
        Commands::Doctor => cmd_doctor(&cli.config).await,
        Commands::Status => cmd_status(&cli.config).await,
    }
}

async fn cmd_run(
    config_path: &Option<PathBuf>,
    prompt: &str,
    rig_name: Option<&str>,
    model_override: Option<&str>,
    max_iterations: u32,
) -> Result<()> {
    let (config, _config_path) = load_config(config_path)?;

    // Determine model.
    let model = model_override
        .map(String::from)
        .or_else(|| rig_name.map(|r| config.model_for_rig(r)))
        .unwrap_or_else(|| {
            config
                .providers
                .openrouter
                .as_ref()
                .map(|or| or.default_model.clone())
                .unwrap_or_else(|| "anthropic/claude-sonnet-4.6".to_string())
        });

    // Build provider.
    let or_config = config
        .providers
        .openrouter
        .as_ref()
        .context("no OpenRouter provider configured in sigil.toml")?;

    let api_key = if or_config.api_key.is_empty() {
        // Try secret store.
        let store_path = config
            .security
            .secret_store
            .as_ref()
            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|| config.data_dir().join("secrets"));
        let store = SecretStore::open(&store_path)?;
        store
            .get("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY not set. Use `sg secrets set OPENROUTER_API_KEY <key>`")?
    } else {
        or_config.api_key.clone()
    };

    let provider = Arc::new(OpenRouterProvider::new(api_key, model.clone()));

    // Build tools.
    let workdir = if let Some(rig_name) = rig_name {
        config
            .rig(rig_name)
            .map(|r| PathBuf::from(&r.repo))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    } else {
        std::env::current_dir().unwrap_or_default()
    };

    let tools: Vec<Arc<dyn Tool>> = vec![
        Arc::new(ShellTool::new(workdir.clone())),
        Arc::new(FileReadTool::new(workdir.clone())),
        Arc::new(FileWriteTool::new(workdir.clone())),
        Arc::new(ListDirTool::new(workdir.clone())),
    ];

    // Load identity.
    let identity = if let Some(rig_name) = rig_name {
        let rig_dir = find_rig_dir(rig_name)?;
        Identity::load(&rig_dir).unwrap_or_default()
    } else {
        Identity::default()
    };

    // Build observer.
    let observer: Arc<dyn Observer> = Arc::new(LogObserver);

    // Build agent config.
    let agent_config = AgentConfig {
        model,
        max_iterations,
        name: rig_name.unwrap_or("default").to_string(),
        ..Default::default()
    };

    // Run agent.
    info!(prompt = %prompt, "starting agent");
    let agent = Agent::new(agent_config, provider, tools, observer, identity);
    let result = agent.run(prompt).await?;

    // Print final output.
    println!("{result}");

    Ok(())
}

async fn cmd_init() -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Create config directory.
    let config_dir = cwd.join("config");
    std::fs::create_dir_all(&config_dir)?;

    // Create rigs directory.
    let rigs_dir = cwd.join("rigs");
    std::fs::create_dir_all(&rigs_dir)?;

    // Create default config.
    let config_path = config_dir.join("sigil.toml");
    if !config_path.exists() {
        let default_config = r#"[sigil]
name = "my-sigil"
data_dir = "~/.sigil"

[providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
default_model = "anthropic/claude-sonnet-4.6"
fallback_model = "anthropic/claude-haiku-4-5-20251001"

[security]
autonomy = "supervised"
workspace_only = true
max_cost_per_day_usd = 10.0

[memory]
backend = "sqlite"
temporal_decay_halflife_days = 30

[heartbeat]
enabled = false
default_interval_minutes = 30
"#;
        std::fs::write(&config_path, default_config)?;
        println!("Created config/sigil.toml");
    } else {
        println!("config/sigil.toml already exists");
    }

    // Create data directory.
    let data_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sigil");
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(data_dir.join("secrets"))?;
    println!("Created ~/.sigil/");

    println!("\nSigil initialized. Next steps:");
    println!("  1. Set your API key: sg secrets set OPENROUTER_API_KEY sk-or-...");
    println!("  2. Add rigs to config/sigil.toml");
    println!("  3. Create rig directories under rigs/");
    println!("  4. Run: sg run \"hello world\"");

    Ok(())
}

async fn cmd_secrets(config_path: &Option<PathBuf>, action: SecretsAction) -> Result<()> {
    let store_path = if let Ok((config, _)) = load_config(config_path) {
        config
            .security
            .secret_store
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| config.data_dir().join("secrets"))
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sigil/secrets")
    };

    let store = SecretStore::open(&store_path)?;

    match action {
        SecretsAction::Set { name, value } => {
            store.set(&name, &value)?;
            println!("Secret '{name}' stored.");
        }
        SecretsAction::Get { name } => {
            let value = store.get(&name)?;
            println!("{value}");
        }
        SecretsAction::List => {
            let names = store.list()?;
            if names.is_empty() {
                println!("No secrets stored.");
            } else {
                for name in names {
                    println!("  {name}");
                }
            }
        }
        SecretsAction::Delete { name } => {
            store.delete(&name)?;
            println!("Secret '{name}' deleted.");
        }
    }

    Ok(())
}

async fn cmd_doctor(config_path: &Option<PathBuf>) -> Result<()> {
    println!("Sigil Doctor");
    println!("============\n");

    // Check config.
    match load_config(config_path) {
        Ok((config, path)) => {
            println!("[OK] Config loaded from {}", path.display());

            // Check OpenRouter API key.
            if let Some(ref or) = config.providers.openrouter {
                if or.api_key.is_empty() {
                    println!("[WARN] OpenRouter API key not set");
                } else {
                    let provider = OpenRouterProvider::new(or.api_key.clone(), or.default_model.clone());
                    match provider.health_check().await {
                        Ok(()) => println!("[OK] OpenRouter API key valid"),
                        Err(e) => println!("[FAIL] OpenRouter health check: {e}"),
                    }
                }
            } else {
                println!("[WARN] No OpenRouter provider configured");
            }

            // Check rigs.
            for rig in &config.rigs {
                let repo_path = PathBuf::from(&rig.repo);
                if repo_path.exists() {
                    println!("[OK] Rig '{}' repo exists: {}", rig.name, rig.repo);
                } else {
                    println!("[WARN] Rig '{}' repo missing: {}", rig.name, rig.repo);
                }

                match find_rig_dir(&rig.name) {
                    Ok(rig_dir) => {
                        let has_soul = rig_dir.join("SOUL.md").exists();
                        let has_identity = rig_dir.join("IDENTITY.md").exists();
                        if has_soul && has_identity {
                            println!("[OK] Rig '{}' identity files present", rig.name);
                        } else {
                            println!(
                                "[WARN] Rig '{}' missing identity files (SOUL.md: {}, IDENTITY.md: {})",
                                rig.name, has_soul, has_identity
                            );
                        }
                    }
                    Err(_) => {
                        println!("[WARN] Rig '{}' directory not found", rig.name);
                    }
                }
            }

            // Check secret store.
            let store_path = config
                .security
                .secret_store
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| config.data_dir().join("secrets"));
            if store_path.exists() {
                println!("[OK] Secret store exists: {}", store_path.display());
            } else {
                println!("[WARN] Secret store missing: {}", store_path.display());
            }
        }
        Err(e) => {
            println!("[FAIL] Config: {e}");
            println!("       Run `sg init` to create a config.");
        }
    }

    // Check beads_rust.
    match tokio::process::Command::new("br")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("[OK] beads_rust: {}", version.trim());
        }
        _ => {
            println!("[INFO] beads_rust not installed (needed for Phase 2)");
        }
    }

    println!("\nDone.");
    Ok(())
}

async fn cmd_status(config_path: &Option<PathBuf>) -> Result<()> {
    let (config, _) = load_config(config_path)?;

    println!("Sigil Status: {}", config.sigil.name);
    println!("===============================\n");

    println!("Rigs:");
    for rig in &config.rigs {
        let repo_exists = PathBuf::from(&rig.repo).exists();
        let status = if repo_exists { "OK" } else { "MISSING" };
        println!(
            "  {} [{}] prefix={} workers={} repo={}",
            rig.name, status, rig.prefix, rig.max_workers, rig.repo
        );
    }

    println!("\nDaemon: not running (Phase 3)");
    println!("Workers: 0 active");

    Ok(())
}

fn load_config(config_path: &Option<PathBuf>) -> Result<(SigilConfig, PathBuf)> {
    if let Some(path) = config_path {
        Ok((SigilConfig::load(path)?, path.clone()))
    } else {
        SigilConfig::discover()
    }
}

fn find_rig_dir(name: &str) -> Result<PathBuf> {
    // Search order: ./rigs/<name>, ../rigs/<name>, sigil project root/rigs/<name>
    let candidates = [
        PathBuf::from(format!("rigs/{name}")),
        PathBuf::from(format!("../rigs/{name}")),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Try from the config file location.
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("rigs").join(name);
            if candidate.exists() {
                return Ok(candidate);
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    anyhow::bail!("rig directory not found: {name}")
}
