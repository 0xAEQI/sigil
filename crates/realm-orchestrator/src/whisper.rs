use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Typed whisper kinds — compile-time checked message protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhisperKind {
    // Spirit → Scout
    QuestDone { quest_id: String, summary: String },
    QuestBlocked { quest_id: String, question: String, context: String },
    QuestFailed { quest_id: String, error: String },

    // Scout → Shadow
    PatrolReport { domain: String, active: usize, pending: usize },
    SpiritCrashed { domain: String, spirit: String, error: String },
    Escalation { domain: String, quest_id: String, subject: String, description: String, attempts: u32 },

    // Pulse → Shadow
    PulseAlert { domain: String, issues: String },

    // Shadow → Scout
    Resolution { quest_id: String, answer: String },

    // Familiar Council
    FamiliarAdvice { familiar: String, topic: String, advice: String, cost_usd: f64 },

    // Chamber Mode (visible council debate)
    ChamberTopic { topic_id: String, message: String, familiars: Vec<String> },
    ChamberResponse { topic_id: String, familiar: String, response: String },
    ChamberSynthesis { topic_id: String, synthesis: String },
}

impl WhisperKind {
    pub fn subject_tag(&self) -> &'static str {
        match self {
            Self::QuestDone { .. } => "DONE",
            Self::QuestBlocked { .. } => "BLOCKED",
            Self::QuestFailed { .. } => "FAILED",
            Self::PatrolReport { .. } => "PATROL",
            Self::SpiritCrashed { .. } => "SPIRIT_CRASHED",
            Self::Escalation { .. } => "ESCALATE",
            Self::PulseAlert { .. } => "HEARTBEAT_ALERT",
            Self::Resolution { .. } => "RESOLVED",
            Self::FamiliarAdvice { .. } => "COUNCIL_ADVICE",
            Self::ChamberTopic { .. } => "CHAMBER_TOPIC",
            Self::ChamberResponse { .. } => "CHAMBER_RESPONSE",
            Self::ChamberSynthesis { .. } => "CHAMBER_SYNTHESIS",
        }
    }

    pub fn body_text(&self) -> String {
        match self {
            Self::QuestDone { quest_id, summary } =>
                format!("Completed quest {quest_id}: {summary}"),
            Self::QuestBlocked { quest_id, question, context } =>
                format!("Quest {quest_id} blocked: {question}\n\nFull context:\n{context}"),
            Self::QuestFailed { quest_id, error } =>
                format!("Failed quest {quest_id}: {error}"),
            Self::PatrolReport { domain, active, pending } =>
                format!("Domain {domain}: {active} active spirits, {pending} pending quests"),
            Self::SpiritCrashed { domain, spirit, error } =>
                format!("Spirit {spirit} crashed in {domain}: {error}"),
            Self::Escalation { domain, quest_id, subject, description, attempts } =>
                format!(
                    "Domain {domain} needs help resolving a blocker.\n\n\
                     Quest: {quest_id} — {subject}\n\n\
                     Full description:\n{description}\n\n\
                     Blocked after {attempts} resolution attempt(s).",
                ),
            Self::PulseAlert { domain, issues } =>
                format!("Domain {domain} pulse detected issues:\n{issues}"),
            Self::Resolution { quest_id, answer } =>
                format!("Resolution for quest {quest_id}: {answer}"),
            Self::FamiliarAdvice { familiar, topic, advice, cost_usd } =>
                format!("[{familiar}] on \"{topic}\" (${cost_usd:.3}): {advice}"),
            Self::ChamberTopic { topic_id, message, familiars } =>
                format!("Chamber {topic_id}: \"{message}\" — summoning: {}", familiars.join(", ")),
            Self::ChamberResponse { topic_id, familiar, response } =>
                format!("Chamber {topic_id} [{familiar}]: {response}"),
            Self::ChamberSynthesis { topic_id, synthesis } =>
                format!("Chamber {topic_id} synthesis: {synthesis}"),
        }
    }
}

/// A durable message between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whisper {
    pub from: String,
    pub to: String,
    pub kind: WhisperKind,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

impl Whisper {
    pub fn new_typed(from: &str, to: &str, kind: WhisperKind) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            kind,
            timestamp: Utc::now(),
            read: false,
        }
    }
}

