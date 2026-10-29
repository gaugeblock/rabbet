//! Cross-tool outbox compatibility (`GB-182`).
//!
//! `storypole verify` and `joiner fill` append to the same `outbox.jsonl`. These tests take the bytes
//! each writer produces, read them back exactly as ingestion reads a spool line, and push them
//! through the gate - the regression that `GB-182` recorded was a reader that could only parse one
//! of the two shapes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use gaugeblock_contracts::Language;
use rabbet_core::db::MasterStore;
use rabbet_core::gate::{compute_canonical_trace_hash, process_ingest_batch, verify_trace_hash};
use rabbet_core::model::{
    AnonymizedSynthesisTrace, BatchIngestRequest, SpoolEntry, SyncStatus, TelemetryTier,
    VerificationReport,
};
use uuid::Uuid;

fn report() -> VerificationReport {
    VerificationReport {
        ast_valid: true,
        typecheck_passed: true,
        tests_passed: Some(true),
        diagnostics: vec![],
        duration_ms: 9,
    }
}

/// A trace shaped like one `storypole verify` captures: provenance keys present, Silver tier.
fn storypole_trace() -> AnonymizedSynthesisTrace {
    AnonymizedSynthesisTrace {
        trace_id: Uuid::from_u128(7),
        language: Language::Rust,
        tier: TelemetryTier::Silver,
        selected_pattern_id: None,
        anonymized_scope_signature: "GB-182 - unify the spool record shape".to_string(),
        filled_holes: vec![("spool".to_string(), "<verified>".to_string())],
        rejected_attempt: None,
        verification: report(),
        task_key: Some("GB-182".to_string()),
        head_commit_sha: Some("9d5f83fdc3305312d2e84d6b56378f318daa22aa".to_string()),
    }
}

/// A trace shaped like one `joiner fill` records: no provenance keys, Gold with a rejected attempt.
fn joiner_trace() -> AnonymizedSynthesisTrace {
    AnonymizedSynthesisTrace {
        trace_id: Uuid::from_u128(11),
        language: Language::Rust,
        tier: TelemetryTier::Gold,
        selected_pattern_id: Some("dado.hole_filler.v3".to_string()),
        anonymized_scope_signature: "fn tokenize(input: &str) -> Vec<Token>".to_string(),
        filled_holes: vec![("body".to_string(), "input.split_whitespace()".to_string())],
        rejected_attempt: Some("input.split(' ')".to_string()),
        verification: report(),
        task_key: None,
        head_commit_sha: None,
    }
}

/// Parses `text` the way ingestion reads a spool line, then runs the full gate over the batch.
fn ingest_lines(text: &str) -> (usize, usize, usize) {
    let entries: Vec<SpoolEntry> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<SpoolEntry>(line).expect("line parses as SpoolEntry"))
        .collect();
    let store = MasterStore::open_in_memory().expect("in-memory store");
    let batch = BatchIngestRequest {
        client_version: "rabbet-test 0.1.0".to_string(),
        traces: entries,
    };
    let response = process_ingest_batch(&store, &batch).expect("gate");
    (response.accepted, response.duplicates, response.rejected)
}

#[test]
fn test_both_writer_shapes_clear_the_ingest_gate() {
    let storypole_line = serde_json::to_string(
        &SpoolEntry::from_trace(storypole_trace(), 1_700_000_000_000).expect("storypole entry"),
    )
    .expect("storypole line");

    let mut joiner_entry =
        SpoolEntry::from_trace(joiner_trace(), 1_700_000_000_001).expect("entry");
    joiner_entry.sync_status = SyncStatus::Synced;
    let joiner_line = serde_json::to_string(&joiner_entry).expect("joiner line");

    let (accepted, duplicates, rejected) =
        ingest_lines(&format!("{storypole_line}\n{joiner_line}\n"));
    assert_eq!(accepted, 2, "both writers are ingestible");
    assert_eq!(duplicates, 0);
    assert_eq!(rejected, 0);
}

#[test]
fn test_reingesting_the_same_outbox_is_all_duplicates() {
    let line = serde_json::to_string(&SpoolEntry::from_trace(storypole_trace(), 5).expect("entry"))
        .expect("line");
    let doubled = format!("{line}\n{line}\n");
    let (accepted, duplicates, rejected) = ingest_lines(&doubled);
    assert_eq!(accepted, 1, "content hash deduplicates within one batch");
    assert_eq!(duplicates, 1);
    assert_eq!(rejected, 0);
}

#[test]
fn test_a_bare_trace_line_is_not_ingestible_until_migrated() {
    // What `storypole verify` used to append: the trace with no envelope around it.
    let bare = serde_json::to_string(&storypole_trace()).expect("bare line");
    assert!(
        serde_json::from_str::<SpoolEntry>(&bare).is_err(),
        "the pre-GB-182 shape cannot be read as a spool entry"
    );

    let migrated =
        serde_json::to_string(&SpoolEntry::from_trace(storypole_trace(), 3).expect("entry"))
            .expect("migrated line");
    let (accepted, _, rejected) = ingest_lines(&migrated);
    assert_eq!((accepted, rejected), (1, 0), "after migration it ingests");
}

#[test]
fn test_writer_hash_is_the_hash_the_gate_recomputes() {
    let trace = storypole_trace();
    let entry = SpoolEntry::from_trace(trace.clone(), 42).expect("entry");
    assert_eq!(
        entry.content_hash,
        compute_canonical_trace_hash(&trace).expect("gate hash"),
        "writer and gate must agree byte-for-byte"
    );
    assert!(verify_trace_hash(&entry).is_ok());

    let mut tampered = entry;
    tampered.trace.anonymized_scope_signature.push('!');
    assert!(
        verify_trace_hash(&tampered).is_err(),
        "editing the payload invalidates the recorded hash"
    );
}
