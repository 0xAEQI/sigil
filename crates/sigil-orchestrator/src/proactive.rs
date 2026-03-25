//! Proactive Engine — the layer that makes Sigil a CEO, not a tool.
//!
//! Provides morning briefs, anomaly detection, suggestion generation,
//! and a notification queue. All pure computation — no daemon wiring.

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Morning Brief
// ---------------------------------------------------------------------------

/// A single item within a brief section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefItem {
    /// Icon: "✓", "✗", "⟳", "!", "○"
    pub icon: String,
    /// Human-readable text.
    pub text: String,
    /// Optional metadata (cost, duration, agent name, etc.).
    pub metadata: Option<String>,
}

/// A titled section within the morning brief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefSection {
    pub title: String,
    pub items: Vec<BriefItem>,
}

/// The assembled morning brief, ready for delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorningBrief {
    pub generated_at: DateTime<Utc>,
    pub greeting: String,
    pub sections: Vec<BriefSection>,
    pub summary: String,
}

/// Summary of a task for brief / anomaly / suggestion inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub subject: String,
    pub project: String,
    pub agent: Option<String>,
    pub cost_usd: Option<f64>,
}

/// A directive status change observed since the last brief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdate {
    pub channel: String,
    pub directive: String,
    pub old_status: String,
    pub new_status: String,
}

/// Builder that assembles a [`MorningBrief`] section by section.
pub struct BriefBuilder {
    sections: Vec<BriefSection>,
}

impl BriefBuilder {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a "Completed" section with ✓ items.
    pub fn with_completed_tasks(&mut self, tasks: &[TaskSummary]) -> &mut Self {
        if tasks.is_empty() {
            return self;
        }
        let items = tasks
            .iter()
            .map(|t| BriefItem {
                icon: "\u{2713}".to_string(), // ✓
                text: format!("{} ({})", t.subject, t.project),
                metadata: t.cost_usd.map(|c| format!("${c:.2}")),
            })
            .collect();
        self.sections.push(BriefSection {
            title: "Completed".to_string(),
            items,
        });
        self
    }

    /// Add a "Blocked" section with ✗ items.
    pub fn with_blocked_tasks(&mut self, tasks: &[TaskSummary]) -> &mut Self {
        if tasks.is_empty() {
            return self;
        }
        let items = tasks
            .iter()
            .map(|t| BriefItem {
                icon: "\u{2717}".to_string(), // ✗
                text: format!("{} ({})", t.subject, t.project),
                metadata: t.agent.clone(),
            })
            .collect();
        self.sections.push(BriefSection {
            title: "Blocked".to_string(),
            items,
        });
        self
    }

    /// Add an "In Progress" section with ⟳ items.
    pub fn with_active_tasks(&mut self, tasks: &[TaskSummary]) -> &mut Self {
        if tasks.is_empty() {
            return self;
        }
        let items = tasks
            .iter()
            .map(|t| BriefItem {
                icon: "\u{27f3}".to_string(), // ⟳
                text: format!("{} ({})", t.subject, t.project),
                metadata: t.agent.clone(),
            })
            .collect();
        self.sections.push(BriefSection {
            title: "In Progress".to_string(),
            items,
        });
        self
    }

    /// Add a "Cost" section with budget information.
    pub fn with_cost_summary(&mut self, spent: f64, budget: f64) -> &mut Self {
        let pct = if budget > 0.0 {
            (spent / budget * 100.0) as u32
        } else {
            0
        };
        let icon = if pct > 80 {
            "!".to_string()
        } else {
            "\u{25cb}".to_string() // ○
        };
        let items = vec![BriefItem {
            icon,
            text: format!("${spent:.2} of ${budget:.2} budget ({pct}%)"),
            metadata: None,
        }];
        self.sections.push(BriefSection {
            title: "Cost".to_string(),
            items,
        });
        self
    }