/// Indexed, durable mail bus with TTL and bounded queues.
///
/// Improvements over v1:
/// - HashMap<recipient, VecDeque> for O(1) recipient lookup
/// - TTL-based expiry (default 1 hour)
/// - Max queue depth per recipient (default 1000)
/// - Optional JSONL persistence for crash recovery
pub struct WhisperBus {
    queues: Mutex<HashMap<String, VecDeque<Whisper>>>,
    persist_path: Option<PathBuf>,
    ttl_secs: u64,
    max_queue_per_recipient: usize,
}

impl WhisperBus {
    pub fn new() -> Self {
        Self {
            queues: Mutex::new(HashMap::new()),
            persist_path: None,
            ttl_secs: 3600,
            max_queue_per_recipient: 1000,
        }
    }

    /// Create a bus with JSONL persistence.
    pub fn with_persistence(path: PathBuf) -> Self {
        Self {
            queues: Mutex::new(HashMap::new()),
            persist_path: Some(path),
            ttl_secs: 3600,
            max_queue_per_recipient: 1000,
        }
    }

    pub fn set_ttl(&mut self, secs: u64) {
        self.ttl_secs = secs;
    }

    /// Send a message. Prunes expired messages from the target queue.
    pub async fn send(&self, mail: Whisper) {
        let recipient = mail.to.clone();
        let mut queues = self.queues.lock().await;
        let queue = queues.entry(recipient).or_default();

        // Prune expired messages.
        let cutoff = Utc::now() - chrono::Duration::seconds(self.ttl_secs as i64);
        queue.retain(|m| m.timestamp > cutoff);

        // Enforce max depth — drop oldest if full.
        while queue.len() >= self.max_queue_per_recipient {
            queue.pop_front();
        }

        queue.push_back(mail);
    }

    /// Read all unread messages for a recipient. O(1) lookup + O(k) scan.
    pub async fn read(&self, recipient: &str) -> Vec<Whisper> {
        let mut queues = self.queues.lock().await;
        let mut result = Vec::new();

        if let Some(queue) = queues.get_mut(recipient) {
            for msg in queue.iter_mut() {
                if !msg.read {
                    msg.read = true;
                    result.push(msg.clone());
                }
            }
        }

        result
    }

    /// Get all messages across all queues (for status/debugging).
    pub async fn all(&self) -> Vec<Whisper> {
        let queues = self.queues.lock().await;
        queues.values().flat_map(|q| q.iter().cloned()).collect()
    }

    /// Count unread messages for a recipient.
    pub async fn unread_count(&self, recipient: &str) -> usize {
        let queues = self.queues.lock().await;
        queues
            .get(recipient)
            .map(|q| q.iter().filter(|m| !m.read).count())
            .unwrap_or(0)
    }

    /// Total pending (unread) message count across all recipients.
    pub fn pending_count(&self) -> usize {
        self.queues
            .try_lock()
            .map(|queues| {
                queues
                    .values()
                    .flat_map(|q| q.iter())
                    .filter(|m| !m.read)
                    .count()
            })
            .unwrap_or(0)
    }

    /// Drain all unread messages (marks them as read and returns them).
    pub fn drain(&self) -> Vec<Whisper> {
        self.queues
            .try_lock()
            .map(|mut queues| {
                let mut result = Vec::new();
                for queue in queues.values_mut() {
                    for msg in queue.iter_mut() {
                        if !msg.read {
                            msg.read = true;
                            result.push(msg.clone());
                        }
                    }
                }
                result
            })
            .unwrap_or_default()
    }

    /// Persist all unread messages to JSONL for crash recovery.
    pub async fn save(&self) -> Result<()> {
        let path = match &self.persist_path {
            Some(p) => p,
            None => return Ok(()),
        };

        let queues = self.queues.lock().await;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        for queue in queues.values() {
            for msg in queue.iter() {
                if !msg.read {
                    content.push_str(&serde_json::to_string(msg)?);
                    content.push('\n');
                }
            }
        }

        std::fs::write(path, &content)
            .with_context(|| format!("failed to write whisper bus: {}", path.display()))?;

        debug!(path = %path.display(), "whisper bus saved");
        Ok(())
    }

