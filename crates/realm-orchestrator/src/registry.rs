use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use crate::cost_ledger::CostLedger;
use crate::metrics::RealmMetrics;
use crate::whisper::WhisperBus;
use crate::domain::Domain;
use crate::scout::Scout;

pub struct DomainRegistry {
    domains: RwLock<HashMap<String, Arc<Domain>>>,
    scouts: RwLock<HashMap<String, Arc<Mutex<Scout>>>>,
    pub whisper_bus: Arc<WhisperBus>,
    pub wake: Arc<tokio::sync::Notify>,
    pub cost_ledger: Arc<CostLedger>,
    pub metrics: Arc<RealmMetrics>,
}

impl DomainRegistry {
    pub fn new(whisper_bus: Arc<WhisperBus>) -> Self {
        Self {
            domains: RwLock::new(HashMap::new()),
            scouts: RwLock::new(HashMap::new()),
            whisper_bus,
            wake: Arc::new(tokio::sync::Notify::new()),
            cost_ledger: Arc::new(CostLedger::new(50.0)), // $50/day default
            metrics: Arc::new(RealmMetrics::new()),
        }
    }

    /// Set a custom cost ledger (e.g., with persistence).
    pub fn set_cost_ledger(&mut self, ledger: Arc<CostLedger>) {
        self.cost_ledger = ledger;
    }

    pub async fn register_domain(&self, domain: Arc<Domain>, mut scout: Scout) {
        let name = domain.name.clone();
        // Inject cost ledger + metrics into the scout.
        scout.cost_ledger = Some(self.cost_ledger.clone());
        scout.metrics = Some(self.metrics.clone());
        self.metrics.ensure_domain(&name);
        self.domains.write().await.insert(name.clone(), domain);
        self.scouts.write().await.insert(name, Arc::new(Mutex::new(scout)));
    }

    pub async fn assign(&self, domain_name: &str, subject: &str, description: &str) -> Result<realm_quests::Quest> {
        let domains = self.domains.read().await;
        let domain = domains
            .get(domain_name)
            .ok_or_else(|| anyhow::anyhow!("domain not found: {domain_name}"))?;

        let mut quest = domain.create_quest(subject).await?;

        if !description.is_empty() {
            let mut store = domain.quests.lock().await;
            quest = store.update(&quest.id.0, |q| {
                q.description = description.to_string();
            })?;
        }

        info!(
            domain = %domain_name,
            quest = %quest.id,
            subject = %subject,
            "task assigned"
        );

        self.wake.notify_one();
        Ok(quest)
    }

    pub async fn patrol_all(&self) -> Result<()> {
        let whispers = self.whisper_bus.read("familiar").await;
        for w in &whispers {
            match &w.kind {
                crate::whisper::WhisperKind::PatrolReport { domain, active, pending } => {
                    info!(from = %w.from, domain = %domain, active = active, pending = pending, "scout report");
                }
                crate::whisper::WhisperKind::SpiritCrashed { domain, spirit, error } => {
                    warn!(from = %w.from, domain = %domain, spirit = %spirit, error = %error, "spirit crashed");
                }
                _ => {
                    info!(from = %w.from, kind = %w.kind.subject_tag(), "whisper received");
                }
            }
        }

        // Parallel patrol: collect Arc clones, drop read lock, then join_all.
        let scout_entries: Vec<(String, Arc<Mutex<Scout>>)> = {
            let scouts = self.scouts.read().await;
            scouts.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };

        let futures: Vec<_> = scout_entries
            .into_iter()
            .map(|(name, scout)| async move {
                let mut s = scout.lock().await;
                if let Err(e) = s.patrol().await {
                    warn!(domain = %name, error = %e, "scout patrol failed");
                }
            })
            .collect();

        futures::future::join_all(futures).await;

        Ok(())
    }

    pub async fn status(&self) -> RegistryStatus {
        let mut domain_statuses = Vec::new();
        let domains = self.domains.read().await;
        let scouts = self.scouts.read().await;

        for (name, domain) in domains.iter() {
            let open = domain.open_quests().await.len();
            let ready = domain.ready_quests().await.len();
            let (idle, working, bonded) = if let Some(s) = scouts.get(name) {
                s.lock().await.spirit_counts()
            } else {
                (0, 0, 0)
            };

            domain_statuses.push(DomainStatus {
                name: name.clone(),
                open_quests: open,
                ready_quests: ready,
                spirits_idle: idle,
                spirits_working: working,
                spirits_bonded: bonded,
            });
        }

        let unread = self.whisper_bus.unread_count("familiar").await;

        RegistryStatus {
            domains: domain_statuses,
            unread_whispers: unread,
        }
    }

    pub async fn all_ready(&self) -> Vec<(String, realm_quests::Quest)> {
        let mut all = Vec::new();
        let domains = self.domains.read().await;
        for (name, domain) in domains.iter() {
            for quest in domain.ready_quests().await {
                all.push((name.clone(), quest));
            }
        }
        all
    }

    pub async fn domain_names(&self) -> Vec<String> {
        self.domains.read().await.keys().cloned().collect()
    }

    pub async fn get_domain(&self, name: &str) -> Option<Arc<Domain>> {
        self.domains.read().await.get(name).cloned()
    }

    pub async fn domain_count(&self) -> usize {
        self.domains.read().await.len()
    }

    pub async fn total_max_spirits(&self) -> u32 {
        self.domains.read().await.values().map(|d| d.max_workers).sum()
    }

    pub async fn domains_info(&self) -> Vec<serde_json::Value> {
        self.domains.read().await.values().map(|d| {
            serde_json::json!({
                "name": d.name,
                "prefix": d.prefix,
                "model": d.model,
                "max_workers": d.max_workers,
            })
        }).collect()
    }
}

#[derive(Debug)]
pub struct RegistryStatus {
    pub domains: Vec<DomainStatus>,
    pub unread_whispers: usize,
}

#[derive(Debug)]
pub struct DomainStatus {
    pub name: String,
    pub open_quests: usize,
    pub ready_quests: usize,
    pub spirits_idle: usize,
    pub spirits_working: usize,
    pub spirits_bonded: usize,
}
