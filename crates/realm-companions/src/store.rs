use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use tracing::debug;

use crate::companion::{Companion, Rarity};
use crate::gacha::PityState;

pub struct CompanionStore {
    conn: Mutex<Connection>,
}

impl CompanionStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("failed to open companion DB: {}", path.display()))?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS companions (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                rarity TEXT NOT NULL,
                is_familiar INTEGER NOT NULL DEFAULT 0,
                is_rostered INTEGER NOT NULL DEFAULT 0,
                roster_slot INTEGER,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pity (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                pulls_since_s INTEGER NOT NULL DEFAULT 0,
                pulls_since_a INTEGER NOT NULL DEFAULT 0,
                total_pulls INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS pull_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                companion_id TEXT NOT NULL,
                rarity TEXT NOT NULL,
                pulled_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS fusion_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_a TEXT NOT NULL,
                source_b TEXT NOT NULL,
                result_id TEXT NOT NULL,
                source_rarity TEXT NOT NULL,
                result_rarity TEXT NOT NULL,
                fused_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_companions_rarity ON companions(rarity);
            CREATE INDEX IF NOT EXISTS idx_companions_familiar ON companions(is_familiar);
            CREATE INDEX IF NOT EXISTS idx_companions_roster ON companions(is_rostered);",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn save_companion(&self, companion: &Companion) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let data = serde_json::to_string(companion)?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO companions (id, data, rarity, is_familiar, is_rostered, created_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            rusqlite::params![
                companion.id,
                data,
                companion.rarity.to_string(),
                companion.is_familiar as i32,
                now,
            ],
        )?;

        debug!(id = %companion.id, rarity = %companion.rarity, "companion saved");
        Ok(())
    }

    pub fn get_companion(&self, id: &str) -> Result<Option<Companion>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut stmt = conn.prepare("SELECT data FROM companions WHERE id = ?1")?;
        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })
            .optional()?;

        match result {
            Some(data) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    pub fn list_all(&self) -> Result<Vec<Companion>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut stmt = conn.prepare("SELECT data FROM companions ORDER BY created_at")?;
        let results = stmt
            .query_map([], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|data| serde_json::from_str::<Companion>(&data).ok())
            .collect();
        Ok(results)
    }

    pub fn list_by_rarity(&self, rarity: Rarity) -> Result<Vec<Companion>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut stmt = conn.prepare("SELECT data FROM companions WHERE rarity = ?1 ORDER BY created_at")?;
        let results = stmt
            .query_map(rusqlite::params![rarity.to_string()], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|data| serde_json::from_str::<Companion>(&data).ok())
            .collect();
        Ok(results)
    }

    pub fn get_familiar(&self) -> Result<Option<Companion>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut stmt = conn.prepare("SELECT data FROM companions WHERE is_familiar = 1 LIMIT 1")?;
        let result = stmt
            .query_row([], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })
            .optional()?;

        match result {
            Some(data) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    pub fn set_familiar(&self, companion_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;

        conn.execute("UPDATE companions SET is_familiar = 0, data = json_set(data, '$.is_familiar', json('false')) WHERE is_familiar = 1", [])?;

        let data: String = conn.query_row(
            "SELECT data FROM companions WHERE id = ?1",
            rusqlite::params![companion_id],
            |row| row.get(0),
        )?;

        let mut companion: Companion = serde_json::from_str(&data)?;
        companion.is_familiar = true;
        let updated = serde_json::to_string(&companion)?;

        conn.execute(
            "UPDATE companions SET is_familiar = 1, data = ?1 WHERE id = ?2",
            rusqlite::params![updated, companion_id],
        )?;

        debug!(id = %companion_id, "familiar set");
        Ok(())
    }

    pub fn remove_companion(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        conn.execute("DELETE FROM companions WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn load_pity(&self) -> Result<PityState> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let result = conn
            .query_row(
                "SELECT pulls_since_s, pulls_since_a, total_pulls FROM pity WHERE id = 1",
                [],
                |row| {
                    Ok(PityState {
                        pulls_since_s_or_above: row.get::<_, u32>(0)?,
                        pulls_since_a_or_above: row.get::<_, u32>(1)?,
                        total_pulls: row.get::<_, u64>(2)?,
                    })
                },
            )
            .optional()?;

        Ok(result.unwrap_or_default())
    }

    pub fn save_pity(&self, pity: &PityState) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        conn.execute(
            "INSERT OR REPLACE INTO pity (id, pulls_since_s, pulls_since_a, total_pulls)
             VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![
                pity.pulls_since_s_or_above,
                pity.pulls_since_a_or_above,
                pity.total_pulls as i64,
            ],
        )?;
        Ok(())
    }

    pub fn record_pull(&self, companion: &Companion) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO pull_history (companion_id, rarity, pulled_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![companion.id, companion.rarity.to_string(), now],
        )?;
        Ok(())
    }

    pub fn record_fusion(&self, a: &Companion, b: &Companion, result: &Companion) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO fusion_history (source_a, source_b, result_id, source_rarity, result_rarity, fused_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                a.id,
                b.id,
                result.id,
                a.rarity.to_string(),
                result.rarity.to_string(),
                now,
            ],
        )?;
        Ok(())
    }

    pub fn collection_stats(&self) -> Result<CollectionStats> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;

        let total: u32 = conn.query_row("SELECT COUNT(*) FROM companions", [], |row| row.get(0))?;

        let by_rarity = |r: &str| -> Result<u32> {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM companions WHERE rarity = ?1",
                rusqlite::params![r],
                |row| row.get(0),
            )?)
        };

        let total_pulls: u64 = conn
            .query_row("SELECT COALESCE(total_pulls, 0) FROM pity WHERE id = 1", [], |row| row.get(0))
            .unwrap_or(0);

        let total_fusions: u32 = conn.query_row("SELECT COUNT(*) FROM fusion_history", [], |row| row.get(0))?;

        Ok(CollectionStats {
            total_companions: total,
            c_count: by_rarity("C")?,
            b_count: by_rarity("B")?,
            a_count: by_rarity("A")?,
            s_count: by_rarity("S")?,
            ss_count: by_rarity("SS")?,
            total_pulls,
            total_fusions,
        })
    }

    pub fn companion_count(&self) -> Result<u32> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        Ok(conn.query_row("SELECT COUNT(*) FROM companions", [], |row| row.get(0))?)
    }

    pub fn fusion_eligible_pairs(&self, rarity: Rarity) -> Result<Vec<Companion>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut stmt = conn.prepare(
            "SELECT data FROM companions WHERE rarity = ?1 AND is_familiar = 0 ORDER BY created_at",
        )?;
        let results = stmt
            .query_map(rusqlite::params![rarity.to_string()], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|data| serde_json::from_str::<Companion>(&data).ok())
            .collect();
        Ok(results)
    }
}