    /// Load unread messages from JSONL. Returns count loaded.
    pub async fn load(&self) -> Result<usize> {
        let path = match &self.persist_path {
            Some(p) => p,
            None => return Ok(0),
        };

        if !path.exists() {
            return Ok(0);
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read whisper bus: {}", path.display()))?;

        let mut queues = self.queues.lock().await;
        let mut count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<Whisper>(line) {
                Ok(msg) => {
                    let queue = queues.entry(msg.to.clone()).or_default();
                    queue.push_back(msg);
                    count += 1;
                }
                Err(e) => {
                    warn!(error = %e, "skipping malformed whisper line");
                }
            }
        }

        debug!(count, "loaded whispers from disk");
        Ok(count)
    }
}

impl Default for WhisperBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_and_read() {
        let bus = WhisperBus::new();
        bus.send(Whisper::new_typed("a", "b", WhisperKind::QuestDone {
            quest_id: "q1".into(),
            summary: "done".into(),
        })).await;

        let msgs = bus.read("b").await;
        assert_eq!(msgs.len(), 1);

        // Second read should return empty (already read).
        let msgs = bus.read("b").await;
        assert_eq!(msgs.len(), 0);
    }

    #[tokio::test]
    async fn test_indexed_recipient() {
        let bus = WhisperBus::new();

        // Send to two different recipients.
        bus.send(Whisper::new_typed("a", "b", WhisperKind::QuestDone {
            quest_id: "q1".into(), summary: "done".into(),
        })).await;
        bus.send(Whisper::new_typed("a", "c", WhisperKind::QuestFailed {
            quest_id: "q2".into(), error: "err".into(),
        })).await;

        // Each recipient only sees their own messages.
        assert_eq!(bus.read("b").await.len(), 1);
        assert_eq!(bus.read("c").await.len(), 1);
        assert_eq!(bus.read("d").await.len(), 0);
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let mut bus = WhisperBus::new();
        bus.set_ttl(1); // 1 second TTL

        // Send and immediately read — should work.
        bus.send(Whisper::new_typed("a", "b", WhisperKind::QuestDone {
            quest_id: "q1".into(), summary: "done".into(),
        })).await;
        assert_eq!(bus.read("b").await.len(), 1);

        // Insert an artificially old message directly.
        {
            let mut queues = bus.queues.lock().await;
            let q = queues.entry("b".to_string()).or_default();
            q.push_back(Whisper {
                from: "a".into(),
                to: "b".into(),
                kind: WhisperKind::QuestDone { quest_id: "old".into(), summary: "old".into() },
                timestamp: Utc::now() - chrono::Duration::seconds(10),
                read: false,
            });
        }

        // Send a new message — should prune the old one.
        bus.send(Whisper::new_typed("a", "b", WhisperKind::QuestDone {
            quest_id: "new".into(), summary: "new".into(),
        })).await;

        let msgs = bus.read("b").await;
        assert_eq!(msgs.len(), 1);
        match &msgs[0].kind {
            WhisperKind::QuestDone { quest_id, .. } => assert_eq!(quest_id, "new"),
            _ => panic!("unexpected kind"),
        }
    }

    #[tokio::test]
    async fn test_max_queue_depth() {
        let bus = WhisperBus {
            queues: Mutex::new(HashMap::new()),
            persist_path: None,
            ttl_secs: 3600,
            max_queue_per_recipient: 3,
        };

        for i in 0..5 {
            bus.send(Whisper::new_typed("a", "b", WhisperKind::QuestDone {
                quest_id: format!("q{i}"), summary: format!("msg{i}"),
            })).await;
        }

        // Should only have the last 3 messages.
        let msgs = bus.read("b").await;
        assert_eq!(msgs.len(), 3);
    }

    #[tokio::test]
    async fn test_persistence_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("whispers.jsonl");

        let bus = WhisperBus::with_persistence(path.clone());
        bus.send(Whisper::new_typed("a", "b", WhisperKind::QuestDone {
            quest_id: "q1".into(), summary: "done".into(),
        })).await;
        bus.save().await.unwrap();

        let bus2 = WhisperBus::with_persistence(path);
        let count = bus2.load().await.unwrap();
        assert_eq!(count, 1);

        let msgs = bus2.read("b").await;
        assert_eq!(msgs.len(), 1);
    }
}
