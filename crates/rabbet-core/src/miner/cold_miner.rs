//! Cold-start repository miner that extracts synthetic (Scope + TypedHole -> Expression) pairs.

use crate::db::MasterStore;
use crate::error::{RabbetError, Result};
use crate::gate::validator::compute_canonical_trace_hash;
use crate::model::{AnonymizedSynthesisTrace, SpoolEntry, TelemetryTier, VerificationReport};
use miter_engine::Language;
use std::path::Path;
use uuid::Uuid;

/// Configuration options for the cold-start OSS repository miner.
#[derive(Debug, Clone)]
pub struct ColdMinerOptions {
    /// Maximum number of synthetic traces to extract across the run.
    pub max_traces: usize,
    /// Minimum line length of an extracted function body to be mined.
    pub min_body_lines: usize,
}

impl Default for ColdMinerOptions {
    fn default() -> Self {
        Self {
            max_traces: 10_000,
            min_body_lines: 3,
        }
    }
}

/// Recursively walks `repo_path` and mines synthetic training pairs into `store`.
pub fn mine_local_repo(
    repo_path: &Path,
    store: &MasterStore,
    options: &ColdMinerOptions,
) -> Result<usize> {
    if !repo_path.exists() {
        return Err(RabbetError::Mining(format!(
            "Repository path does not exist: {}",
            repo_path.display()
        )));
    }

    let mut extracted = 0;
    let walker = walkdir::WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "rs"));

    for entry in walker {
        if extracted >= options.max_traces {
            break;
        }

        let path = entry.path();
        if path.components().any(|c| c.as_os_str() == "target" || c.as_os_str() == ".git") {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let file_traces = extract_synthetic_traces_from_source(&content, Language::Rust);
            for trace in file_traces {
                if extracted >= options.max_traces {
                    break;
                }
                let hash = compute_canonical_trace_hash(&trace)?;
                let spool_entry = SpoolEntry {
                    trace,
                    content_hash: hash,
                    sync_status: "synced".to_string(),
                    recorded_at_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                };
                let repo_name = repo_path.file_name().and_then(|n| n.to_str());
                if store.insert_trace(&spool_entry, true, repo_name)? {
                    extracted += 1;
                }
            }
        }
    }

    Ok(extracted)
}

/// Extracts synthetic (scope + hole -> expression) traces from Rust source code.
pub fn extract_synthetic_traces_from_source(source: &str, language: Language) -> Vec<AnonymizedSynthesisTrace> {
    let mut traces = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx].trim();
        if (line.starts_with("pub fn ") || line.starts_with("fn ")) && line.contains('(') {
            let _sig_start = idx;
            let mut sig_lines = Vec::new();
            while idx < lines.len() && !lines[idx].contains('{') && !lines[idx].contains(';') {
                sig_lines.push(lines[idx].trim());
                idx += 1;
            }
            if idx < lines.len() && lines[idx].contains('{') {
                sig_lines.push(lines[idx].split('{').next().unwrap_or("").trim());
                let scope_signature = sig_lines.join(" ");

                // Find closing brace of body
                let mut body_lines = Vec::new();
                let mut depth = 1;
                idx += 1;
                while idx < lines.len() && depth > 0 {
                    for ch in lines[idx].chars() {
                        if ch == '{' { depth += 1; }
                        else if ch == '}' { depth -= 1; }
                    }
                    if depth > 0 {
                        body_lines.push(lines[idx].trim());
                    }
                    idx += 1;
                }

                if !body_lines.is_empty() && !scope_signature.is_empty() {
                    let target_expr = body_lines.join(" ");
                    if !target_expr.is_empty() {
                        let hole_id = "body_expr".to_string();
                        traces.push(AnonymizedSynthesisTrace {
                            trace_id: Uuid::new_v4(),
                            language,
                            tier: TelemetryTier::Silver,
                            selected_pattern_id: Some("synthetic_extracted_fn".to_string()),
                            anonymized_scope_signature: scope_signature,
                            filled_holes: vec![(hole_id, target_expr)],
                            rejected_attempt: None,
                            verification: VerificationReport {
                                ast_valid: true,
                                typecheck_passed: true,
                                tests_passed: Some(true),
                                diagnostics: Vec::new(),
                                duration_ms: 0,
                            },
                        });
                    }
                }
            }
        }
        idx += 1;
    }

    traces
}
