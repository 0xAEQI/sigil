use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use realm_core::traits::{Channel, ToolResult, ToolSpec};
use realm_core::traits::Tool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::whisper::{Whisper, WhisperBus, WhisperKind};
use crate::registry::DomainRegistry;
use realm_core::traits::{Memory, MemoryCategory, MemoryQuery, MemoryScope};

/// Tool for querying rig health, bead counts, and worker states.
pub struct RigStatusTool {
    registry: Arc<DomainRegistry>,
}

impl RigStatusTool {
    pub fn new(registry: Arc<DomainRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Tool for RigStatusTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let rig_filter = args.get("rig").and_then(|v| v.as_str());

        let status = self.registry.status().await;
        let mut output = String::new();

        for ds in &status.domains {
            if let Some(filter) = rig_filter
                && ds.name != filter
            {
                continue;
            }
            output.push_str(&format!(
                "{}: {} open, {} ready | spirits: {} idle, {} working, {} bonded\n",
                ds.name, ds.open_quests, ds.ready_quests,
                ds.spirits_idle, ds.spirits_working, ds.spirits_bonded,
            ));
        }

        if rig_filter.is_none() {
            output.push_str(&format!("\nUnread whispers: {}\n", status.unread_whispers));
        }

        if output.is_empty() {
            if let Some(filter) = rig_filter {
                return Ok(ToolResult::error(format!("Rig not found: {filter}")));
            }
            output = "No rigs registered.\n".to_string();
        }

        Ok(ToolResult::success(output))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "rig_status".to_string(),
            description: "Get rig health, bead counts, and worker states. Optionally filter by rig name.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rig": { "type": "string", "description": "Optional rig name to filter (omit for all rigs)" }
                }
            }),
        }
    }

    fn name(&self) -> &str { "rig_status" }
}

/// Tool for assigning a task (bead) to a target rig.
pub struct RigAssignTool {
    registry: Arc<DomainRegistry>,
}

impl RigAssignTool {
    pub fn new(registry: Arc<DomainRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Tool for RigAssignTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let rig = args.get("rig")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing rig"))?;
        let subject = args.get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing subject"))?;
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match self.registry.assign(rig, subject, description).await {
            Ok(bead) => Ok(ToolResult::success(format!(
                "Assigned {} [{}] {} to rig '{}'",
                bead.id, bead.priority, bead.subject, rig
            ))),
            Err(e) => Ok(ToolResult::error(format!("Failed to assign: {e}"))),
        }
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "rig_assign".to_string(),
            description: "Assign a task to a specific rig by creating a bead on it.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rig": { "type": "string", "description": "Target rig name. Use rig_list to discover available rigs." },
                    "subject": { "type": "string", "description": "Task title" },
                    "description": { "type": "string", "description": "Detailed task description" }
                },
                "required": ["rig", "subject"]
            }),
        }
    }

    fn name(&self) -> &str { "rig_assign" }
}

/// Tool for listing all registered rigs with metadata.
pub struct RigListTool {
    registry: Arc<DomainRegistry>,
}