    /// Add a "Notes" section showing directive status changes.
    pub fn with_note_updates(&mut self, updates: &[NoteUpdate]) -> &mut Self {
        if updates.is_empty() {
            return self;
        }
        let items = updates
            .iter()
            .map(|u| BriefItem {
                icon: "\u{27f3}".to_string(), // ⟳
                text: format!(
                    "{}: {} \u{2192} {}",
                    u.directive, u.old_status, u.new_status
                ),
                metadata: Some(u.channel.clone()),
            })
            .collect();
        self.sections.push(BriefSection {
            title: "Notes".to_string(),
            items,
        });
        self
    }

    /// Add an "Attention" section with anomaly items.
    pub fn with_anomalies(&mut self, anomalies: &[Anomaly]) -> &mut Self {
        if anomalies.is_empty() {
            return self;
        }
        let items = anomalies
            .iter()
            .map(|a| BriefItem {
                icon: "!".to_string(),
                text: a.message.clone(),
                metadata: a.project.clone(),
            })
            .collect();
        self.sections.push(BriefSection {
            title: "Attention".to_string(),
            items,
        });
        self
    }

    /// Assemble the final [`MorningBrief`].
    pub fn build(&self) -> MorningBrief {
        let now = Utc::now();
        let greeting = greeting_for_hour(now.hour());

        let total_items: usize = self.sections.iter().map(|s| s.items.len()).sum();
        let summary = if total_items == 0 {
            "Nothing to report. All quiet.".to_string()
        } else {
            let section_names: Vec<&str> = self.sections.iter().map(|s| s.title.as_str()).collect();
            format!(
                "{total_items} item{} across {}.",
                if total_items == 1 { "" } else { "s" },
                section_names.join(", ")
            )
        };

        MorningBrief {
            generated_at: now,
            greeting,
            sections: self.sections.clone(),
            summary,
        }
    }

    /// Render the brief to plain text (for Telegram / terminal delivery).
    pub fn render_text(&self) -> String {
        let brief = self.build();
        let mut out = String::new();

        out.push_str(&brief.greeting);
        out.push('\n');
        out.push('\n');

        for section in &brief.sections {
            out.push_str(&format!("--- {} ---\n", section.title));
            for item in &section.items {
                out.push_str(&format!("  {} {}", item.icon, item.text));
                if let Some(ref meta) = item.metadata {
                    out.push_str(&format!("  [{meta}]"));
                }
                out.push('\n');
            }
            out.push('\n');
        }

        out.push_str(&brief.summary);
        out.push('\n');
        out
    }
}

impl Default for BriefBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Return a greeting string based on the hour of day (UTC).
fn greeting_for_hour(hour: u32) -> String {
    match hour {
        5..=11 => "Good morning. Here\u{2019}s your brief.".to_string(),
        12..=17 => "Good afternoon. Here\u{2019}s your brief.".to_string(),
        18..=21 => "Good evening. Here\u{2019}s your brief.".to_string(),
        _ => "Here\u{2019}s your brief.".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Anomaly Detection
// ---------------------------------------------------------------------------

/// The type of anomaly detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    CostSpike,
    ErrorSurge,
    PerformanceDrop,
    AgentDegradation,
    StaleTask,
}

impl fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CostSpike => write!(f, "cost_spike"),
            Self::ErrorSurge => write!(f, "error_surge"),
            Self::PerformanceDrop => write!(f, "performance_drop"),
            Self::AgentDegradation => write!(f, "agent_degradation"),
            Self::StaleTask => write!(f, "stale_task"),
        }
    }
}

/// How severe the anomaly is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for AnomalySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// A detected anomaly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub message: String,
    pub project: Option<String>,
    pub detected_at: DateTime<Utc>,
}

/// Running baseline statistics for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub avg_cost: f64,
    pub avg_failure_rate: f32,
    pub avg_duration_ms: u64,
    pub sample_count: u32,
}

