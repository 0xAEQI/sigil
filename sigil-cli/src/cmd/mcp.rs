use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::helpers::load_config;

/// MCP JSON-RPC request.
#[derive(Debug, Deserialize)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

/// MCP JSON-RPC response.
#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

/// Tool definition for MCP.
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

pub fn cmd_mcp(config_path: &Option<PathBuf>) -> Result<()> {
    let (config, _) = load_config(config_path)?;
    let data_dir = config.data_dir();

    let tools = vec![
        ToolDef {
            name: "sigil_create_task".to_string(),
            description: "Create a new task in a Sigil project. The task will be assigned to the project's team and executed by a worker.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Project name (e.g., algostaking, sigil, riftdecks-shop)"},
                    "subject": {"type": "string", "description": "Short task title describing what needs to be done"},
                    "description": {"type": "string", "description": "Detailed description of the task (optional)"}
                },
                "required": ["project", "subject"]
            }),
        },
        ToolDef {
            name: "sigil_recall".to_string(),
            description: "Search Sigil's memory for relevant knowledge about a topic. Returns learned insights from past task executions, notes, and reflections.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Project to search memories in"},
                    "query": {"type": "string", "description": "What to search for (natural language)"},
                    "limit": {"type": "integer", "description": "Max results (default 5)", "default": 5}
                },
                "required": ["project", "query"]
            }),
        },
        ToolDef {
            name: "sigil_remember".to_string(),
            description: "Store a new piece of knowledge in Sigil's memory. Use this to save important insights, decisions, or facts learned during execution.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Project this knowledge belongs to"},
                    "key": {"type": "string", "description": "Short slug key (e.g., 'signal-latency-target')"},
                    "content": {"type": "string", "description": "The knowledge to store"},
                    "category": {"type": "string", "enum": ["fact", "procedure", "preference", "context", "evergreen"], "default": "fact"}
                },
                "required": ["project", "key", "content"]
            }),
        },
        ToolDef {
            name: "sigil_status".to_string(),
            description: "Get the current status of Sigil projects, workers, costs, and recent activity.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project": {"type": "string", "description": "Filter to a specific project (optional)"}
                }
            }),
        },
        ToolDef {
            name: "sigil_blackboard".to_string(),
            description: "Read or post to the shared blackboard. The blackboard is ephemeral inter-agent knowledge visible to all workers on a project.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["read", "post"], "description": "Read entries or post new entry"},
                    "project": {"type": "string", "description": "Project name"},
                    "key": {"type": "string", "description": "Entry key (for post)"},
                    "content": {"type": "string", "description": "Entry content (for post)"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tags (for post)"}
                },
                "required": ["action", "project"]
            }),
        },
        ToolDef {
            name: "sigil_close_task".to_string(),
            description: "Close/complete a task by its ID.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID (e.g., as-001, sg-002)"},
                    "reason": {"type": "string", "description": "Reason for closing"}
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
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "sigil",
                        "version": "3.2.0"
                    }
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
                let tool_name = request
                    .params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let args = request.params.get("arguments").cloned().unwrap_or_default();

                let result = match tool_name {
                    "sigil_create_task" => {
                        let mut ipc = args.clone();
                        ipc["cmd"] = serde_json::json!("create_task");
                        ipc_request_sync(&data_dir, &ipc)
                    }
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
                        if ipc.get("scope").is_none() {
                            ipc["scope"] = serde_json::json!("domain");
                        }
                        ipc_request_sync(&data_dir, &ipc)
                    }
                    "sigil_status" => {
                        let project = args.get("project").and_then(|v| v.as_str());
                        let mut ipc = serde_json::json!({"cmd": "status"});
                        if let Some(p) = project {
                            ipc["project"] = serde_json::json!(p);
                        }
                        ipc_request_sync(&data_dir, &ipc)
                    }
                    "sigil_blackboard" => {
                        let action = args
                            .get("action")
                            .and_then(|v| v.as_str())
                            .unwrap_or("read");
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
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&data).unwrap_or_default()
                            }]
                        })),
                        error: None,
                    },
                    Err(e) => McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.unwrap_or(serde_json::Value::Null),
                        result: Some(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {e}")
                            }],
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
