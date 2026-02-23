use anyhow::Result;
use realm_quests::QuestBoard;
use realm_core::config::DomainConfig;
use realm_core::identity::Identity;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A Domain is an isolated Business Unit container.
/// Each domain has its own quest store, identity, spirits, and worktree root.
pub struct Domain {
    pub name: String,
    pub prefix: String,
    pub repo: PathBuf,
    pub worktree_root: PathBuf,
    pub model: String,
    pub max_workers: u32,
    pub spirit_timeout_secs: u64,
    pub identity: Identity,
    pub quests: Arc<Mutex<QuestBoard>>,
}

impl Domain {
    /// Create a domain from configuration.
    pub fn from_config(config: &DomainConfig, domain_dir: &std::path::Path, default_model: &str) -> Result<Self> {
        let identity = Identity::load(domain_dir).unwrap_or_default();

        let quests_dir = domain_dir.join(".quests");
        let quest_board = QuestBoard::open(&quests_dir)?;

        let worktree_root = config
            .worktree_root
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&config.repo).join("..").join("worktrees"));

        Ok(Self {
            name: config.name.clone(),
            prefix: config.prefix.clone(),
            repo: PathBuf::from(&config.repo),
            worktree_root,
            model: config.model.clone().unwrap_or_else(|| default_model.to_string()),
            max_workers: config.max_workers,
            spirit_timeout_secs: config.spirit_timeout_secs,
            identity,
            quests: Arc::new(Mutex::new(quest_board)),
        })
    }

    /// Create a quest in this domain's store.
    pub async fn create_quest(&self, subject: &str) -> Result<realm_quests::Quest> {
        let mut store = self.quests.lock().await;
        store.create(&self.prefix, subject)
    }

    /// Get ready quests for this domain.
    pub async fn ready_quests(&self) -> Vec<realm_quests::Quest> {
        let store = self.quests.lock().await;
        store.ready().into_iter().cloned().collect()
    }

    /// Get all open quests for this domain.
    pub async fn open_quests(&self) -> Vec<realm_quests::Quest> {
        let store = self.quests.lock().await;
        store
            .by_prefix(&self.prefix)
            .into_iter()
            .filter(|q| !q.is_closed())
            .cloned()
            .collect()
    }
}