impl RigListTool {
    pub fn new(registry: Arc<DomainRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Tool for RigListTool {
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let domains = self.registry.domains_info().await;

        if domains.is_empty() {
            return Ok(ToolResult::success("No domains registered."));
        }

        let mut output = String::new();
        for d in &domains {
            output.push_str(&format!(
                "{} (prefix: {}, model: {}, max_workers: {})\n",
                d["name"].as_str().unwrap_or("?"),
                d["prefix"].as_str().unwrap_or("?"),
                d["model"].as_str().unwrap_or("?"),
                d["max_workers"].as_u64().unwrap_or(0),
            ));
        }
        Ok(ToolResult::success(output))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "rig_list".to_string(),
            description: "List all registered rigs with their prefix, model, and worker count.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn name(&self) -> &str { "rig_list" }
}

/// Tool for reading unread mail addressed to the familiar.
pub struct MailReadTool {
    whisper_bus: Arc<WhisperBus>,
}

impl MailReadTool {
    pub fn new(whisper_bus: Arc<WhisperBus>) -> Self {
        Self { whisper_bus }
    }
}

#[async_trait]
impl Tool for MailReadTool {
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let messages = self.whisper_bus.read("familiar").await;

        if messages.is_empty() {
            return Ok(ToolResult::success("No unread mail."));
        }

        let mut output = String::new();
        for m in &messages {
            output.push_str(&format!(
                "[{}] from={} subject={}\n{}\n\n",
                m.timestamp.format("%H:%M:%S"),
                m.from, m.kind.subject_tag(), m.kind.body_text(),
            ));
        }
        Ok(ToolResult::success(output))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "whisper_read".to_string(),
            description: "Read all unread mail addressed to the familiar.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn name(&self) -> &str { "whisper_read" }
}

/// Tool for sending mail through the bus.
pub struct MailSendTool {
    whisper_bus: Arc<WhisperBus>,
}

impl MailSendTool {
    pub fn new(whisper_bus: Arc<WhisperBus>) -> Self {
        Self { whisper_bus }
    }
}

#[async_trait]
impl Tool for MailSendTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let to = args.get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing to"))?;
        let subject = args.get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing subject"))?;
        let body = args.get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let quest_id = args.get("quest_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let kind = match subject.to_uppercase().as_str() {
            "RESOLVED" => WhisperKind::Resolution {
                quest_id,
                answer: body.to_string(),
            },
            _ => WhisperKind::Resolution {
                quest_id,
                answer: format!("[{}] {}", subject, body),
            },
        };

        self.whisper_bus.send(Whisper::new_typed("familiar", to, kind)).await;
        Ok(ToolResult::success(format!("Whisper sent to '{to}': {subject}")))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "whisper_send".to_string(),
            description: "Send a mail message to another rig or agent through the mail bus.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": { "type": "string", "description": "Recipient (rig name or agent name)" },
                    "subject": { "type": "string", "description": "Whisper subject" },
                    "body": { "type": "string", "description": "Whisper body" }
                },
                "required": ["to", "subject"]
            }),
        }
    }

    fn name(&self) -> &str { "whisper_send" }
}

/// Tool for listing all unblocked beads across all rigs.
pub struct AllReadyTool {
    registry: Arc<DomainRegistry>,
}

impl AllReadyTool {
    pub fn new(registry: Arc<DomainRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl Tool for AllReadyTool {
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let ready = self.registry.all_ready().await;

        if ready.is_empty() {
            return Ok(ToolResult::success("No ready work across any rig."));
        }

        let mut output = String::new();
        for (domain_name, bead) in &ready {
            output.push_str(&format!(
                "[{}] {} [{}] {} — {}\n",
                domain_name, bead.id, bead.priority, bead.subject,
                if bead.description.is_empty() { "(no description)" } else { &bead.description }
            ));
        }
        Ok(ToolResult::success(output))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "all_ready".to_string(),
            description: "List all unblocked beads across all rigs that are ready for work.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn name(&self) -> &str { "all_ready" }
}

/// Tool for replying to a channel message (Telegram, Discord, etc.)
pub struct ChannelReplyTool {
    channels: Arc<RwLock<HashMap<String, Arc<dyn Channel>>>>,
}

impl ChannelReplyTool {
    pub fn new(channels: Arc<RwLock<HashMap<String, Arc<dyn Channel>>>>) -> Self {
        Self { channels }
    }
}
#[async_trait]
impl Tool for ChannelReplyTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let channel_name = args.get("channel")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing channel"))?;
        let text = args.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing text"))?;

        // Extract optional reaction emoji
        let reaction = args.get("reaction").and_then(|v| v.as_str());

        // Build metadata from args (pass through chat_id etc.)
        let mut metadata = serde_json::Map::new();
        if let Some(chat_id) = args.get("chat_id") {
            metadata.insert("chat_id".to_string(), chat_id.clone());
        }
        if let Some(message_id) = args.get("message_id") {
            metadata.insert("message_id".to_string(), message_id.clone());
        }

        let channels = self.channels.read().await;
        let channel = channels.get(channel_name)
            .ok_or_else(|| anyhow::anyhow!("channel not found: {channel_name}"))?;

        let outgoing = realm_core::traits::OutgoingMessage {
            channel: channel_name.to_string(),
            recipient: String::new(),
            text: text.to_string(),
            metadata: serde_json::Value::Object(metadata),
        };

        channel.send(outgoing).await?;

        // Add reaction if specified
        if let Some(emoji) = reaction {
            let chat_id = args.get("chat_id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("missing chat_id for reaction"))?;
            let message_id = args.get("message_id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("missing message_id for reaction"))?;
            
            channel.react(chat_id, message_id, emoji).await?;
        }

        Ok(ToolResult::success(format!("Reply sent via {channel_name}")))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "channel_reply".to_string(),
            description: "Send a reply through a messaging channel (Telegram, Discord, etc.)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel": { "type": "string", "description": "Channel name (telegram, discord, slack)" },
                    "chat_id": { "type": "integer", "description": "Chat ID to reply to" },
                    "text": { "type": "string", "description": "Message text to send" }
                },
                "required": ["channel", "chat_id", "text"]
            }),
        }
    }

    fn name(&self) -> &str { "channel_reply" }
}

