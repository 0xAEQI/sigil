use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sigil_tasks::TaskId;

/// A Hook pins a task to a worker. Workers discover their work via hooks on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub task_id: TaskId,
    pub subject: String,
    pub assigned_at: DateTime<Utc>,
}

impl Hook {
    pub fn new(task_id: TaskId, subject: String) -> Self {
        Self {
            task_id,
            subject,
            assigned_at: Utc::now(),
        }
    }
}
