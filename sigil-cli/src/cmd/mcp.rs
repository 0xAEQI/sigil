use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::helpers::load_config;

#[derive(Debug, Deserialize)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ToolDef {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
}

fn ipc_request_sync(
    data_dir: &std::path::Path,
    request: &serde_json::Value,
) -> Result<serde_json::Value> {
    let sock_path = data_dir.join("rm.sock");
    let stream = std::os::unix::net::UnixStream::connect(&sock_path)?;
    let mut writer = io::BufWriter::new(&stream);
    let mut reader = io::BufReader::new(&stream);

    let mut req_bytes = serde_json::to_vec(request)?;
    req_bytes.push(b'\n');
    writer.write_all(&req_bytes)?;
    writer.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let response: serde_json::Value = serde_json::from_str(&line)?;
    Ok(response)
}

/// Scan a directory for files, returning entries with name, source, content.
fn scan_dir(dir: &std::path::Path, source: &str) -> Vec<serde_json::Value> {
    let mut items = Vec::new();
    if !dir.exists() {
        return items;
    }
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "toml" && ext != "md" {
            continue;
        }
        let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let preview = content.lines().take(3).collect::<Vec<_>>().join(" ").chars().take(120).collect::<String>();
        items.push(serde_json::json!({
            "name": name,
            "source": source,
            "kind": if ext == "toml" { "skill" } else { "doc" },
            "preview": preview,
            "content": content,
        }));
    }
    items
}

