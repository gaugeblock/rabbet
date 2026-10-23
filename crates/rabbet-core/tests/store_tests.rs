//! Unit tests for SQLite MasterStore and deduplication queries.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use miter_engine::Language;
use rabbet_core::db::MasterStore;
use rabbet_core::gate::compute_canonical_trace_hash;
use rabbet_core::model::{
    AnonymizedSynthesisTrace, PatternCandidate, SpoolEntry, TelemetryTier, TraceFilter,
    VerificationReport,
};
use uuid::Uuid;

fn make_sample_trace(tier: TelemetryTier, pattern: Option<&str>) -> AnonymizedSynthesisTrace {
    AnonymizedSynthesisTrace {
        trace_id: Uuid::new_v4(),
        language: Language::Rust,
        tier,
        selected_pattern_id: pattern.map(String::from),
        anonymized_scope_signature: "fn calculate(x: i32) -> i32".to_string(),
        filled_holes: vec![("hole_0".to_string(), "x * 2".to_string())],
        rejected_attempt: if tier == TelemetryTier::Gold {
            Some("x + 1".to_string())
        } else {
            None
        },
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
fn test_store_insert_and_deduplication() {
    let store = MasterStore::open_in_memory().expect("in-memory db");
    let trace = make_sample_trace(TelemetryTier::Silver, Some("pattern_calc"));
    let hash = compute_canonical_trace_hash(&trace).expect("hash");

    let entry = SpoolEntry {
        trace: trace.clone(),
        content_hash: hash.clone(),
        sync_status: "pending".to_string(),
        recorded_at_ms: 1000,
    };

    // First insert: succeeds
    let inserted = store.insert_trace(&entry, false, None).expect("insert");
    assert!(inserted, "First insert should return true");

    // Duplicate insert with same content_hash: ignored
    let dup = store
        .insert_trace(&entry, false, None)
        .expect("duplicate insert");
    assert!(!dup, "Duplicate insert should return false");

    // Verify stats
    let stats = store.get_stats().expect("stats");
    assert_eq!(stats.total_traces, 1);
    assert_eq!(stats.community_count, 1);
    assert_eq!(stats.synthetic_count, 0);
    assert_eq!(stats.counts_by_tier.get("silver"), Some(&1));
}

#[test]
fn test_store_query_and_candidates() {
    let store = MasterStore::open_in_memory().expect("in-memory db");
    let gold_trace = make_sample_trace(TelemetryTier::Gold, None);
    let hash = compute_canonical_trace_hash(&gold_trace).expect("hash");

    let entry = SpoolEntry {
        trace: gold_trace,
        content_hash: hash,
        sync_status: "pending".to_string(),
        recorded_at_ms: 2000,
    };
    store
        .insert_trace(&entry, true, Some("gaugeblock/joiner"))
        .expect("insert");

    let stats = store.get_stats().expect("stats");
    assert_eq!(stats.total_traces, 1);
    assert_eq!(stats.dpo_pairs_count, 1);
    assert_eq!(stats.synthetic_count, 1);

    let filter = TraceFilter {
        tier: Some(TelemetryTier::Gold),
        ..Default::default()
    };
    let traces = store.query_traces(&filter).expect("query");
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].1.tier, TelemetryTier::Gold);

    // Pattern candidate insertion & listing
    let cand = PatternCandidate {
        candidate_id: "test_pattern_01".to_string(),
        language: Language::Rust,
        description: "Test description".to_string(),
        template: "fn test() { ?[body: ()] }".to_string(),
        typed_holes: vec!["body".to_string()],
        occurrence_count: 5,
        example_snippets: vec!["println!(\"hello\");".to_string()],
    };
    store
        .insert_pattern_candidate(&cand)
        .expect("insert candidate");

    let candidates = store.list_pattern_candidates(None).expect("list");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].candidate_id, "test_pattern_01");

    let updated = store
        .update_candidate_status("test_pattern_01", "approved")
        .expect("update");
    assert!(updated);
}
