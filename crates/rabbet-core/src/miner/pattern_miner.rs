//! Pattern discovery and clustering engine from recurring Gold/Oracle synthesis traces.

use super::ast_skeleton::{extract_ast_skeleton, normalize_whitespace_skeleton};
use crate::db::MasterStore;
use crate::error::Result;
use crate::model::{PatternCandidate, TelemetryTier, TraceFilter};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Configuration options for pattern discovery.
#[derive(Debug, Clone)]
pub struct PatternMinerOptions {
    /// Minimum occurrences of a recurring AST skeleton required to propose a pattern.
    pub min_frequency: usize,
}

impl Default for PatternMinerOptions {
    fn default() -> Self {
        Self { min_frequency: 3 }
    }
}

/// Discovers candidate `dado` patterns from Gold traces in the master store.
pub fn discover_pattern_candidates(
    store: &MasterStore,
    options: &PatternMinerOptions,
) -> Result<Vec<PatternCandidate>> {
    let filter = TraceFilter {
        tier: Some(TelemetryTier::Gold),
        ..Default::default()
    };
    let gold_traces = store.query_traces(&filter)?;
    let mut clusters: HashMap<String, (String, Vec<String>, Vec<String>)> = HashMap::new();

    for (_, trace) in gold_traces {
        for (_, expr) in &trace.filled_holes {
            let (template, holes) = extract_ast_skeleton(expr);
            let key = normalize_whitespace_skeleton(&template);

            let entry = clusters
                .entry(key)
                .or_insert_with(|| (template, holes, Vec::new()));
            if entry.2.len() < 5 {
                entry.2.push(expr.clone());
            }
        }
    }

    let mut proposed = Vec::new();
    for (key, (template, holes, examples)) in clusters {
        if examples.len() >= options.min_frequency {
            let mut hasher = Sha256::new();
            hasher.update(key.as_bytes());
            let hash_hex = format!("{:x}", hasher.finalize());
            let candidate_id = format!("mined_rust_{}", &hash_hex[..8]);

            let candidate = PatternCandidate {
                candidate_id,
                language: miter_engine::Language::Rust,
                description: format!(
                    "Discovered recurring pattern with {} observed examples",
                    examples.len()
                ),
                template,
                typed_holes: holes,
                occurrence_count: examples.len(),
                example_snippets: examples,
            };

            store.insert_pattern_candidate(&candidate)?;
            proposed.push(candidate);
        }
    }

    Ok(proposed)
}

/// Exports an approved pattern candidate as a valid `.dado` YAML template file.
pub fn export_dado_pattern_file(
    candidate: &PatternCandidate,
    output_dir: &Path,
) -> Result<std::path::PathBuf> {
    std::fs::create_dir_all(output_dir)?;
    let file_path = output_dir.join(format!("{}.yaml", candidate.candidate_id));

    let yaml = format!(
        r#"# Auto-mined dado AST pattern candidate from Rabbet Gold telemetry
id: "{}"
language: "{:?}"
description: "{}"
frequency_count: {}
template: |
{}
typed_holes:
{}
"#,
        candidate.candidate_id,
        candidate.language,
        candidate.description,
        candidate.occurrence_count,
        indent_text(&candidate.template, 2),
        candidate
            .typed_holes
            .iter()
            .map(|h| format!("  - id: \"{}\"\n    expected_type: \"inferred\"", h))
            .collect::<Vec<_>>()
            .join("\n")
    );

    std::fs::write(&file_path, yaml)?;
    Ok(file_path)
}

fn indent_text(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.lines()
        .map(|l| format!("{}{}", pad, l))
        .collect::<Vec<_>>()
        .join("\n")
}
