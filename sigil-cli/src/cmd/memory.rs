use anyhow::Result;
use sigil_core::traits::Memory;
use std::path::PathBuf;

use crate::helpers::{load_config, open_memory};

pub(crate) async fn cmd_recall(
    config_path: &Option<PathBuf>,
    query: &str,
    project_name: Option<&str>,
    top_k: usize,
) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let memory = open_memory(&config, project_name)?;

    let results = memory
        .search(&sigil_core::traits::MemoryQuery::new(query, top_k))
        .await?;

    if results.is_empty() {
        println!("No memories found for: {query}");
    } else {
        for (i, entry) in results.iter().enumerate() {
            let age = chrono::Utc::now() - entry.created_at;
            let age_str = if age.num_days() > 0 {
                format!("{}d ago", age.num_days())
            } else if age.num_hours() > 0 {
                format!("{}h ago", age.num_hours())
            } else {
                format!("{}m ago", age.num_minutes())
            };
            println!(
                "{}. [{}] ({:.2}) {} — {}",
                i + 1,
                age_str,
                entry.score,
                entry.key,
                entry.content
            );
        }
    }
    Ok(())
}

pub(crate) async fn cmd_remember(
    config_path: &Option<PathBuf>,
    key: &str,
    content: &str,
    project_name: Option<&str>,
) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let memory = open_memory(&config, project_name)?;

    let scope = if project_name.is_some() {
        sigil_core::traits::MemoryScope::Domain
    } else {
        sigil_core::traits::MemoryScope::System
    };
    let id = memory
        .store(
            key,
            content,
            sigil_core::traits::MemoryCategory::Fact,
            scope,
            None,
        )
        .await?;
    let scope = project_name.unwrap_or("global");
    println!("Stored memory {id} [{scope}] {key}");
    Ok(())
}
