//! Master SQLite store implementing trace ingestion, deduplication, and queries.

use super::queries::{
    insert_pattern_candidate_internal, list_pattern_candidates_internal, query_traces_internal,
    update_candidate_status_internal,
};
use super::schema::initialize_database;
use crate::error::{RabbetError, Result};
use crate::model::{
    AnonymizedSynthesisTrace, CorpusStats, PatternCandidate, SpoolEntry, TraceFilter,
};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Thread-safe master store handle for rabbet SQLite database.
#[derive(Debug, Clone)]
pub struct MasterStore {
    conn: Arc<Mutex<Connection>>,
    db_path: Option<PathBuf>,
}

impl MasterStore {
    /// Opens or creates a SQLite master store at the specified file path.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        initialize_database(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: Some(path.to_path_buf()),
        })
    }

    /// Creates an ephemeral in-memory SQLite store for unit testing.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        initialize_database(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: None,
        })
    }

    /// Inserts a trace into the master store. Returns `true` if inserted, `false` if duplicate.
    pub fn insert_trace(
        &self,
        entry: &SpoolEntry,
        is_synthetic: bool,
        source_repo: Option<&str>,
    ) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| RabbetError::InvalidRequest(e.to_string()))?;
        let filled_holes_json = serde_json::to_string(&entry.trace.filled_holes)?;
        let verification_json = serde_json::to_string(&entry.trace.verification)?;
        let lang_str = entry.trace.language.name().to_string();
        let tier_str = entry.trace.tier.to_string();

        let inserted = conn.execute(
            r#"
            INSERT OR IGNORE INTO traces (
                content_hash, trace_id, language, tier, selected_pattern_id,
                anonymized_scope_signature, filled_holes_json, rejected_attempt,
                verification_json, is_synthetic, source_repo, created_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                entry.content_hash,
                entry.trace.trace_id.to_string(),
                lang_str,
                tier_str,
                entry.trace.selected_pattern_id,
                entry.trace.anonymized_scope_signature,
                filled_holes_json,
                entry.trace.rejected_attempt,
                verification_json,
                is_synthetic,
                source_repo,
                entry.recorded_at_ms as i64,
            ],
        )?;

        Ok(inserted > 0)
    }

    /// Queries summary statistics across tiers, languages, and storage size.
    pub fn get_stats(&self) -> Result<CorpusStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| RabbetError::InvalidRequest(e.to_string()))?;
        let mut stats = CorpusStats::default();

        let mut stmt = conn.prepare("SELECT tier, count(*) FROM traces GROUP BY tier")?;
        let rows = stmt.query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count as usize))
        })?;
        for row in rows {
            let (tier, count) = row?;
            stats.total_traces += count;
            stats.counts_by_tier.insert(tier, count);
        }

        let mut lang_stmt =
            conn.prepare("SELECT language, count(*) FROM traces GROUP BY language")?;
        let lang_rows = lang_stmt.query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count as usize))
        })?;
        for row in lang_rows {
            let (lang, count) = row?;
            stats.counts_by_language.insert(lang, count);
        }

        let mut synth_stmt =
            conn.prepare("SELECT is_synthetic, count(*) FROM traces GROUP BY is_synthetic")?;
        let synth_rows = synth_stmt.query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, bool>(0)?, count as usize))
        })?;
        for row in synth_rows {
            let (is_syn, count) = row?;
            if is_syn {
                stats.synthetic_count = count;
            } else {
                stats.community_count = count;
            }
        }

        let mut dpo_stmt = conn.prepare(
            "SELECT count(*) FROM traces WHERE tier = 'gold' AND rejected_attempt IS NOT NULL",
        )?;
        let dpo_count: i64 = dpo_stmt.query_row([], |row| row.get(0))?;
        stats.dpo_pairs_count = dpo_count as usize;

        if let Some(path) = &self.db_path
            && let Ok(metadata) = std::fs::metadata(path)
        {
            stats.db_size_bytes = metadata.len();
        }

        Ok(stats)
    }

    /// Fetches traces matching the filter criteria.
    pub fn query_traces(
        &self,
        filter: &TraceFilter,
    ) -> Result<Vec<(String, AnonymizedSynthesisTrace)>> {
        query_traces_internal(&self.conn, filter)
    }

    /// Inserts a mined pattern candidate.
    pub fn insert_pattern_candidate(&self, c: &PatternCandidate) -> Result<()> {
        insert_pattern_candidate_internal(&self.conn, c)
    }

    /// Lists pattern candidates optionally filtered by status.
    pub fn list_pattern_candidates(&self, status: Option<&str>) -> Result<Vec<PatternCandidate>> {
        list_pattern_candidates_internal(&self.conn, status)
    }

    /// Updates the review status of a pattern candidate.
    pub fn update_candidate_status(&self, candidate_id: &str, status: &str) -> Result<bool> {
        update_candidate_status_internal(&self.conn, candidate_id, status)
    }
}
