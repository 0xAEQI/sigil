//! Cost Ledger — Tracks spending per domain/quest, enforces daily budgets.
//!
//! Records every spirit execution cost, provides budget status queries,
//! and persists to JSONL for crash recovery.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{info, warn};

/// A single cost entry from a spirit execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEntry {
    pub domain: String,
    pub quest_id: String,
    pub spirit: String,
    pub cost_usd: f64,
    pub turns: u32,
    pub timestamp: DateTime<Utc>,
}

/// Tracks spending across domains and enforces budget caps.
pub struct CostLedger {
    entries: Mutex<Vec<CostEntry>>,
    daily_budget_usd: f64,
    persist_path: Option<PathBuf>,
    /// Per-domain daily budget ceilings. Domains not in this map fall back to the global budget.
    domain_budgets: Mutex<HashMap<String, f64>>,
}

impl CostLedger {
    pub fn new(daily_budget_usd: f64) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            daily_budget_usd,
            persist_path: None,
            domain_budgets: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_persistence(daily_budget_usd: f64, path: PathBuf) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            daily_budget_usd,
            persist_path: Some(path),
            domain_budgets: Mutex::new(HashMap::new()),
        }
    }

    /// Record a cost entry. Warns if daily budget or domain budget exceeded.
    pub fn record(&self, entry: CostEntry) -> Result<()> {
        let domain_name = entry.domain.clone();
        let mut entries = self.entries.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;

        info!(
            domain = %entry.domain,
            quest = %entry.quest_id,
            spirit = %entry.spirit,
            cost = entry.cost_usd,
            turns = entry.turns,
            "cost recorded"
        );

        entries.push(entry);

        // Check global budget after recording.
        let spent_today = Self::sum_since(&entries, Utc::now() - Duration::hours(24));
        if spent_today > self.daily_budget_usd {
            warn!(
                spent = spent_today,
                budget = self.daily_budget_usd,
                overage = spent_today - self.daily_budget_usd,
                "DAILY BUDGET EXCEEDED"
            );
        }

        // Check domain-specific budget after recording.
        let budgets = self.domain_budgets.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(&domain_budget) = budgets.get(&domain_name) {
            let since = Utc::now() - Duration::hours(24);
            let domain_spent: f64 = entries.iter()
                .filter(|e| e.domain == domain_name && e.timestamp > since)
                .map(|e| e.cost_usd)
                .sum();
            if domain_spent > domain_budget {
                warn!(
                    domain = %domain_name,
                    spent = domain_spent,
                    budget = domain_budget,
                    overage = domain_spent - domain_budget,
                    "DOMAIN BUDGET EXCEEDED"
                );
            }
        }

        Ok(())
    }

    /// Check budget status: (spent_today, budget, remaining).
    pub fn budget_status(&self) -> (f64, f64, f64) {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let spent = Self::sum_since(&entries, Utc::now() - Duration::hours(24));
        let remaining = (self.daily_budget_usd - spent).max(0.0);
        (spent, self.daily_budget_usd, remaining)
    }

    /// Total spend for a domain in the last N hours.
    pub fn domain_spend(&self, domain: &str, hours: u32) -> f64 {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let since = Utc::now() - Duration::hours(hours as i64);
        entries
            .iter()
            .filter(|e| e.domain == domain && e.timestamp > since)
            .map(|e| e.cost_usd)
            .sum()
    }

    /// Total spend for a quest across all attempts.
    pub fn quest_spend(&self, quest_id: &str) -> f64 {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries
            .iter()
            .filter(|e| e.quest_id == quest_id)
            .map(|e| e.cost_usd)
            .sum()
    }

    /// Check if we can afford a new execution (budget not exhausted).
    pub fn can_afford(&self) -> bool {
        let (spent, budget, _) = self.budget_status();
        spent < budget
    }

    /// Set the daily budget cap for a specific domain.
    pub fn set_domain_budget(&self, domain: &str, budget_usd: f64) {
        let mut budgets = self.domain_budgets.lock().unwrap_or_else(|e| e.into_inner());
        budgets.insert(domain.to_string(), budget_usd);
        info!(domain = %domain, budget_usd, "domain budget set");
    }

    /// Check if a domain can afford a new execution.
    /// Returns false if EITHER the global daily budget OR the domain-specific cap is exceeded.
    /// If no domain budget is set, falls back to the global budget check only.
    pub fn can_afford_domain(&self, domain: &str) -> bool {
        // Global check first.
        if !self.can_afford() {
            return false;
        }

        // Domain-specific check.
        let budgets = self.domain_budgets.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(&domain_budget) = budgets.get(domain) {
            let spent = self.domain_spend(domain, 24);
            if spent >= domain_budget {
                return false;
            }
        }

        true
    }

    /// Get per-domain budget status: (spent_today, budget, remaining).
    /// If no domain budget is set, returns (spent_today, global_budget, global_remaining).
    pub fn domain_budget_status(&self, domain: &str) -> (f64, f64, f64) {
        let spent = self.domain_spend(domain, 24);
        let budgets = self.domain_budgets.lock().unwrap_or_else(|e| e.into_inner());
        let budget = budgets.get(domain).copied().unwrap_or(self.daily_budget_usd);
        let remaining = (budget - spent).max(0.0);
        (spent, budget, remaining)
    }

    /// Get all per-domain budget statuses as a map.
    /// Returns entries for every domain that has a budget set, plus any domain with spending.
    pub fn all_domain_budget_statuses(&self) -> HashMap<String, (f64, f64, f64)> {
        let budgets = self.domain_budgets.lock().unwrap_or_else(|e| e.into_inner());
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let since = Utc::now() - Duration::hours(24);

        // Collect all domains that either have a budget or have spending.
        let mut all_domains: HashSet<String> = budgets.keys().cloned().collect();
        for entry in entries.iter() {
            if entry.timestamp > since {
                all_domains.insert(entry.domain.clone());
            }
        }

        let mut result = HashMap::new();
        for domain in all_domains {
            let spent: f64 = entries.iter()
                .filter(|e| e.domain == domain && e.timestamp > since)
                .map(|e| e.cost_usd)
                .sum();
            let budget = budgets.get(&domain).copied().unwrap_or(self.daily_budget_usd);
            let remaining = (budget - spent).max(0.0);
            result.insert(domain, (spent, budget, remaining));
        }

        result
    }

    /// Save entries to JSONL file.
    pub fn save(&self) -> Result<()> {
        let path = match &self.persist_path {
            Some(p) => p,
            None => return Ok(()),
        };

        let entries = self.entries.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        for entry in entries.iter() {
            content.push_str(&serde_json::to_string(entry)?);
            content.push('\n');
        }

        std::fs::write(path, &content)
            .with_context(|| format!("failed to write cost ledger: {}", path.display()))?;

        Ok(())
    }

    /// Load entries from JSONL file.
    pub fn load(&self) -> Result<usize> {
        let path = match &self.persist_path {
            Some(p) => p,
            None => return Ok(0),
        };

        if !path.exists() {
            return Ok(0);
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read cost ledger: {}", path.display()))?;

        let mut entries = self.entries.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<CostEntry>(line) {
                Ok(entry) => {
                    entries.push(entry);
                    count += 1;
                }
                Err(e) => {
                    warn!(error = %e, "skipping malformed cost entry");
                }
            }
        }

        Ok(count)
    }

    /// Per-domain totals for the last 24 hours.
    pub fn daily_report(&self) -> HashMap<String, f64> {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let since = Utc::now() - Duration::hours(24);
        let mut report = HashMap::new();

        for entry in entries.iter() {
            if entry.timestamp > since {
                *report.entry(entry.domain.clone()).or_insert(0.0) += entry.cost_usd;
            }
        }

        report
    }

    /// Prune entries older than 7 days to prevent unbounded growth.
    pub fn prune_old(&self) {
        let cutoff = Utc::now() - Duration::days(7);
        if let Ok(mut entries) = self.entries.lock() {
            let before = entries.len();
            entries.retain(|e| e.timestamp > cutoff);
            let pruned = before - entries.len();
            if pruned > 0 {
                info!(pruned, "pruned old cost entries");
            }
        }
    }

    fn sum_since(entries: &[CostEntry], since: DateTime<Utc>) -> f64 {
        entries
            .iter()
            .filter(|e| e.timestamp > since)
            .map(|e| e.cost_usd)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query() {
        let ledger = CostLedger::new(100.0);

        ledger
            .record(CostEntry {
                domain: "algostaking".into(),
                quest_id: "as-001".into(),
                spirit: "as-worker-1".into(),
                cost_usd: 0.50,
                turns: 5,
                timestamp: Utc::now(),
            })
            .unwrap();

        ledger
            .record(CostEntry {
                domain: "riftdecks".into(),
                quest_id: "rd-001".into(),
                spirit: "rd-worker-1".into(),
                cost_usd: 0.30,
                turns: 3,
                timestamp: Utc::now(),
            })
            .unwrap();

        let (spent, budget, remaining) = ledger.budget_status();
        assert!((spent - 0.80).abs() < 0.01);
        assert!((budget - 100.0).abs() < 0.01);
        assert!(remaining > 99.0);

        assert!((ledger.domain_spend("algostaking", 24) - 0.50).abs() < 0.01);
        assert!((ledger.quest_spend("as-001") - 0.50).abs() < 0.01);
        assert!(ledger.can_afford());
    }

    #[test]
    fn test_daily_report() {
        let ledger = CostLedger::new(100.0);

        for i in 0..5 {
            ledger
                .record(CostEntry {
                    domain: "algostaking".into(),
                    quest_id: format!("as-{i:03}"),
                    spirit: format!("as-worker-{i}"),
                    cost_usd: 1.0,
                    turns: 5,
                    timestamp: Utc::now(),
                })
                .unwrap();
        }

        let report = ledger.daily_report();
        assert!((report["algostaking"] - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("costs.jsonl");

        let ledger = CostLedger::with_persistence(100.0, path.clone());
        ledger
            .record(CostEntry {
                domain: "test".into(),
                quest_id: "t-001".into(),
                spirit: "w1".into(),
                cost_usd: 1.23,
                turns: 4,
                timestamp: Utc::now(),
            })
            .unwrap();
        ledger.save().unwrap();

        let ledger2 = CostLedger::with_persistence(100.0, path);
        let count = ledger2.load().unwrap();
        assert_eq!(count, 1);
        assert!((ledger2.quest_spend("t-001") - 1.23).abs() < 0.01);
    }

    #[test]
    fn test_prune_old() {
        let ledger = CostLedger::new(100.0);

        // Add an old entry.
        {
            let mut entries = ledger.entries.lock().unwrap();
            entries.push(CostEntry {
                domain: "test".into(),
                quest_id: "old".into(),
                spirit: "w".into(),
                cost_usd: 1.0,
                turns: 1,
                timestamp: Utc::now() - Duration::days(10),
            });
        }

        // Add a recent entry.
        ledger
            .record(CostEntry {
                domain: "test".into(),
                quest_id: "new".into(),
                spirit: "w".into(),
                cost_usd: 2.0,
                turns: 2,
                timestamp: Utc::now(),
            })
            .unwrap();

        ledger.prune_old();
        assert!((ledger.quest_spend("old")).abs() < 0.01); // pruned
        assert!((ledger.quest_spend("new") - 2.0).abs() < 0.01); // kept
    }

    #[test]
    fn test_domain_budget_blocks_overspend() {
        let ledger = CostLedger::new(100.0); // Global: $100/day
        ledger.set_domain_budget("algostaking", 2.0); // Domain: $2/day

        // Spend $1.50 in algostaking — should still be under domain cap.
        ledger.record(CostEntry {
            domain: "algostaking".into(),
            quest_id: "as-001".into(),
            spirit: "w1".into(),
            cost_usd: 1.50,
            turns: 5,
            timestamp: Utc::now(),
        }).unwrap();

        assert!(ledger.can_afford_domain("algostaking"));

        // Spend another $1.00 — now over the $2 domain cap.
        ledger.record(CostEntry {
            domain: "algostaking".into(),
            quest_id: "as-002".into(),
            spirit: "w2".into(),
            cost_usd: 1.00,
            turns: 3,
            timestamp: Utc::now(),
        }).unwrap();

        assert!(!ledger.can_afford_domain("algostaking"));
        // Global budget is still fine.
        assert!(ledger.can_afford());
    }

    #[test]
    fn test_domain_budget_does_not_affect_other_domains() {
        let ledger = CostLedger::new(100.0);
        ledger.set_domain_budget("algostaking", 1.0);

        // Exhaust algostaking's budget.
        ledger.record(CostEntry {
            domain: "algostaking".into(),
            quest_id: "as-001".into(),
            spirit: "w1".into(),
            cost_usd: 2.0,
            turns: 5,
            timestamp: Utc::now(),
        }).unwrap();

        // algostaking is blocked.
        assert!(!ledger.can_afford_domain("algostaking"));
        // riftdecks (no domain budget) is still fine.
        assert!(ledger.can_afford_domain("riftdecks"));
    }

    #[test]
    fn test_domain_without_budget_uses_global() {
        let ledger = CostLedger::new(5.0); // Global: $5/day
        // No domain budget set for "riftdecks".

        ledger.record(CostEntry {
            domain: "riftdecks".into(),
            quest_id: "rd-001".into(),
            spirit: "w1".into(),
            cost_usd: 3.0,
            turns: 5,
            timestamp: Utc::now(),
        }).unwrap();

        // Under global budget — still affordable.
        assert!(ledger.can_afford_domain("riftdecks"));

        // Exceed global budget.
        ledger.record(CostEntry {
            domain: "riftdecks".into(),
            quest_id: "rd-002".into(),
            spirit: "w2".into(),
            cost_usd: 3.0,
            turns: 5,
            timestamp: Utc::now(),
        }).unwrap();

        // Global budget exceeded — no domain can afford.
        assert!(!ledger.can_afford_domain("riftdecks"));
        assert!(!ledger.can_afford_domain("algostaking"));
    }

    #[test]
    fn test_domain_budget_status() {
        let ledger = CostLedger::new(100.0);
        ledger.set_domain_budget("algostaking", 10.0);

        ledger.record(CostEntry {
            domain: "algostaking".into(),
            quest_id: "as-001".into(),
            spirit: "w1".into(),
            cost_usd: 3.50,
            turns: 5,
            timestamp: Utc::now(),
        }).unwrap();

        let (spent, budget, remaining) = ledger.domain_budget_status("algostaking");
        assert!((spent - 3.50).abs() < 0.01);
        assert!((budget - 10.0).abs() < 0.01);
        assert!((remaining - 6.50).abs() < 0.01);

        // Domain without a budget returns global budget.
        let (spent, budget, _remaining) = ledger.domain_budget_status("riftdecks");
        assert!((spent).abs() < 0.01);
        assert!((budget - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_all_domain_budget_statuses() {
        let ledger = CostLedger::new(100.0);
        ledger.set_domain_budget("algostaking", 10.0);
        ledger.set_domain_budget("riftdecks", 5.0);

        ledger.record(CostEntry {
            domain: "algostaking".into(),
            quest_id: "as-001".into(),
            spirit: "w1".into(),
            cost_usd: 2.0,
            turns: 5,
            timestamp: Utc::now(),
        }).unwrap();

        ledger.record(CostEntry {
            domain: "sigil".into(),
            quest_id: "sg-001".into(),
            spirit: "w1".into(),
            cost_usd: 1.0,
            turns: 3,
            timestamp: Utc::now(),
        }).unwrap();

        let statuses = ledger.all_domain_budget_statuses();

        // algostaking: has budget + spending.
        let (spent, budget, remaining) = statuses["algostaking"];
        assert!((spent - 2.0).abs() < 0.01);
        assert!((budget - 10.0).abs() < 0.01);
        assert!((remaining - 8.0).abs() < 0.01);

        // riftdecks: has budget, no spending.
        let (spent, budget, remaining) = statuses["riftdecks"];
        assert!((spent).abs() < 0.01);
        assert!((budget - 5.0).abs() < 0.01);
        assert!((remaining - 5.0).abs() < 0.01);

        // sigil: no budget set, but has spending — uses global budget.
        let (spent, budget, _remaining) = statuses["sigil"];
        assert!((spent - 1.0).abs() < 0.01);
        assert!((budget - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_global_budget_blocks_even_with_domain_headroom() {
        let ledger = CostLedger::new(5.0); // Tight global budget
        ledger.set_domain_budget("algostaking", 50.0); // Generous domain budget

        // Spend enough to exhaust global but not domain.
        ledger.record(CostEntry {
            domain: "algostaking".into(),
            quest_id: "as-001".into(),
            spirit: "w1".into(),
            cost_usd: 6.0,
            turns: 10,
            timestamp: Utc::now(),
        }).unwrap();

        // Domain has headroom ($6 of $50) but global is exceeded ($6 of $5).
        assert!(!ledger.can_afford_domain("algostaking"));
        assert!(!ledger.can_afford());
    }
}
