//! Unit tests for Parquet SFT and DPO shard dataset export.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use miter_engine::Language;
use parquet::file::reader::{FileReader, SerializedFileReader};
use rabbet_core::db::MasterStore;
use rabbet_core::gate::compute_canonical_trace_hash;
use rabbet_core::model::{
    AnonymizedSynthesisTrace, CompilerDiagnostic, SpoolEntry, TelemetryTier, VerificationReport,
};
use rabbet_core::parquet::export_parquet_shards;
use std::fs::File;
use tempfile::tempdir;
use uuid::Uuid;

fn insert_sample_traces(store: &MasterStore) {
    // 1. Bronze trace with pattern
    let t1 = AnonymizedSynthesisTrace {
        trace_id: Uuid::new_v4(),
        language: Language::Rust,
        tier: TelemetryTier::Bronze,
        selected_pattern_id: Some("pattern_iterator".to_string()),
        anonymized_scope_signature: "fn iter_items()".to_string(),
        filled_holes: vec![("hole_iter".to_string(), "items.into_iter()".to_string())],
        rejected_attempt: None,
        verification: VerificationReport {
            ast_valid: true,
            typecheck_passed: true,
            tests_passed: Some(true),
            diagnostics: Vec::new(),
            duration_ms: 10,
        },
    };
    let h1 = compute_canonical_trace_hash(&t1).expect("h1");
    store
        .insert_trace(
            &SpoolEntry {
                trace: t1,
                content_hash: h1,
                sync_status: "synced".to_string(),
                recorded_at_ms: 10,
            },
            false,
            None,
        )
        .expect("insert t1");

    // 2. Gold trace with rejected attempt (DPO candidate)
    let t2 = AnonymizedSynthesisTrace {
        trace_id: Uuid::new_v4(),
        language: Language::Rust,
        tier: TelemetryTier::Gold,
        selected_pattern_id: Some("pattern_oracle_repair".to_string()),
        anonymized_scope_signature: "fn repair_me() -> bool".to_string(),
        filled_holes: vec![("hole_cond".to_string(), "true".to_string())],
        rejected_attempt: Some("false".to_string()),
        verification: VerificationReport {
            ast_valid: true,
            typecheck_passed: true,
            tests_passed: Some(true),
            diagnostics: vec![CompilerDiagnostic {
                file: "repair.rs".to_string(),
                line: 1,
                column: 1,
                code: None,
                message: "assertion failed: expected true, got false".to_string(),
                suggestion: None,
            }],
            duration_ms: 20,
        },
    };
    let h2 = compute_canonical_trace_hash(&t2).expect("h2");
    store
        .insert_trace(
            &SpoolEntry {
                trace: t2,
                content_hash: h2,
                sync_status: "synced".to_string(),
                recorded_at_ms: 20,
            },
            false,
            None,
        )
        .expect("insert t2");
}

#[test]
fn test_parquet_export_shards() {
    let store = MasterStore::open_in_memory().expect("in-memory db");
    insert_sample_traces(&store);

    let temp_dir = tempdir().expect("tempdir");
    let result = export_parquet_shards(&store, temp_dir.path()).expect("export");

    assert_eq!(result.router_sft_rows, 2);
    assert_eq!(result.hole_filler_sft_rows, 2);
    assert_eq!(result.hole_filler_dpo_rows, 1);

    // Verify router_sft.parquet
    let f_router = File::open(&result.router_sft_path).expect("open router");
    let reader_router = SerializedFileReader::new(f_router).expect("read router");
    assert_eq!(reader_router.metadata().file_metadata().num_rows(), 2);

    // Verify hole_filler_sft.parquet
    let f_filler = File::open(&result.hole_filler_sft_path).expect("open filler");
    let reader_filler = SerializedFileReader::new(f_filler).expect("read filler");
    assert_eq!(reader_filler.metadata().file_metadata().num_rows(), 2);

    // Verify hole_filler_dpo.parquet
    let f_dpo = File::open(&result.hole_filler_dpo_path).expect("open dpo");
    let reader_dpo = SerializedFileReader::new(f_dpo).expect("read dpo");
    assert_eq!(reader_dpo.metadata().file_metadata().num_rows(), 1);
}
