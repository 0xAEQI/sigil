//! Agent orchestration engine — the operational heart of Sigil.
//!
//! Coordinates worker execution ([`AgentWorker`]), supervisor patrol ([`Supervisor`]),
//! Gemini Flash router classification ([`AgentRouter`]), project registry ([`ProjectRegistry`]),
//! dispatch bus ([`DispatchBus`]), cost ledger ([`CostLedger`]), Prometheus metrics
//! ([`SigilMetrics`]), lifecycle engine ([`LifecycleEngine`]), and conversation storage.
//!
//! Workers spawn via Claude Code (`claude -p`) with full tool access. The supervisor
//! enforces budgets and escalation chains (worker → project leader → system leader → human).

pub mod operation;
pub mod schedule;
pub mod daemon;
pub mod executor;
pub mod heartbeat;
pub mod reflection;
pub mod hook;
pub mod message;
pub mod pipeline;
pub mod registry;
pub mod project;
pub mod tools;
pub mod supervisor;
pub mod agent_worker;
pub mod agent_router;
pub mod council;
pub mod cost_ledger;
pub mod context_budget;
pub mod metrics;
pub mod template;
pub mod checkpoint;
pub mod session_tracker;
pub mod conversation_store;
pub mod emotional_state;
pub mod lifecycle;
pub mod audit;
pub mod expertise;
pub mod blackboard;
pub mod failure_analysis;
pub mod preflight;
pub mod decomposition;
pub mod watchdog;

pub use operation::{Operation, OperationStore};
pub use schedule::{ScheduledJob, ScheduleStore};
pub use daemon::Daemon;
pub use executor::{ClaudeCodeExecutor, TaskOutcome};
pub use heartbeat::Heartbeat;
pub use reflection::Reflection;
pub use hook::Hook;
pub use message::{Dispatch, DispatchBus, DispatchKind};
pub use pipeline::{Pipeline, PipelineStep};
pub use registry::{ProjectRegistry, ProjectSummary, TeamSummary};
pub use project::Project;
pub use supervisor::Supervisor;
pub use agent_worker::{AgentWorker, WorkerState};
pub use agent_router::{AgentRouter, RouteDecision};
pub use council::Council;
pub use cost_ledger::CostLedger;
pub use context_budget::ContextBudget;
pub use metrics::SigilMetrics;
pub use template::Template;
pub use checkpoint::AgentCheckpoint;
pub use session_tracker::SessionTracker;
pub use conversation_store::ConversationStore;
pub use emotional_state::EmotionalState;
pub use lifecycle::LifecycleEngine;
pub use audit::{AuditLog, AuditEvent, DecisionType};
pub use expertise::ExpertiseLedger;
pub use blackboard::Blackboard;
pub use watchdog::WatchdogEngine;
