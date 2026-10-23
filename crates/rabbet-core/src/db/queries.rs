//! Trace querying and pattern candidate management for MasterStore.

use crate::error::{RabbetError, Result};
use crate::model::{AnonymizedSynthesisTrace, PatternCandidate, TelemetryTier, TraceFilter};
use miter_engine::Language;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Executes trace queries matching the specified filter criteria.
pub fn query_traces_internal(
    conn: &Arc<Mutex<Connection>>,
    filter: &TraceFilter,
) -> Result<Vec<(String, AnonymizedSynthesisTrace)>> {
    let conn = conn
        .lock()
        .map_err(|e| RabbetError::InvalidRequest(e.to_string()))?;
    let mut query = "SELECT content_hash, trace_id, language, tier, selected_pattern_id, anonymized_scope_signature, filled_holes_json, rejected_attempt, verification_json FROM traces WHERE 1=1".to_string();
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(tier) = &filter.tier {
        query.push_str(" AND tier = ?");
        params_vec.push(rusqlite::types::Value::Text(tier.to_string()));
    }
    if let Some(syn) = filter.is_synthetic {
        query.push_str(" AND is_synthetic = ?");
        params_vec.push(rusqlite::types::Value::Integer(if syn { 1 } else { 0 }));
    }
    if let Some(limit) = filter.limit {
        query.push_str(&format!(" LIMIT {}", limit));
    }
    if let Some(offset) = filter.offset {
        query.push_str(&format!(" OFFSET {}", offset));
    }

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| {
        let hash: String = row.get(0)?;
        let id_str: String = row.get(1)?;
        let lang_str: String = row.get(2)?;
        let tier_str: String = row.get(3)?;
        let pat_id: Option<String> = row.get(4)?;
        let scope: String = row.get(5)?;
        let holes_json: String = row.get(6)?;
        let rejected: Option<String> = row.get(7)?;
        let verif_json: String = row.get(8)?;

        let trace_id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
        let language = match lang_str.as_str() {
            "typescript" => Language::TypeScript,
            "python" => Language::Python,
            _ => Language::Rust,
        };
        let tier = match tier_str.as_str() {
            "silver" => TelemetryTier::Silver,
            "gold" => TelemetryTier::Gold,
            "diamond" => TelemetryTier::Diamond,
            _ => TelemetryTier::Bronze,
        };
        let filled_holes: Vec<(String, String)> =
            serde_json::from_str(&holes_json).unwrap_or_default();
        let verification = serde_json::from_str(&verif_json).unwrap_or_else(|_| {
            crate::model::VerificationReport {
                ast_valid: true,
                typecheck_passed: true,
                tests_passed: Some(true),
                diagnostics: Vec::new(),
                duration_ms: 0,
            }
        });

        Ok((
            hash,
            AnonymizedSynthesisTrace {
                trace_id,
                language,
                tier,
                selected_pattern_id: pat_id,
                anonymized_scope_signature: scope,
                filled_holes,
                rejected_attempt: rejected,
                verification,
            },
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Inserts or replaces a candidate pattern in the database.
pub fn insert_pattern_candidate_internal(
    conn: &Arc<Mutex<Connection>>,
    c: &PatternCandidate,
) -> Result<()> {
    let conn = conn
        .lock()
        .map_err(|e| RabbetError::InvalidRequest(e.to_string()))?;
    let holes_json = serde_json::to_string(&c.typed_holes)?;
    let examples_json = serde_json::to_string(&c.example_snippets)?;
    let lang_str = c.language.name().to_string();

    conn.execute(
        r#"
        INSERT OR REPLACE INTO pattern_candidates (
            candidate_id, language, description, template, typed_holes_json,
            occurrence_count, example_snippets_json, status, created_at_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)
        "#,
        params![
            c.candidate_id,
            lang_str,
            c.description,
            c.template,
            holes_json,
            c.occurrence_count as i64,
            examples_json,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        ],
    )?;
    Ok(())
}

/// Lists pattern candidates optionally filtered by review status.
pub fn list_pattern_candidates_internal(
    conn: &Arc<Mutex<Connection>>,
    status: Option<&str>,
) -> Result<Vec<PatternCandidate>> {
    let conn = conn
        .lock()
        .map_err(|e| RabbetError::InvalidRequest(e.to_string()))?;
    let query = if let Some(st) = status {
        format!(
            "SELECT candidate_id, language, description, template, typed_holes_json, occurrence_count, example_snippets_json FROM pattern_candidates WHERE status = '{}' ORDER BY occurrence_count DESC",
            st
        )
    } else {
        "SELECT candidate_id, language, description, template, typed_holes_json, occurrence_count, example_snippets_json FROM pattern_candidates ORDER BY occurrence_count DESC".to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let lang_str: String = row.get(1)?;
        let desc: String = row.get(2)?;
        let tmpl: String = row.get(3)?;
        let holes_json: String = row.get(4)?;
        let count: i64 = row.get(5)?;
        let ex_json: String = row.get(6)?;

        let language = match lang_str.as_str() {
            "typescript" => Language::TypeScript,
            "python" => Language::Python,
            _ => Language::Rust,
        };
        let typed_holes: Vec<String> = serde_json::from_str(&holes_json).unwrap_or_default();
        let example_snippets: Vec<String> = serde_json::from_str(&ex_json).unwrap_or_default();

        Ok(PatternCandidate {
            candidate_id: id,
            language,
            description: desc,
            template: tmpl,
            typed_holes,
            occurrence_count: count as usize,
            example_snippets,
        })
    })?;

    let mut candidates = Vec::new();
    for r in rows {
        candidates.push(r?);
    }
    Ok(candidates)
}

/// Updates candidate review status.
pub fn update_candidate_status_internal(
    conn: &Arc<Mutex<Connection>>,
    candidate_id: &str,
    status: &str,
) -> Result<bool> {
    let conn = conn
        .lock()
        .map_err(|e| RabbetError::InvalidRequest(e.to_string()))?;
    let updated = conn.execute(
        "UPDATE pattern_candidates SET status = ?1 WHERE candidate_id = ?2",
        params![status, candidate_id],
    )?;
    Ok(updated > 0)
}