/// Detects anomalies by comparing current metrics against per-project baselines.
pub struct AnomalyDetector {
    pub baselines: HashMap<String, Baseline>,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
        }
    }

    /// Check if the current cost for a project is anomalous.
    /// Returns `CostSpike` warning if cost > 3x the baseline average.
    pub fn check_cost(&self, project: &str, current_cost: f64) -> Option<Anomaly> {
        let baseline = self.baselines.get(project)?;
        if baseline.avg_cost <= 0.0 {
            return None;
        }
        if current_cost > baseline.avg_cost * 3.0 {
            Some(Anomaly {
                anomaly_type: AnomalyType::CostSpike,
                severity: AnomalySeverity::Warning,
                message: format!(
                    "{project}: cost ${current_cost:.2} is {:.1}x the baseline ${:.2}",
                    current_cost / baseline.avg_cost,
                    baseline.avg_cost,
                ),
                project: Some(project.to_string()),
                detected_at: Utc::now(),
            })
        } else {
            None
        }
    }

    /// Check if the failure rate for a project is anomalous.
    /// Returns `ErrorSurge` warning if the current rate > 2x the baseline average.
    pub fn check_failure_rate(
        &self,
        project: &str,
        failures: u32,
        total: u32,
    ) -> Option<Anomaly> {
        if total == 0 {
            return None;
        }
        let baseline = self.baselines.get(project)?;
        if baseline.avg_failure_rate <= 0.0 {
            return None;
        }
        let current_rate = failures as f32 / total as f32;
        if current_rate > baseline.avg_failure_rate * 2.0 {
            Some(Anomaly {
                anomaly_type: AnomalyType::ErrorSurge,
                severity: AnomalySeverity::Warning,
                message: format!(
                    "{project}: failure rate {:.0}% is {:.1}x the baseline {:.0}%",
                    current_rate * 100.0,
                    current_rate / baseline.avg_failure_rate,
                    baseline.avg_failure_rate * 100.0,
                ),
                project: Some(project.to_string()),
                detected_at: Utc::now(),
            })
        } else {
            None
        }
    }

    /// Check for stale tasks — tasks with no progress beyond `threshold_hours`.
    pub fn check_stale_tasks(
        &self,
        tasks: &[TaskSummary],
        threshold_hours: u64,
    ) -> Vec<Anomaly> {
        // In a real system we'd compare against last-updated timestamps.
        // Here we flag every task provided (the caller pre-filters to stale ones).
        tasks
            .iter()
            .map(|t| Anomaly {
                anomaly_type: AnomalyType::StaleTask,
                severity: AnomalySeverity::Info,
                message: format!(
                    "{}: \"{}\" has had no progress for >{threshold_hours}h",
                    t.project, t.subject,
                ),
                project: Some(t.project.clone()),
                detected_at: Utc::now(),
            })
            .collect()
    }

    /// Record or update baseline metrics for a project (running average).
    pub fn record_baseline(
        &mut self,
        project: &str,
        cost: f64,
        failure_rate: f32,
        avg_duration_ms: u64,
    ) {
        let entry = self
            .baselines
            .entry(project.to_string())
            .or_insert(Baseline {
                avg_cost: 0.0,
                avg_failure_rate: 0.0,
                avg_duration_ms: 0,
                sample_count: 0,
            });

        let n = entry.sample_count as f64;
        let new_n = n + 1.0;

        // Incremental mean update: avg = avg + (new - avg) / new_count
        entry.avg_cost = entry.avg_cost + (cost - entry.avg_cost) / new_n;
        entry.avg_failure_rate =
            entry.avg_failure_rate + (failure_rate - entry.avg_failure_rate) / new_n as f32;
        entry.avg_duration_ms = ((entry.avg_duration_ms as f64
            + (avg_duration_ms as f64 - entry.avg_duration_ms as f64) / new_n)
            as u64)
            .max(1);
        entry.sample_count += 1;
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Suggestion Engine
// ---------------------------------------------------------------------------

/// The kind of suggestion being offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    ActivateDirective,
    SimilarPastTask,
    AutomatePattern,
    FollowUp,
}

impl fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActivateDirective => write!(f, "activate_directive"),
            Self::SimilarPastTask => write!(f, "similar_past_task"),
            Self::AutomatePattern => write!(f, "automate_pattern"),
            Self::FollowUp => write!(f, "follow_up"),
        }
    }
}

