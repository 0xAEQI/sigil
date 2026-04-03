use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Entity,
    Department,
    Domain,
    System,
}

impl std::fmt::Display for MemoryScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
            Self::Department => write!(f, "department"),
            Self::Domain => write!(f, "domain"),
            Self::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub key: String,
    pub content: String,
    pub category: MemoryCategory,
    pub scope: MemoryScope,
    pub entity_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub session_id: Option<String>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    Fact,
    Procedure,
    Preference,
    Context,
    Evergreen,
}

#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub text: String,
    pub top_k: usize,
    pub category: Option<MemoryCategory>,
    pub session_id: Option<String>,
    pub scope: Option<MemoryScope>,
    pub entity_id: Option<String>,
}

impl MemoryQuery {
    pub fn new(text: impl Into<String>, top_k: usize) -> Self {
        Self {
            text: text.into(),
            top_k,
            category: None,
            session_id: None,
            scope: None,
            entity_id: None,
        }
    }

    pub fn with_entity(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self.scope = Some(MemoryScope::Entity);
        self
    }

    pub fn with_scope(mut self, scope: MemoryScope) -> Self {
        self.scope = Some(scope);
        self
    }
}

#[async_trait]
pub trait Memory: Send + Sync {
    async fn store(
        &self,
        key: &str,
        content: &str,
        category: MemoryCategory,
        scope: MemoryScope,
        entity_id: Option<&str>,
    ) -> anyhow::Result<String>;

    async fn search(&self, query: &MemoryQuery) -> anyhow::Result<Vec<MemoryEntry>>;

    /// Hierarchical search: agent -> department -> project scope.
    /// Default implementation does 3 separate searches and merges.
    async fn hierarchical_search(
        &self,
        query: &str,
        agent_id: &str,
        department_id: Option<&str>,
        project_id: Option<&str>,
        top_k: usize,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        let mut all = Vec::new();

        // Agent scope
        let q = MemoryQuery::new(query, top_k).with_entity(agent_id.to_string());
        if let Ok(entries) = self.search(&q).await {
            all.extend(entries);
        }

        // Department scope
        if let Some(dept_id) = department_id {
            let mut q = MemoryQuery::new(query, top_k);
            q.scope = Some(MemoryScope::Department);
            q.entity_id = Some(dept_id.to_string());
            if let Ok(entries) = self.search(&q).await {
                all.extend(entries);
            }
        }

        // Project/domain scope
        if let Some(proj_id) = project_id {
            let mut q = MemoryQuery::new(query, top_k);
            q.scope = Some(MemoryScope::Domain);
            q.entity_id = Some(proj_id.to_string());
            if let Ok(entries) = self.search(&q).await {
                all.extend(entries);
            }
        } else {
            // Fallback: unscoped domain search
            let q = MemoryQuery::new(query, top_k).with_scope(MemoryScope::Domain);
            if let Ok(entries) = self.search(&q).await {
                all.extend(entries);
            }
        }

        // Dedup by id, sort by score, truncate
        all.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all.dedup_by(|a, b| a.id == b.id);
        all.truncate(top_k);
        Ok(all)
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()>;

    fn name(&self) -> &str;

    /// Store a memory graph edge. Default is no-op for backends that don't support edges.
    async fn store_memory_edge(
        &self,
        _source_id: &str,
        _target_id: &str,
        _relation: &str,
        _strength: f32,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