use rusqlite::OptionalExtension;

#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub total_companions: u32,
    pub c_count: u32,
    pub b_count: u32,
    pub a_count: u32,
    pub s_count: u32,
    pub ss_count: u32,
    pub total_pulls: u64,
    pub total_fusions: u32,
}

impl std::fmt::Display for CollectionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Collection: {} companions | Pulls: {} | Fusions: {}\n\
             C: {} | B: {} | A: {} | S: {} | SS: {}",
            self.total_companions,
            self.total_pulls,
            self.total_fusions,
            self.c_count,
            self.b_count,
            self.a_count,
            self.s_count,
            self.ss_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gacha::{GachaEngine, PityState as GachaPity};
    use tempfile::TempDir;

    fn temp_store() -> (CompanionStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = CompanionStore::open(&dir.path().join("companions.db")).unwrap();
        (store, dir)
    }

    fn make_companion() -> Companion {
        let engine = GachaEngine::default();
        let mut pity = GachaPity::default();
        engine.pull(&mut pity)
    }

    #[test]
    fn test_save_and_get() {
        let (store, _dir) = temp_store();
        let c = make_companion();
        store.save_companion(&c).unwrap();

        let loaded = store.get_companion(&c.id).unwrap().unwrap();
        assert_eq!(loaded.id, c.id);
        assert_eq!(loaded.rarity, c.rarity);
    }

    #[test]
    fn test_list_all() {
        let (store, _dir) = temp_store();
        for _ in 0..5 {
            store.save_companion(&make_companion()).unwrap();
        }
        let all = store.list_all().unwrap();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_familiar_management() {
        let (store, _dir) = temp_store();
        let mut c1 = make_companion();
        c1.rarity = Rarity::SS;
        c1.familiar_eligible = true;
        store.save_companion(&c1).unwrap();

        let mut c2 = make_companion();
        c2.rarity = Rarity::SS;
        c2.familiar_eligible = true;
        store.save_companion(&c2).unwrap();

        store.set_familiar(&c1.id).unwrap();
        let fam = store.get_familiar().unwrap().unwrap();
        assert_eq!(fam.id, c1.id);
        assert!(fam.is_familiar);

        store.set_familiar(&c2.id).unwrap();
        let fam = store.get_familiar().unwrap().unwrap();
        assert_eq!(fam.id, c2.id);

        let old_fam = store.get_companion(&c1.id).unwrap().unwrap();
        assert!(!old_fam.is_familiar);
    }

    #[test]
    fn test_pity_persistence() {
        let (store, _dir) = temp_store();
        let pity = PityState {
            pulls_since_s_or_above: 15,
            pulls_since_a_or_above: 5,
            total_pulls: 42,
        };
        store.save_pity(&pity).unwrap();

        let loaded = store.load_pity().unwrap();
        assert_eq!(loaded.pulls_since_s_or_above, 15);
        assert_eq!(loaded.pulls_since_a_or_above, 5);
        assert_eq!(loaded.total_pulls, 42);
    }

    #[test]
    fn test_collection_stats() {
        let (store, _dir) = temp_store();
        for _ in 0..3 {
            store.save_companion(&make_companion()).unwrap();
        }
        let stats = store.collection_stats().unwrap();
        assert_eq!(stats.total_companions, 3);
    }
}