/// A proactive suggestion offered to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub suggestion_type: SuggestionType,
    pub message: String,
    pub context: Option<String>,
    pub action_label: String,
}

/// Pure-computation suggestion engine.
pub struct SuggestionEngine;

impl SuggestionEngine {
    /// For each pending directive (channel, directive_text) without a linked task,
    /// suggest activation.
    pub fn suggest_from_notes(directives: &[(String, String)]) -> Vec<Suggestion> {
        directives
            .iter()
            .map(|(channel, directive)| Suggestion {
                suggestion_type: SuggestionType::ActivateDirective,
                message: format!("You wrote \"{directive}\" \u{2014} want me to start?"),
                context: Some(channel.clone()),
                action_label: "Start".to_string(),
            })
            .collect()
    }

    /// After a task completes, suggest a generic follow-up.
    pub fn suggest_follow_up(completed_task: &TaskSummary) -> Option<Suggestion> {
        Some(Suggestion {
            suggestion_type: SuggestionType::FollowUp,
            message: format!(
                "Task \"{}\" completed. Need anything else for {}?",
                completed_task.subject, completed_task.project,
            ),
            context: Some(completed_task.id.clone()),
            action_label: "Follow up".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

/// The kind of notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Brief,
    TaskCompleted,
    TaskFailed,
    Anomaly,
    Suggestion,
    DirectiveUpdate,
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Brief => write!(f, "brief"),
            Self::TaskCompleted => write!(f, "task_completed"),
            Self::TaskFailed => write!(f, "task_failed"),
            Self::Anomaly => write!(f, "anomaly"),
            Self::Suggestion => write!(f, "suggestion"),
            Self::DirectiveUpdate => write!(f, "directive_update"),
        }
    }
}

/// A notification ready for delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub channel: Option<String>,
    pub created_at: DateTime<Utc>,
    pub delivered: bool,
}

/// Simple in-memory notification queue.
pub struct NotificationQueue {
    queue: Vec<Notification>,
}

impl NotificationQueue {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Enqueue a notification.
    pub fn push(&mut self, notification: Notification) {
        self.queue.push(notification);
    }

    /// Drain all undelivered notifications, marking them as delivered.
    /// Returns the notifications that were pending.
    pub fn drain_pending(&mut self) -> Vec<Notification> {
        let mut pending = Vec::new();
        for n in &mut self.queue {
            if !n.delivered {
                n.delivered = true;
                pending.push(n.clone());
            }
        }
        pending
    }

