use aeqi_core::config::CompanyConfig;
use aeqi_core::identity::Identity;
use aeqi_tasks::TaskBoard;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// A Company is a container for repos, tasks, and budget.
/// Companies do NOT have agent personality — agents work ON companies.
pub struct Company {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub repo: PathBuf,
    pub worktree_root: PathBuf,
    pub model: String,
    pub max_workers: u32,
    pub worker_timeout_secs: u64,
    /// Project-only context (AGENTS.md, KNOWLEDGE.md, HEARTBEAT.md from projects/{name}/).
    pub company_identity: Identity,
    pub tasks: Arc<Mutex<TaskBoard>>,
    pub task_notify: Arc<Notify>,
    pub departments: Vec<aeqi_core::config::DepartmentConfig>,
}

impl Company {
    /// Create a project from configuration.
    pub fn from_config(
        config: &CompanyConfig,
        project_dir: &std::path::Path,
        default_model: &str,
    ) -> Result<Self> {
        // Load project-only files (AGENTS.md, KNOWLEDGE.md, HEARTBEAT.md).
        // Uses load_from_dir since projects don't have agent personality.
        let company_identity = Identity::load_from_dir(project_dir).unwrap_or_default();

        let tasks_dir = project_dir.join(".tasks");
        let task_board = TaskBoard::open(&tasks_dir)?;

        let worktree_root = config
            .worktree_root
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&config.repo).join("..").join("worktrees"));

        Ok(Self {
            id: config
                .id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: config.name.clone(),
            prefix: config.prefix.clone(),
            repo: PathBuf::from(&config.repo),
            worktree_root,
            model: config
                .model
                .clone()
                .unwrap_or_else(|| default_model.to_string()),
            max_workers: config.max_workers,
            worker_timeout_secs: config.worker_timeout_secs,
            company_identity,
            tasks: Arc::new(Mutex::new(task_board)),
            task_notify: Arc::new(Notify::new()),
            departments: config.departments.clone(),
        })
    }

    /// Create a task in this project's store.
    pub async fn create_task(
        &self,
        subject: &str,
        agent_id: Option<&str>,
    ) -> Result<aeqi_tasks::Task> {
        let mut store = self.tasks.lock().await;
        store.create_with_agent(&self.prefix, subject, agent_id)
    }

    /// Get ready tasks for this project.
    pub async fn ready_tasks(&self) -> Vec<aeqi_tasks::Task> {
        let store = self.tasks.lock().await;
        store.ready().into_iter().cloned().collect()
    }

    /// Get all open tasks for this project.
    pub async fn open_tasks(&self) -> Vec<aeqi_tasks::Task> {
        let store = self.tasks.lock().await;
        store
            .by_prefix(&self.prefix)
            .into_iter()
            .filter(|q| !q.is_closed())
            .cloned()
            .collect()
    }
}
