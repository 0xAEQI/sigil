use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::executor::TaskOutcome;
use crate::verification::VerificationResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePhase {
    Prime,
    Frame,
    Act,
    Verify,
    Commit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSessionStatus {
    Created,
    Running,
    Completed,
    Blocked,
    Handoff,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub id: String,
    pub phase: RuntimePhase,
    pub summary: String,
    pub status: StepStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    File,
    GitCommit,
    GitBranch,
    Worktree,
    Checkpoint,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub label: String,
    pub reference: String,
}

impl Artifact {
    pub fn new(kind: ArtifactKind, label: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            kind,
            label: label.into(),
            reference: reference.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerificationReport {
    pub checks_run: Vec<String>,
    pub confidence: Option<f32>,
    pub approved: Option<bool>,
    pub warnings: Vec<String>,
    pub evidence_summary: Vec<String>,
}

impl From<&VerificationResult> for VerificationReport {
    fn from(value: &VerificationResult) -> Self {
        let mut checks_run = Vec::new();
        let mut warnings = value.suggestions.clone();
        let mut evidence_summary = Vec::new();

        if !value.signals.is_empty() {
            checks_run.push(format!("signals: {}", value.signals.len()));
        }

        if let Some(ref evidence) = value.evidence {
            if evidence.test_exit_code.is_some() {
                checks_run.push("test_runner".to_string());
            }
            if !evidence.files_changed.is_empty() {
                checks_run.push("git_diff".to_string());
                evidence_summary.push(format!(
                    "files changed: {}",
                    evidence.files_changed.join(", ")
                ));
            }
            if let Some(code) = evidence.test_exit_code {
                evidence_summary.push(format!("test exit code: {code}"));
            }
            if evidence.lines_added > 0 || evidence.lines_removed > 0 {
                evidence_summary.push(format!(
                    "diff stats: +{} -{}",
                    evidence.lines_added, evidence.lines_removed
                ));
            }
        }

        if !value.approved {
            warnings.push(value.reason.clone());
        }

        Self {
            checks_run,
            confidence: Some(value.confidence),
            approved: Some(value.approved),
            warnings,
            evidence_summary,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeOutcomeStatus {
    Done,
    Blocked,
    Handoff,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOutcome {
    pub status: RuntimeOutcomeStatus,
    pub summary: String,
    pub reason: Option<String>,
    pub next_action: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub verification: Option<VerificationReport>,
}

impl RuntimeOutcome {
    pub fn from_task_outcome(outcome: &TaskOutcome, artifacts: Vec<Artifact>) -> Self {
        match outcome {
            TaskOutcome::Done(summary) => Self {
                status: RuntimeOutcomeStatus::Done,
                summary: summary.clone(),
                reason: None,
                next_action: None,
                artifacts,
                verification: None,
            },
            TaskOutcome::Blocked {
                question,
                full_text,
            } => Self {
                status: RuntimeOutcomeStatus::Blocked,
                summary: full_text.clone(),
                reason: Some(question.clone()),
                next_action: Some("await_operator_input".to_string()),
                artifacts,
                verification: None,
            },
            TaskOutcome::Handoff { checkpoint } => Self {
                status: RuntimeOutcomeStatus::Handoff,
                summary: checkpoint.clone(),
                reason: Some(checkpoint.clone()),
                next_action: Some("resume_from_checkpoint".to_string()),
                artifacts,
                verification: None,
            },
            TaskOutcome::Failed(error) => Self {
                status: RuntimeOutcomeStatus::Failed,
                summary: error.clone(),
                reason: Some(error.clone()),
                next_action: Some("inspect_failure".to_string()),
                artifacts,
                verification: None,
            },
        }
    }

    pub fn artifact_refs(&self) -> Vec<String> {
        self.artifacts
            .iter()
            .map(|artifact| artifact.reference.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSession {
    pub session_id: String,
    pub task_id: String,
    pub worker_id: String,
    pub project: String,
    pub model: Option<String>,
    pub status: RuntimeSessionStatus,
    pub phase: RuntimePhase,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub checkpoint_refs: Vec<String>,
    pub steps: Vec<StepRecord>,
}

impl RuntimeSession {
    pub fn new(
        task_id: impl Into<String>,
        worker_id: impl Into<String>,
        project: impl Into<String>,
        model: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id: format!("rt-{}", Uuid::new_v4().simple()),
            task_id: task_id.into(),
            worker_id: worker_id.into(),
            project: project.into(),
            model,
            status: RuntimeSessionStatus::Created,
            phase: RuntimePhase::Prime,
            started_at: now,
            updated_at: now,
            checkpoint_refs: Vec::new(),
            steps: Vec::new(),
        }
    }

    pub fn mark_phase(&mut self, phase: RuntimePhase, summary: impl Into<String>) {
        let now = Utc::now();
        self.phase = phase;
        self.updated_at = now;
        if self.status == RuntimeSessionStatus::Created {
            self.status = RuntimeSessionStatus::Running;
        }
        self.steps.push(StepRecord {
            id: format!("step-{}", self.steps.len() + 1),
            phase,
            summary: summary.into(),
            status: StepStatus::Completed,
            timestamp: now,
        });
    }

    pub fn add_checkpoint_ref(&mut self, reference: impl Into<String>) {
        self.checkpoint_refs.push(reference.into());
        self.updated_at = Utc::now();
    }

    pub fn finish(&mut self, outcome: &RuntimeOutcome) {
        self.phase = RuntimePhase::Commit;
        self.updated_at = Utc::now();
        self.status = match outcome.status {
            RuntimeOutcomeStatus::Done => RuntimeSessionStatus::Completed,
            RuntimeOutcomeStatus::Blocked => RuntimeSessionStatus::Blocked,
            RuntimeOutcomeStatus::Handoff => RuntimeSessionStatus::Handoff,
            RuntimeOutcomeStatus::Failed => RuntimeSessionStatus::Failed,
        };
        self.steps.push(StepRecord {
            id: format!("step-{}", self.steps.len() + 1),
            phase: RuntimePhase::Commit,
            summary: "Committed runtime outcome".to_string(),
            status: StepStatus::Completed,
            timestamp: self.updated_at,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExecution {
    pub session: RuntimeSession,
    pub outcome: RuntimeOutcome,
}
