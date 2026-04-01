use anyhow::Result;
use std::path::PathBuf;

use crate::cli::BlackboardAction;
use crate::helpers::load_config;
use sigil_orchestrator::blackboard::{Blackboard, ClaimResult, EntryDurability};

pub(crate) async fn cmd_blackboard(
    config_path: &Option<PathBuf>,
    action: BlackboardAction,
) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let data_dir = config.data_dir();
    let bb_path = PathBuf::from(&data_dir).join("blackboard.db");
    let orch = &config.orchestrator;
    let bb = Blackboard::open(
        &bb_path,
        orch.blackboard_transient_ttl_hours,
        orch.blackboard_durable_ttl_days,
        orch.blackboard_claim_ttl_hours,
    )?;

    match action {
        BlackboardAction::List { project, limit } => {
            let entries = bb.list_project(&project, limit)?;
            if entries.is_empty() {
                println!("No blackboard entries for project '{project}'.");
                return Ok(());
            }
            for entry in &entries {
                let tags_str = if entry.tags.is_empty() {
                    "-".to_string()
                } else {
                    entry.tags.join(", ")
                };
                println!(
                    "[{}] {} ({:?}) by {} | tags: {} | {}",
                    entry.created_at.format("%Y-%m-%d %H:%M"),
                    entry.key,
                    entry.durability,
                    entry.agent,
                    tags_str,
                    entry.content,
                );
            }
        }
        BlackboardAction::Post {
            project,
            key,
            content,
            tags,
            durability,
        } => {
            let dur = match durability.as_str() {
                "durable" => EntryDurability::Durable,
                _ => EntryDurability::Transient,
            };
            let entry = bb.post(&key, &content, "cli", &project, &tags, dur)?;
            println!(
                "Posted {} (expires {})",
                entry.key,
                entry.expires_at.format("%Y-%m-%d %H:%M")
            );
        }
        BlackboardAction::Query {
            project,
            tags,
            limit,
        } => {
            let entries = bb.query(&project, &tags, limit)?;
            if entries.is_empty() {
                println!("No matching entries.");
                return Ok(());
            }
            for entry in &entries {
                println!("{}: {} (by {})", entry.key, entry.content, entry.agent);
            }
        }
        BlackboardAction::Get { project, key } => match bb.get_by_key(&project, &key)? {
            Some(entry) => {
                println!(
                    "{}: {} (by {}, expires {})",
                    entry.key,
                    entry.content,
                    entry.agent,
                    entry.expires_at.format("%Y-%m-%d %H:%M"),
                );
            }
            None => println!("No entry found for key '{key}'."),
        },
        BlackboardAction::Claim {
            project,
            resource,
            content,
            agent,
        } => {
            let agent = agent.as_deref().unwrap_or("cli");
            match bb.claim(&resource, agent, &project, &content)? {
                ClaimResult::Acquired => {
                    println!("Claimed: {resource}");
                }
                ClaimResult::Renewed => {
                    println!("Renewed claim: {resource}");
                }
                ClaimResult::Held { holder, content } => {
                    println!("Held by {holder}: {content}");
                }
            }
        }
        BlackboardAction::Release {
            project,
            resource,
            agent,
            force,
        } => {
            let agent = agent.as_deref().unwrap_or("cli");
            if bb.release(&resource, agent, &project, force)? {
                println!("Released: {resource}");
            } else {
                println!("No claim found for '{resource}' (or not owned by {agent}).");
            }
        }
        BlackboardAction::Delete { project, key } => {
            if bb.delete_by_key(&project, &key)? {
                println!("Deleted: {key}");
            } else {
                println!("No entry found for key '{key}'.");
            }
        }
    }

    Ok(())
}
