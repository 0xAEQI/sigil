use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use realm_quests::QuestId;

/// A Bond pins a bead to a worker. GUPP: "If there is work on your hook,
/// you MUST run it." Spirits discover their work via hooks on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    pub quest_id: QuestId,
    pub subject: String,
    pub assigned_at: DateTime<Utc>,
}

impl Bond {
    pub fn new(quest_id: QuestId, subject: String) -> Self {
        Self {
            quest_id,
            subject,
            assigned_at: Utc::now(),
        }
    }
}
