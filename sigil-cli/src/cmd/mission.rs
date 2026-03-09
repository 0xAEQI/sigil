use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::cli::MissionAction;
use crate::helpers::{load_config, open_tasks_for_project, project_name_for_prefix};

pub(crate) async fn cmd_mission(
    config_path: &Option<PathBuf>,
    action: MissionAction,
) -> Result<()> {
    match action {
        MissionAction::Create {
            name,
            project,
            description,
            decompose: _decompose,
        } => {
            let (config, _) = load_config(config_path)?;
            let prefix = if let Some(pcfg) = config.project(&project) {
                pcfg.prefix.clone()
            } else if let Some(acfg) = config.agent(&project) {
                acfg.prefix.clone()
            } else {
                anyhow::bail!("project or agent not found: {project}");
            };

            let mut store = open_tasks_for_project(&project)?;
            let mut mission = store.create_mission(&prefix, &name)?;

            if !description.is_empty() {
                mission = store.update_mission(&mission.id, |m| {
                    m.description = description.clone();
                })?;
            }

            println!("Created mission {} — {}", mission.id, mission.name);
            Ok(())
        }
        MissionAction::List { project, all } => {
            let (config, _) = load_config(config_path)?;

            let projects: Vec<&str> = if let Some(ref name) = project {
                vec![name.as_str()]
            } else {
                config.projects.iter().map(|r| r.name.as_str()).collect()
            };

            for name in projects {
                if let Ok(store) = open_tasks_for_project(name) {
                    let missions = if all {
                        store.missions(None)
                    } else {
                        store.active_missions(None)
                    };

                    if missions.is_empty() {
                        continue;
                    }

                    println!("=== {} ===", name);
                    for m in missions {
                        let task_count = store.mission_tasks(&m.id).len();
                        let done_count = store
                            .mission_tasks(&m.id)
                            .iter()
                            .filter(|t| t.is_closed())
                            .count();
                        println!(
                            "  {} [{}] {} — {}/{} tasks done",
                            m.id, m.status, m.name, done_count, task_count
                        );
                    }
                }
            }
            Ok(())
        }
        MissionAction::Status { id } => {
            let (config, _) = load_config(config_path)?;
            let prefix = id.split('-').next().unwrap_or("");
            let project_name = project_name_for_prefix(&config, prefix)
                .context(format!("no project with prefix '{prefix}'"))?;

            let store = open_tasks_for_project(&project_name)?;
            let mission = store
                .get_mission(&id)
                .ok_or_else(|| anyhow::anyhow!("mission not found: {id}"))?;

            println!("Mission: {} — {}", mission.id, mission.name);
            println!("Status: {}", mission.status);
            if !mission.description.is_empty() {
                println!("Description: {}", mission.description);
            }

            let tasks = store.mission_tasks(&id);
            if tasks.is_empty() {
                println!("No tasks assigned to this mission.");
            } else {
                let done = tasks.iter().filter(|t| t.is_closed()).count();
                println!("Progress: {}/{} tasks done", done, tasks.len());
                for t in &tasks {
                    let assignee = t.assignee.as_deref().unwrap_or("-");
                    println!(
                        "  {} [{}] {} — assignee={}",
                        t.id, t.status, t.subject, assignee
                    );
                }
            }
            Ok(())
        }
        MissionAction::Close { id } => {
            let (config, _) = load_config(config_path)?;
            let prefix = id.split('-').next().unwrap_or("");
            let project_name = project_name_for_prefix(&config, prefix)
                .context(format!("no project with prefix '{prefix}'"))?;

            let mut store = open_tasks_for_project(&project_name)?;
            let mission = store.close_mission(&id)?;
            println!("Closed mission {} — {}", mission.id, mission.name);
            Ok(())
        }
    }
}
