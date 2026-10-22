//! Unit tests for cold-start OSS miner and AST skeleton pattern discovery.

use miter_engine::Language;
use rabbet_core::db::MasterStore;
use rabbet_core::gate::compute_canonical_trace_hash;
use rabbet_core::miner::{
    PatternMinerOptions, discover_pattern_candidates, export_dado_pattern_file,
    extract_ast_skeleton, extract_synthetic_traces_from_source,
};
use rabbet_core::model::{AnonymizedSynthesisTrace, SpoolEntry, TelemetryTier, VerificationReport};
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn test_synthetic_trace_extraction() {
    let code = r#"
pub fn add_numbers(a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum
}

fn process_data(v: Vec<u8>) -> usize {
    v.len()
}
"#;

    let traces = extract_synthetic_traces_from_source(code, Language::Rust);
    assert_eq!(traces.len(), 2);
    assert_eq!(traces[0].language, Language::Rust);
    assert_eq!(traces[0].tier, TelemetryTier::Silver);
    assert!(traces[0].anonymized_scope_signature.contains("add_numbers"));
}

#[test]
fn test_ast_skeleton_and_pattern_discovery() {
    let (template, holes) = extract_ast_skeleton("items.iter().filter(|x| x > 10).map(|x| \"result\").collect()");
    assert!(template.contains("?[num_lit_"));
    assert!(template.contains("?[str_lit_"));
    assert_eq!(holes.len(), 2);

    // Insert 3 recurring Gold traces
    let store = MasterStore::open_in_memory().expect("store");
    for i in 0..3 {
        let trace = AnonymizedSynthesisTrace {
            trace_id: Uuid::new_v4(),
            language: Language::Rust,
            tier: TelemetryTier::Gold,
            selected_pattern_id: None,
            anonymized_scope_signature: format!("fn batch_{}(s: &str)", i),
            filled_holes: vec![("expr".to_string(), "values.iter().filter(|v| v > 0).collect()".to_string())],
            rejected_attempt: Some("values.iter().collect()".to_string()),
            verification: VerificationReport {
                ast_valid: true,
                typecheck_passed: true,
                tests_passed: Some(true),
                diagnostics: Vec::new(),
                duration_ms: 0,
            },
        };
        let hash = compute_canonical_trace_hash(&trace).expect("hash");
        store.insert_trace(&SpoolEntry {
            trace,
            content_hash: hash,
            sync_status: "synced".to_string(),
            recorded_at_ms: 100 + i,
        }, false, None).expect("insert");
    }

    let options = PatternMinerOptions { min_frequency: 3 };
    let candidates = discover_pattern_candidates(&store, &options).expect("discover");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].occurrence_count, 3);

    let temp_dir = tempdir().expect("tempdir");
    let yaml_path = export_dado_pattern_file(&candidates[0], temp_dir.path()).expect("export");
    assert!(yaml_path.exists());
    let content = std::fs::read_to_string(&yaml_path).expect("read");
    assert!(content.contains("typed_holes:"));
    assert!(content.contains("frequency_count: 3"));
}
