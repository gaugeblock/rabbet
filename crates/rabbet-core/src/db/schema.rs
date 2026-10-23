//! SQLite DDL schema, migrations, and index definitions for `rabbet`.

use crate::error::Result;
use rusqlite::Connection;

/// DDL for the primary synthesis trace master store table.
pub const TRACES_TABLE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS traces (
    content_hash TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    language TEXT NOT NULL,
    tier TEXT NOT NULL,
    selected_pattern_id TEXT,
    anonymized_scope_signature TEXT NOT NULL,
    filled_holes_json TEXT NOT NULL,
    rejected_attempt TEXT,
    verification_json TEXT NOT NULL,
    is_synthetic BOOLEAN NOT NULL DEFAULT 0,
    source_repo TEXT,
    created_at_ms INTEGER NOT NULL
);
"#;

/// DDL for query optimization indices.
pub const TRACES_INDICES_DDL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_traces_tier ON traces(tier);
CREATE INDEX IF NOT EXISTS idx_traces_language ON traces(language);
CREATE INDEX IF NOT EXISTS idx_traces_created ON traces(created_at_ms);
CREATE INDEX IF NOT EXISTS idx_traces_synthetic ON traces(is_synthetic);
"#;

/// DDL for mined candidate patterns table.
pub const CANDIDATE_PATTERNS_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS pattern_candidates (
    candidate_id TEXT PRIMARY KEY,
    language TEXT NOT NULL,
    description TEXT NOT NULL,
    template TEXT NOT NULL,
    typed_holes_json TEXT NOT NULL,
    occurrence_count INTEGER NOT NULL,
    example_snippets_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at_ms INTEGER NOT NULL
);
"#;

/// Initializes database pragmas and executes all migrations on the connection.
pub fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        "#,
    )?;

    conn.execute(TRACES_TABLE_DDL, [])?;
    conn.execute_batch(TRACES_INDICES_DDL)?;
    conn.execute(CANDIDATE_PATTERNS_DDL, [])?;

    Ok(())
}
