use anyhow::Result;

pub(crate) async fn cmd_init() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config_dir = cwd.join("config");
    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(cwd.join("projects"))?;

    let config_path = config_dir.join("sigil.toml");
    if !config_path.exists() {
        std::fs::write(
            &config_path,
            r#"[sigil]
name = "my-project"
data_dir = "~/.sigil"

[providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
default_model = "minimax/minimax-m2.5"
fallback_model = "deepseek/deepseek-v3.2"

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
"#,
        )?;
        println!("Created config/sigil.toml");
    }

    let data_dir = dirs::home_dir().unwrap_or_default().join(".sigil");
    std::fs::create_dir_all(&data_dir)?;
    std::fs::create_dir_all(data_dir.join("secrets"))?;
    println!("Created ~/.sigil/");

    println!("\nSigil initialized. Next steps:");
    println!("  1. sigil secrets set OPENROUTER_API_KEY sk-or-...");
    println!("  2. Add projects to config/sigil.toml");
    println!("  3. sigil run \"hello world\"");
    Ok(())
}