/// Tool that surfaces Claude Code session costs, OpenRouter key usage, and
/// per-rig worker execution costs aggregated from `~/.sigil/usage.jsonl`.
pub struct UsageStatsTool {
    api_key: Option<String>,
}

impl UsageStatsTool {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl Tool for UsageStatsTool {
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let mut output = String::new();

        output.push_str("**Claude Code Lifetime Usage**\n");
        match collect_claude_usage().await {
            Ok(s) => output.push_str(&s),
            Err(_) => output.push_str("  (not available — ~/.claude.json missing)\n"),
        }
        output.push('\n');

        output.push_str("**OpenRouter API Key**\n");
        match &self.api_key {
            Some(key) => match collect_openrouter_usage(key).await {
                Ok(s) => output.push_str(&s),
                Err(e) => output.push_str(&format!("  Error fetching key info: {e}\n")),
            },
            None => output.push_str("  (API key not configured)\n"),
        }
        output.push('\n');

        output.push_str("**Spirit Executions (all time)**\n");
        match collect_worker_usage().await {
            Ok(s) => output.push_str(&s),
            Err(_) => output.push_str("  (no executions logged yet)\n"),
        }

        Ok(ToolResult::success(output))
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "usage_stats".to_string(),
            description: "Get Claude Code session costs, OpenRouter API key credit usage, and per-rig worker execution costs.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    fn name(&self) -> &str { "usage_stats" }
}

/// Read Claude Code's ~/.claude.json and return a formatted usage summary.
pub async fn collect_claude_usage() -> Result<String> {
    let path = dirs::home_dir()
        .context("no home directory")?
        .join(".claude.json");

    let content = tokio::fs::read_to_string(&path).await
        .context("failed to read ~/.claude.json")?;

    let v: serde_json::Value = serde_json::from_str(&content)
        .context("failed to parse ~/.claude.json")?;

    let mut out = String::new();

    if let Some(total) = v.get("lastCost").and_then(|c| c.as_f64()) {
        out.push_str(&format!("  Total: ${total:.2}\n"));
    }

    if let Some(model_usage) = v.get("lastModelUsage").and_then(|m| m.as_object()) {
        let mut models: Vec<_> = model_usage.iter().collect();
        models.sort_by(|a, b| {
            let ac = a.1.get("cost").and_then(|c| c.as_f64()).unwrap_or(0.0);
            let bc = b.1.get("cost").and_then(|c| c.as_f64()).unwrap_or(0.0);
            bc.partial_cmp(&ac).unwrap_or(std::cmp::Ordering::Equal)
        });
        for (model, usage) in &models {
            let input = usage.get("inputTokens").and_then(|t| t.as_u64()).unwrap_or(0);
            let output = usage.get("outputTokens").and_then(|t| t.as_u64()).unwrap_or(0);
            let cache_read = usage.get("cacheReadInputTokens").and_then(|t| t.as_u64()).unwrap_or(0);
            let cost = usage.get("cost").and_then(|c| c.as_f64()).unwrap_or(0.0);
            out.push_str(&format!(
                "  {model}: {}k in / {}k out (cache: {}k read) — ${cost:.2}\n",
                input / 1000,
                output / 1000,
                cache_read / 1000,
            ));
        }
    }

    Ok(out)
}

/// Query OpenRouter /api/v1/auth/key and return a formatted credit summary.
pub async fn collect_openrouter_usage(api_key: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let resp = client
        .get("https://openrouter.ai/api/v1/auth/key")
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .context("request failed")?;

    let v: serde_json::Value = resp.json().await.context("failed to parse response")?;
    let data = v.get("data").context("no data field in response")?;

    let usage = data.get("usage").and_then(|u| u.as_f64()).unwrap_or(0.0);
    let limit = data.get("limit").and_then(|l| l.as_f64());
    let limit_str = match limit {
        Some(l) => format!("${l:.2}"),
        None => "unlimited".to_string(),
    };

    let mut out = format!("  Spent: ${usage:.4} / {limit_str}\n");

    if let Some(rl) = data.get("rate_limit") {
        let requests = rl.get("requests").and_then(|r| r.as_u64()).unwrap_or(0);
        let interval = rl.get("interval").and_then(|i| i.as_str()).unwrap_or("?");
        out.push_str(&format!("  Rate limit: {requests} req/{interval}\n"));
    }

    Ok(out)
}

/// Read ~/.sigil/usage.jsonl and return a per-rig cost summary.
pub async fn collect_worker_usage() -> Result<String> {
    let path = usage_log_path();

    let content = tokio::fs::read_to_string(&path).await
        .context("no usage log yet")?;

    let mut rig_totals: HashMap<String, (f64, usize)> = HashMap::new();
    for line in content.lines() {
        if line.is_empty() { continue; }
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            let rig = entry.get("rig")
                .and_then(|r| r.as_str())
                .unwrap_or("unknown")
                .to_string();
            let cost = entry.get("cost_usd").and_then(|c| c.as_f64()).unwrap_or(0.0);
            let e = rig_totals.entry(rig).or_insert((0.0, 0));
            e.0 += cost;
            e.1 += 1;
        }
    }

