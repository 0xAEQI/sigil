//! Git-native task management with JSONL persistence and hierarchical IDs.
//!
//! Tasks are organized as a DAG with prefix-based IDs (e.g., `ALG-1`, `ALG-1.1`),
//! support priorities, dependencies, assignees, and checkpoints. Missions group
//! related tasks and auto-complete when all member tasks are done.
//!
//! Key types: [`Task`], [`TaskBoard`], [`Mission`], [`TaskQuery`].

pub mod task;
pub mod mission;
pub mod store;
pub mod query;
pub mod dependency_inference;

pub use task::{Checkpoint, Task, TaskId, TaskStatus, Priority};
pub use mission::Mission;
pub use store::TaskBoard;
pub use query::TaskQuery;
pub use dependency_inference::{InferredDependency, infer_dependencies};
