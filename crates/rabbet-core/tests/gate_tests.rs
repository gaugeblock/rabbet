//! Unit tests for the 4-stage ingestion validation gate.

use miter_engine::Language;
use rabbet_core::db::MasterStore;
use rabbet_core::gate::{compute_canonical_trace_hash, process_ingest_batch, verify_trace_hash};
use rabbet_core::model::{
    AnonymizedSynthesisTrace, BatchIngestRequest, SpoolEntry, TelemetryTier, VerificationReport,
};
use uuid::Uuid;

fn make_valid_trace() -> AnonymizedSynthesisTrace {
    AnonymizedSynthesisTrace {
        trace_id: Uuid::new_v4(),
        language: Language::Rust,
        tier: TelemetryTier::Bronze,
        selected_pattern_id: Some("pat_01".to_string()),
        anonymized_scope_signature: "fn solve(n: usize) -> usize".to_string(),
        filled_holes: vec![("hole_0".to_string(), "n + 1".to_string())],
        rejected_attempt: None,
        verification: VerificationReport {
            ast_valid: true,
            typecheck_passed: true,
            tests_passed: Some(true),
            diagnostics: Vec::new(),
            duration_ms: 0,
        },
    }
}

#[test]
fn test_hash_verification() {
    let trace = make_valid_trace();
    let computed_hash = compute_canonical_trace_hash(&trace).expect("hash");

    let valid_entry = SpoolEntry {
        trace: trace.clone(),
        content_hash: computed_hash,
        sync_status: "pending".to_string(),
        recorded_at_ms: 100,
    };
    assert!(verify_trace_hash(&valid_entry).is_ok());

    let corrupted_entry = SpoolEntry {
        trace,
        content_hash: "deadbeef00000000deadbeef00000000deadbeef00000000deadbeef00000000".to_string(),
        sync_status: "pending".to_string(),
        recorded_at_ms: 100,
    };
    assert!(verify_trace_hash(&corrupted_entry).is_err());
}

#[test]
fn test_batch_ingest_pipeline() {
    let store = MasterStore::open_in_memory().expect("in-memory db");
    let trace1 = make_valid_trace();
    let hash1 = compute_canonical_trace_hash(&trace1).expect("hash1");

    let mut trace_bad_hash = make_valid_trace();
    trace_bad_hash.anonymized_scope_signature = "fn bad() -> ()".to_string();

    let batch = BatchIngestRequest {
        client_version: "benchwork-test 0.1.0".to_string(),
        traces: vec![
            SpoolEntry {
                trace: trace1.clone(),
                content_hash: hash1.clone(),
                sync_status: "pending".to_string(),
                recorded_at_ms: 200,
            },
            SpoolEntry {
                trace: trace_bad_hash,
                content_hash: "invalid_hash_signature".to_string(),
                sync_status: "pending".to_string(),
                recorded_at_ms: 201,
            },
        ],
    };

    let resp = process_ingest_batch(&store, &batch).expect("process");
    assert_eq!(resp.accepted, 1);
    assert_eq!(resp.rejected, 1);
    assert_eq!(resp.duplicates, 0);

    // Ingest the same batch again -> trace1 should now be duplicate
    let resp2 = process_ingest_batch(&store, &batch).expect("process again");
    assert_eq!(resp2.accepted, 0);
    assert_eq!(resp2.duplicates, 1);
    assert_eq!(resp2.rejected, 1);
}