    if rig_totals.is_empty() {
        return Ok("  (no executions logged yet)\n".to_string());
    }

    let mut rigs: Vec<_> = rig_totals.iter().collect();
    rigs.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = String::new();
    let total_cost: f64 = rigs.iter().map(|(_, (c, _))| c).sum();
    for (rig, (cost, count)) in &rigs {
        out.push_str(&format!("  {rig}: ${cost:.4} ({count} runs)\n"));
    }
    out.push_str(&format!("  Total: ${total_cost:.4}\n"));

    Ok(out)
}

pub fn usage_log_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".sigil")
        .join("usage.jsonl")
}

pub struct MemoryStoreTool {
    memory: Arc<dyn Memory>,
}

impl MemoryStoreTool {
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }
}

#[async_trait]
impl Tool for MemoryStoreTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let key = args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing key"))?;
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing content"))?;
        let scope = match args.get("scope").and_then(|v| v.as_str()) {
            Some("realm") => MemoryScope::Realm,
            Some("companion") => MemoryScope::Companion,
            _ => MemoryScope::Domain,
        };
        let category = match args.get("category").and_then(|v| v.as_str()) {
            Some("procedure") => MemoryCategory::Procedure,
            Some("preference") => MemoryCategory::Preference,
            Some("context") => MemoryCategory::Context,
            Some("evergreen") => MemoryCategory::Evergreen,
            _ => MemoryCategory::Fact,
        };
        let companion_id = args.get("companion_id").and_then(|v| v.as_str());

        match self.memory.store(key, content, category, scope, companion_id).await {
            Ok(id) => Ok(ToolResult::success(format!("Stored memory {id} [{scope}] {key}"))),
            Err(e) => Ok(ToolResult::error(format!("Failed to store: {e}"))),
        }
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "memory_store".to_string(),
            description: "Store a memory with semantic embeddings for later recall. Use for facts, preferences, patterns, and context worth remembering.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string", "description": "Short label for the memory (e.g. 'jwt-auth-preference')" },
                    "content": { "type": "string", "description": "The memory content to store" },
                    "scope": { "type": "string", "enum": ["domain", "realm", "companion"], "description": "Memory scope (default: domain)" },
                    "category": { "type": "string", "enum": ["fact", "procedure", "preference", "context", "evergreen"], "description": "Memory category (default: fact)" },
                    "companion_id": { "type": "string", "description": "Companion ID for companion-scoped memories" }
                },
                "required": ["key", "content"]
            }),
        }
    }

    fn name(&self) -> &str { "memory_store" }
}