pub fn cmd_mcp(config_path: &Option<PathBuf>) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let data_dir = config.data_dir();
    let cwd = std::env::current_dir().unwrap_or_default();

    let tools = vec![
        ToolDef {
            name: "sigil_projects".to_string(),
            description: "List all Sigil projects with repo paths, prefixes, and teams. Use to discover project names and match working directories.".to_string(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
        ToolDef {
            name: "sigil_primer".to_string(),
            description: "Get a project's primer context (SIGIL.md) — architecture, critical rules, build/deploy. This is the essential project brief. Call this before starting work on any project.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Project name"}
                },
                "required": ["project"]
            }),
        },
        ToolDef {
            name: "sigil_skills".to_string(),
            description: "List or retrieve skills — domain knowledge, procedures, and checklists. Skills are reference material loaded into context.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["list", "get"], "default": "list"},
                    "project": {"type": "string", "description": "Filter by project (optional)"},
                    "name": {"type": "string", "description": "Skill name (required for get)"}
                }
            }),
        },
        ToolDef {
            name: "sigil_agents".to_string(),
            description: "List or retrieve agent definitions — autonomous actor templates with model preferences, tool policies, and specialized prompts. Use these when spawning subagents for specific tasks.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["list", "get"], "default": "list"},
                    "project": {"type": "string", "description": "Filter by project (optional)"},
                    "name": {"type": "string", "description": "Agent name (required for get)"}
                }
            }),
        },
        ToolDef {
            name: "sigil_recall".to_string(),
            description: "Search memory for relevant knowledge. Searches within a project's memory by default, or across all projects with scope 'system'.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Project to search"},
                    "query": {"type": "string", "description": "Natural language query"},
                    "limit": {"type": "integer", "description": "Max results", "default": 5},
                    "scope": {"type": "string", "enum": ["domain", "system", "entity"], "default": "domain", "description": "domain = project-level, system = cross-project, entity = per-agent"}
                },
                "required": ["project", "query"]
            }),
        },
        ToolDef {
            name: "sigil_remember".to_string(),
            description: "Store knowledge in memory for future recall.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Project this belongs to"},
                    "key": {"type": "string", "description": "Short slug key"},
                    "content": {"type": "string", "description": "The knowledge to store"},
                    "category": {"type": "string", "enum": ["fact", "procedure", "preference", "context", "evergreen"], "default": "fact"},
                    "scope": {"type": "string", "enum": ["domain", "system", "entity"], "default": "domain", "description": "domain = project-level, system = cross-project, entity = per-agent"},
                    "entity_id": {"type": "string", "description": "Agent name (required when scope is 'entity')"}
                },
                "required": ["project", "key", "content"]
            }),
        },
        ToolDef {
            name: "sigil_status".to_string(),
            description: "Live status: active workers, budget, costs, pending tasks.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Filter to project (optional)"}
                }
            }),
        },
        ToolDef {
            name: "sigil_blackboard".to_string(),
            description: "Ephemeral inter-agent notes. Post breadcrumbs or read coordination state.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["read", "post"]},
                    "project": {"type": "string"},
                    "key": {"type": "string", "description": "Entry key (for post)"},
                    "content": {"type": "string", "description": "Entry content (for post)"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags (for post)"}
                },
                "required": ["action", "project"]
            }),
        },
        ToolDef {
            name: "sigil_create_task".to_string(),
            description: "Create a task in a Sigil project for the team to execute.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string"},
                    "subject": {"type": "string", "description": "Short task title"},
                    "description": {"type": "string", "description": "Detailed description (optional)"}
                },
                "required": ["project", "subject"]
            }),
        },
        ToolDef {
            name: "sigil_close_task".to_string(),
            description: "Close/complete a task by ID.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string"},
                    "reason": {"type": "string"}
                },
                "required": ["task_id"]
            }),
        },
    ];

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = match request.method.as_str() {
            "initialize" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(serde_json::Value::Null),
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "sigil", "version": "4.0.0"}
                })),
                error: None,
            },
            "notifications/initialized" => continue,
            "tools/list" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(serde_json::Value::Null),
                result: Some(serde_json::json!({"tools": tools})),
                error: None,
            },
            "tools/call" => {
                let tool_name = request.params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = request.params.get("arguments").cloned().unwrap_or_default();

                let result = match tool_name {
                    // ── Discovery ──
                    "sigil_projects" => {
                        let projects: Vec<serde_json::Value> = config
                            .projects
                            .iter()
                            .map(|p| {
                                let mut obj = serde_json::json!({
                                    "name": p.name,
                                    "prefix": p.prefix,
                                    "repo": p.repo,
                                });
                                if let Some(ref team) = p.team {
                                    obj["leader"] = serde_json::json!(team.leader);
                                    obj["agents"] = serde_json::json!(team.agents);
                                }
                                obj
                            })
                            .collect();
                        Ok(serde_json::json!({"ok": true, "projects": projects}))
                    }

                    // ── Primer ──
                    "sigil_primer" => {
                        let project = args.get("project").and_then(|v| v.as_str()).unwrap_or("");
                        let project_dir = cwd.join("projects").join(project);
                        let sigil_md = project_dir.join("SIGIL.md");
                        let content = if sigil_md.exists() {
                            std::fs::read_to_string(&sigil_md).unwrap_or_default()
                        } else {
                            let mut parts = Vec::new();
                            let knowledge = project_dir.join("KNOWLEDGE.md");
                            if knowledge.exists() {
                                parts.push(std::fs::read_to_string(&knowledge).unwrap_or_default());
                            }
                            let agents_md = project_dir.join("AGENTS.md");
                            if agents_md.exists() {
                                parts.push(std::fs::read_to_string(&agents_md).unwrap_or_default());
                            }
                            parts.join("\n\n---\n\n")
                        };
                        if content.is_empty() {
                            Ok(serde_json::json!({"ok": false, "error": format!("no primer found for project '{project}'")}))
                        } else {
                            let shared = if project != "shared" {
                                let shared_sigil = cwd.join("projects").join("shared").join("SIGIL.md");
                                if shared_sigil.exists() {
                                    format!("\n\n---\n\n{}", std::fs::read_to_string(&shared_sigil).unwrap_or_default())
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };
                            Ok(serde_json::json!({
                                "ok": true,
                                "project": project,
                                "content": format!("{content}{shared}")
                            }))
                        }
                    }

                    // ── Skills (knowledge, procedures, checklists) ──
                    "sigil_skills" => {
                        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("list");
                        let project_filter = args.get("project").and_then(|v| v.as_str());
                        let name_filter = args.get("name").and_then(|v| v.as_str());

                        let mut all_skills = Vec::new();
                        all_skills.extend(scan_dir(&cwd.join("projects/shared/skills"), "shared"));
                        for entry in std::fs::read_dir(cwd.join("projects")).into_iter().flatten().flatten() {
                            let p = entry.file_name().to_string_lossy().to_string();
                            if p == "shared" { continue; }
                            all_skills.extend(scan_dir(&entry.path().join("skills"), &p));
                        }

                        if action == "get" {
                            let name = name_filter.unwrap_or("");
                            match all_skills.into_iter().find(|s| s.get("name").and_then(|n| n.as_str()).is_some_and(|n| n == name)) {
                                Some(s) => Ok(s),
                                None => Ok(serde_json::json!({"ok": false, "error": format!("skill '{name}' not found")})),
                            }
                        } else {
                            let filtered: Vec<serde_json::Value> = all_skills.into_iter()
                                .filter(|s| {
                                    project_filter.is_none_or(|pf| {
                                        let src = s.get("source").and_then(|v| v.as_str()).unwrap_or("");
                                        src == pf || src == "shared"
                                    })
                                })
                                .map(|s| serde_json::json!({
                                    "name": s["name"], "source": s["source"],
                                    "kind": s["kind"], "preview": s["preview"],
                                }))
                                .collect();
                            Ok(serde_json::json!({"ok": true, "count": filtered.len(), "skills": filtered}))
                        }
                    }

                    // ── Agents (autonomous actor templates) ──
                    "sigil_agents" => {
                        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("list");
                        let project_filter = args.get("project").and_then(|v| v.as_str());
                        let name_filter = args.get("name").and_then(|v| v.as_str());

                        let mut all_agents = Vec::new();
                        all_agents.extend(scan_dir(&cwd.join("projects/shared/agents"), "shared"));
                        for entry in std::fs::read_dir(cwd.join("projects")).into_iter().flatten().flatten() {
                            let p = entry.file_name().to_string_lossy().to_string();
                            if p == "shared" { continue; }
                            all_agents.extend(scan_dir(&entry.path().join("agents"), &p));
                        }

                        if action == "get" {
                            let name = name_filter.unwrap_or("");
                            match all_agents.into_iter().find(|a| a.get("name").and_then(|n| n.as_str()).is_some_and(|n| n == name)) {
                                Some(a) => Ok(a),
                                None => Ok(serde_json::json!({"ok": false, "error": format!("agent '{name}' not found")})),
                            }
                        } else {
                            let filtered: Vec<serde_json::Value> = all_agents.into_iter()
                                .filter(|a| {
                                    project_filter.is_none_or(|pf| {
                                        let src = a.get("source").and_then(|v| v.as_str()).unwrap_or("");
                                        src == pf || src == "shared"
                                    })
                                })
                                .map(|a| serde_json::json!({
                                    "name": a["name"], "source": a["source"],
                                    "kind": a["kind"], "preview": a["preview"],
                                }))
                                .collect();
                            Ok(serde_json::json!({"ok": true, "count": filtered.len(), "agents": filtered}))
                        }
                    }

                    // ── Memory ──
                    "sigil_recall" => {
                        let ipc = serde_json::json!({
                            "cmd": "memories",
                            "project": args.get("project").and_then(|v| v.as_str()).unwrap_or(""),
                            "query": args.get("query").and_then(|v| v.as_str()).unwrap_or(""),
                            "limit": args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5),
                        });
                        ipc_request_sync(&data_dir, &ipc)
                    }
                    "sigil_remember" => {
                        let mut ipc = args.clone();
                        ipc["cmd"] = serde_json::json!("knowledge_store");
                        if !ipc.get("scope").and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty()) {
                            ipc["scope"] = serde_json::json!("domain");
                        }
                        ipc_request_sync(&data_dir, &ipc)
                    }

                    // ── Operations ──
                    "sigil_status" => {
                        let mut ipc = serde_json::json!({"cmd": "status"});
                        if let Some(p) = args.get("project").and_then(|v| v.as_str()) {
                            ipc["project"] = serde_json::json!(p);
                        }
                        ipc_request_sync(&data_dir, &ipc)
                    }
                    "sigil_blackboard" => {
                        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("read");
                        if action == "post" {
                            let mut ipc = args.clone();
                            ipc["cmd"] = serde_json::json!("post_blackboard");
                            ipc["agent"] = serde_json::json!("worker");
                            ipc["durability"] = serde_json::json!("durable");
                            ipc_request_sync(&data_dir, &ipc)
                        } else {
                            let ipc = serde_json::json!({
                                "cmd": "blackboard",
                                "project": args.get("project").and_then(|v| v.as_str()).unwrap_or(""),
                            });
                            ipc_request_sync(&data_dir, &ipc)
                        }
                    }
                    "sigil_create_task" => {
                        let mut ipc = args.clone();
                        ipc["cmd"] = serde_json::json!("create_task");
                        ipc_request_sync(&data_dir, &ipc)
                    }
                    "sigil_close_task" => {
                        let mut ipc = args.clone();
                        ipc["cmd"] = serde_json::json!("close_task");
                        ipc_request_sync(&data_dir, &ipc)
                    }

                    _ => Err(anyhow::anyhow!("unknown tool: {tool_name}")),
                };

                match result {
                    Ok(data) => McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.unwrap_or(serde_json::Value::Null),
                        result: Some(serde_json::json!({
                            "content": [{"type": "text", "text": serde_json::to_string_pretty(&data).unwrap_or_default()}]
                        })),
                        error: None,
                    },
                    Err(e) => McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.unwrap_or(serde_json::Value::Null),
                        result: Some(serde_json::json!({
                            "content": [{"type": "text", "text": format!("Error: {e}")}],
                            "isError": true
                        })),
                        error: None,
                    },
                }
            }
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(serde_json::Value::Null),
                result: Some(serde_json::json!({})),
                error: None,
            },
        };

        let resp_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{resp_json}")?;
        stdout.flush()?;
    }

    Ok(())
}
