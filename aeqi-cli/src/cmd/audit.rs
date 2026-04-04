use anyhow::Result;
use std::path::PathBuf;

use crate::helpers::load_config;

pub(crate) async fn cmd_audit(
    config_path: &Option<PathBuf>,
    project: Option<&str>,
    task: Option<&str>,
    last: u32,
) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let data_dir = config.data_dir();
    let audit_path = PathBuf::from(&data_dir).join("audit.db");

    if !audit_path.exists() {
        println!("No audit log found at {}", audit_path.display());
        return Ok(());
    }

    let log = aeqi_orchestrator::AuditLog::open(&audit_path)?;

    let events = if let Some(task_id) = task {
        log.query_by_task(task_id)?
    } else if let Some(proj) = project {
        log.query_by_project(proj)?
    } else {
        log.query_recent(last)?
    };

    if events.is_empty() {
        println!("No audit events found.");
        return Ok(());
    }

    for event in &events {
        let task_str = event.task_id.as_deref().unwrap_or("-");
        let agent_str = event.agent.as_deref().unwrap_or("-");
        println!(
            "[{}] {} | {} | task={} agent={} | {}",
            event.timestamp.format("%Y-%m-%d %H:%M:%S"),
            event.project,
            event.decision_type,
            task_str,
            agent_str,
            event.reasoning,
        );
    }

    println!("\n{} events shown.", events.len());
    Ok(())
}