pub struct MemoryRecallTool {
    memory: Arc<dyn Memory>,
}

impl MemoryRecallTool {
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }
}

#[async_trait]
impl Tool for MemoryRecallTool {
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let query_text = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing query"))?;
        let top_k = args.get("top_k")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let mut query = MemoryQuery::new(query_text, top_k);

        if let Some(scope) = args.get("scope").and_then(|v| v.as_str()) {
            query.scope = Some(match scope {
                "realm" => MemoryScope::Realm,
                "companion" => MemoryScope::Companion,
                _ => MemoryScope::Domain,
            });
        }
        if let Some(cid) = args.get("companion_id").and_then(|v| v.as_str()) {
            query = query.with_companion(cid);
        }

        match self.memory.search(&query).await {
            Ok(results) if results.is_empty() => {
                Ok(ToolResult::success(format!("No memories found for: {query_text}")))
            }
            Ok(results) => {
                let mut output = String::new();
                for (i, entry) in results.iter().enumerate() {
                    let age = chrono::Utc::now() - entry.created_at;
                    let age_str = if age.num_days() > 0 {
                        format!("{}d ago", age.num_days())
                    } else if age.num_hours() > 0 {
                        format!("{}h ago", age.num_hours())
                    } else {
                        format!("{}m ago", age.num_minutes())
                    };
                    output.push_str(&format!(
                        "{}. [{}] ({:.2}) {} — {}\n",
                        i + 1, age_str, entry.score, entry.key, entry.content,
                    ));
                }
                Ok(ToolResult::success(output))
            }
            Err(e) => Ok(ToolResult::error(format!("Search failed: {e}"))),
        }
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "memory_recall".to_string(),
            description: "Search memories using semantic similarity + keyword matching. Returns the most relevant memories ranked by hybrid score.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language search query" },
                    "top_k": { "type": "integer", "description": "Max results to return (default: 5)" },
                    "scope": { "type": "string", "enum": ["domain", "realm", "companion"], "description": "Filter by scope" },
                    "companion_id": { "type": "string", "description": "Filter to specific companion's memories" }
                },
                "required": ["query"]
            }),
        }
    }

    fn name(&self) -> &str { "memory_recall" }
}

/// Build orchestration tools for the familiar rig.
///
/// NOTE: `channel_reply` is intentionally excluded. The familiar's final text output
/// is automatically delivered to the originating channel by the daemon's polling loop.
/// Including `channel_reply` causes double-delivery: the tool sends once, and the
/// bead's closed_reason (the LLM's confirmation text) gets sent again.
pub fn build_orchestration_tools(
    registry: Arc<DomainRegistry>,
    whisper_bus: Arc<WhisperBus>,
    _channels: Arc<RwLock<HashMap<String, Arc<dyn Channel>>>>,
    api_key: Option<String>,
    memory: Option<Arc<dyn Memory>>,
) -> Vec<Arc<dyn Tool>> {
    let mut tools: Vec<Arc<dyn Tool>> = vec![
        Arc::new(RigStatusTool::new(registry.clone())),
        Arc::new(RigAssignTool::new(registry.clone())),
        Arc::new(RigListTool::new(registry.clone())),
        Arc::new(MailReadTool::new(whisper_bus.clone())),
        Arc::new(MailSendTool::new(whisper_bus)),
        Arc::new(AllReadyTool::new(registry)),
        Arc::new(UsageStatsTool::new(api_key)),
    ];

    if let Some(mem) = memory {
        tools.push(Arc::new(MemoryStoreTool::new(mem.clone())));
        tools.push(Arc::new(MemoryRecallTool::new(mem)));
    }

    tools
}