    /// Count of notifications not yet delivered.
    pub fn pending_count(&self) -> usize {
        self.queue.iter().filter(|n| !n.delivered).count()
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper ---

    fn sample_task(id: &str, subject: &str, project: &str) -> TaskSummary {
        TaskSummary {
            id: id.to_string(),
            subject: subject.to_string(),
            project: project.to_string(),
            agent: None,
            cost_usd: None,
        }
    }

    fn sample_notification(ntype: NotificationType, title: &str) -> Notification {
        Notification {
            id: uuid::Uuid::new_v4().to_string(),
            notification_type: ntype,
            title: title.to_string(),
            body: "body".to_string(),
            channel: None,
            created_at: Utc::now(),
            delivered: false,
        }
    }

    // ===== BriefBuilder =====

    #[test]
    fn test_brief_empty() {
        let builder = BriefBuilder::new();
        let brief = builder.build();
        assert!(brief.sections.is_empty());
        assert_eq!(brief.summary, "Nothing to report. All quiet.");
    }

    #[test]
    fn test_brief_all_sections() {
        let completed = vec![
            sample_task("t1", "Fix auth", "sigil"),
            sample_task("t2", "Deploy bot", "algostaking"),
        ];
        let blocked = vec![sample_task("t3", "DB migration", "sigil")];
        let active = vec![sample_task("t4", "New dashboard", "sigil")];
        let note_updates = vec![NoteUpdate {
            channel: "sigil/engineering".to_string(),
            directive: "build pricing page".to_string(),
            old_status: "pending".to_string(),
            new_status: "active".to_string(),
        }];
        let anomalies = vec![Anomaly {
            anomaly_type: AnomalyType::CostSpike,
            severity: AnomalySeverity::Warning,
            message: "sigil: cost spike detected".to_string(),
            project: Some("sigil".to_string()),
            detected_at: Utc::now(),
        }];

        let mut builder = BriefBuilder::new();
        builder
            .with_completed_tasks(&completed)
            .with_blocked_tasks(&blocked)
            .with_active_tasks(&active)
            .with_cost_summary(4.50, 10.0)
            .with_note_updates(&note_updates)
            .with_anomalies(&anomalies);

        let brief = builder.build();
        assert_eq!(brief.sections.len(), 6);
        assert_eq!(brief.sections[0].title, "Completed");
        assert_eq!(brief.sections[0].items.len(), 2);
        assert_eq!(brief.sections[1].title, "Blocked");
        assert_eq!(brief.sections[1].items.len(), 1);
        assert_eq!(brief.sections[2].title, "In Progress");
        assert_eq!(brief.sections[3].title, "Cost");
        assert_eq!(brief.sections[4].title, "Notes");
        assert_eq!(brief.sections[5].title, "Attention");

        // Total items: 2 + 1 + 1 + 1 + 1 + 1 = 7
        assert!(brief.summary.contains("7 items"));
    }

    #[test]
    fn test_brief_greeting_morning() {
        let greeting = greeting_for_hour(8);
        assert!(greeting.contains("morning"), "8am should be morning");
    }

    #[test]
    fn test_brief_greeting_afternoon() {
        let greeting = greeting_for_hour(14);
        assert!(greeting.contains("afternoon"), "2pm should be afternoon");
    }

    #[test]
    fn test_brief_greeting_evening() {
        let greeting = greeting_for_hour(19);
        assert!(greeting.contains("evening"), "7pm should be evening");
    }

    #[test]
    fn test_brief_greeting_night() {
        let greeting = greeting_for_hour(2);
        // Night falls through to the default.
        assert!(
            !greeting.contains("morning")
                && !greeting.contains("afternoon")
                && !greeting.contains("evening"),
            "2am should be the default greeting"
        );
        assert!(greeting.contains("brief"));
    }

    #[test]
    fn test_brief_render_text() {
        let completed = vec![sample_task("t1", "Fix auth", "sigil")];
        let mut builder = BriefBuilder::new();
        builder.with_completed_tasks(&completed);

        let text = builder.render_text();
        assert!(text.contains("--- Completed ---"));
        assert!(text.contains("Fix auth (sigil)"));
        assert!(text.contains("brief")); // greeting
    }

    #[test]
    fn test_brief_empty_sections_skipped() {
        let empty: Vec<TaskSummary> = vec![];
        let mut builder = BriefBuilder::new();
        builder.with_completed_tasks(&empty);
        builder.with_blocked_tasks(&empty);
        let brief = builder.build();
        assert!(brief.sections.is_empty());
    }

    #[test]
    fn test_brief_cost_high_usage_icon() {
        let mut builder = BriefBuilder::new();
        builder.with_cost_summary(9.0, 10.0);
        let brief = builder.build();
        // 90% usage should use "!" icon
        assert_eq!(brief.sections[0].items[0].icon, "!");
    }

    #[test]
    fn test_brief_cost_low_usage_icon() {
        let mut builder = BriefBuilder::new();
        builder.with_cost_summary(2.0, 10.0);
        let brief = builder.build();
        // 20% usage should use "○" icon
        assert_eq!(brief.sections[0].items[0].icon, "\u{25cb}");
    }

    // ===== AnomalyDetector =====

    #[test]
    fn test_anomaly_cost_spike() {
        let mut detector = AnomalyDetector::new();
        // Record baseline: avg_cost = $1.00
        for _ in 0..5 {
            detector.record_baseline("sigil", 1.0, 0.05, 5000);
        }

        // Current cost $4.00 is 4x baseline → spike
        let anomaly = detector.check_cost("sigil", 4.0);
        assert!(anomaly.is_some());
        let a = anomaly.unwrap();
        assert_eq!(a.anomaly_type, AnomalyType::CostSpike);
        assert_eq!(a.severity, AnomalySeverity::Warning);
        assert!(a.message.contains("sigil"));
    }

    #[test]
    fn test_anomaly_cost_below_threshold() {
        let mut detector = AnomalyDetector::new();
        for _ in 0..5 {
            detector.record_baseline("sigil", 1.0, 0.05, 5000);
        }

        // Current cost $2.50 is 2.5x baseline → below 3x threshold
        let anomaly = detector.check_cost("sigil", 2.5);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_anomaly_cost_no_baseline() {
        let detector = AnomalyDetector::new();
        // No baseline recorded → returns None
        let anomaly = detector.check_cost("sigil", 100.0);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_anomaly_failure_rate_surge() {
        let mut detector = AnomalyDetector::new();
        // Baseline: 10% failure rate
        for _ in 0..5 {
            detector.record_baseline("sigil", 1.0, 0.10, 5000);
        }

        // Current: 5/10 = 50% failure rate → 5x baseline → surge
        let anomaly = detector.check_failure_rate("sigil", 5, 10);
        assert!(anomaly.is_some());
        let a = anomaly.unwrap();
        assert_eq!(a.anomaly_type, AnomalyType::ErrorSurge);
    }

    #[test]
    fn test_anomaly_failure_rate_below_threshold() {
        let mut detector = AnomalyDetector::new();
        for _ in 0..5 {
            detector.record_baseline("sigil", 1.0, 0.10, 5000);
        }

        // Current: 1/10 = 10% failure rate → 1x baseline → no anomaly
        let anomaly = detector.check_failure_rate("sigil", 1, 10);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_anomaly_failure_rate_zero_total() {
        let mut detector = AnomalyDetector::new();
        detector.record_baseline("sigil", 1.0, 0.10, 5000);

        // Zero total tasks → None (avoid division by zero)
        let anomaly = detector.check_failure_rate("sigil", 0, 0);
        assert!(anomaly.is_none());
    }

    #[test]
    fn test_anomaly_stale_tasks() {
        let detector = AnomalyDetector::new();
        let tasks = vec![
            sample_task("t1", "Stale task 1", "sigil"),
            sample_task("t2", "Stale task 2", "algostaking"),
        ];

        let anomalies = detector.check_stale_tasks(&tasks, 2);
        assert_eq!(anomalies.len(), 2);
        assert!(anomalies
            .iter()
            .all(|a| a.anomaly_type == AnomalyType::StaleTask));
        assert!(anomalies
            .iter()
            .all(|a| a.severity == AnomalySeverity::Info));
        assert!(anomalies[0].message.contains(">2h"));
    }

    #[test]
    fn test_anomaly_stale_tasks_empty() {
        let detector = AnomalyDetector::new();
        let anomalies = detector.check_stale_tasks(&[], 2);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_baseline_recording() {
        let mut detector = AnomalyDetector::new();
        detector.record_baseline("sigil", 2.0, 0.10, 4000);
        detector.record_baseline("sigil", 4.0, 0.20, 6000);

        let baseline = detector.baselines.get("sigil").unwrap();
        assert_eq!(baseline.sample_count, 2);
        // avg_cost = (2.0 + 4.0) / 2 = 3.0
        assert!((baseline.avg_cost - 3.0).abs() < 0.001);
        // avg_failure_rate = (0.10 + 0.20) / 2 = 0.15
        assert!((baseline.avg_failure_rate - 0.15).abs() < 0.001);
        // avg_duration_ms = (4000 + 6000) / 2 = 5000
        assert_eq!(baseline.avg_duration_ms, 5000);
    }

    #[test]
    fn test_baseline_recording_single() {
        let mut detector = AnomalyDetector::new();
        detector.record_baseline("sigil", 5.0, 0.25, 3000);

        let baseline = detector.baselines.get("sigil").unwrap();
        assert_eq!(baseline.sample_count, 1);
        assert!((baseline.avg_cost - 5.0).abs() < 0.001);
        assert!((baseline.avg_failure_rate - 0.25).abs() < 0.001);
        assert_eq!(baseline.avg_duration_ms, 3000);
    }

    // ===== SuggestionEngine =====

    #[test]
    fn test_suggestion_from_notes() {
        let directives = vec![
            (
                "sigil/engineering".to_string(),
                "build pricing page".to_string(),
            ),
            (
                "algostaking".to_string(),
                "optimize bot latency".to_string(),
            ),
        ];

        let suggestions = SuggestionEngine::suggest_from_notes(&directives);
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::ActivateDirective);
        assert!(suggestions[0].message.contains("build pricing page"));
        assert!(suggestions[0].message.contains("want me to start"));
        assert_eq!(suggestions[0].action_label, "Start");
        assert_eq!(
            suggestions[0].context.as_deref(),
            Some("sigil/engineering")
        );
    }

    #[test]
    fn test_suggestion_from_notes_empty() {
        let suggestions = SuggestionEngine::suggest_from_notes(&[]);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_suggestion_follow_up() {
        let task = sample_task("sg-1001", "Fix auth bug", "sigil");
        let suggestion = SuggestionEngine::suggest_follow_up(&task);
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert_eq!(s.suggestion_type, SuggestionType::FollowUp);
        assert!(s.message.contains("Fix auth bug"));
        assert!(s.message.contains("sigil"));
        assert_eq!(s.action_label, "Follow up");
    }

    // ===== NotificationQueue =====

    #[test]
    fn test_notification_push_and_drain() {
        let mut queue = NotificationQueue::new();
        queue.push(sample_notification(NotificationType::Brief, "Morning brief"));
        queue.push(sample_notification(
            NotificationType::TaskCompleted,
            "Task done",
        ));

        assert_eq!(queue.pending_count(), 2);

        let drained = queue.drain_pending();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].title, "Morning brief");
        assert_eq!(drained[1].title, "Task done");
    }

    #[test]
    fn test_notification_pending_count() {
        let mut queue = NotificationQueue::new();
        assert_eq!(queue.pending_count(), 0);

        queue.push(sample_notification(NotificationType::Anomaly, "Spike"));
        assert_eq!(queue.pending_count(), 1);

        queue.push(sample_notification(NotificationType::Suggestion, "Idea"));
        assert_eq!(queue.pending_count(), 2);
    }

    #[test]
    fn test_notification_drain_marks_delivered() {
        let mut queue = NotificationQueue::new();
        queue.push(sample_notification(NotificationType::Brief, "Brief"));

        assert!(!queue.queue[0].delivered, "should start undelivered");

        let drained = queue.drain_pending();
        assert_eq!(drained.len(), 1);
        // The notification IN the queue should be marked delivered.
        assert!(queue.queue[0].delivered, "queue entry should be marked delivered");
        // Subsequent drain returns nothing.
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_notification_double_drain_returns_empty() {
        let mut queue = NotificationQueue::new();
        queue.push(sample_notification(NotificationType::Brief, "Brief"));
        queue.push(sample_notification(
            NotificationType::TaskFailed,
            "Failed",
        ));

        let first = queue.drain_pending();
        assert_eq!(first.len(), 2);

        let second = queue.drain_pending();
        assert!(second.is_empty(), "second drain should return nothing");
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_notification_interleaved_push_drain() {
        let mut queue = NotificationQueue::new();
        queue.push(sample_notification(NotificationType::Brief, "Brief 1"));

        let first = queue.drain_pending();
        assert_eq!(first.len(), 1);

        // Push another after drain.
        queue.push(sample_notification(NotificationType::Anomaly, "Anomaly 1"));

        let second = queue.drain_pending();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].title, "Anomaly 1");

        assert_eq!(queue.pending_count(), 0);
    }
}
