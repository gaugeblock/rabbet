//! Dataset card generator and version metadata packaging for Hugging Face Hub.

use crate::model::CorpusStats;
use std::path::{Path, PathBuf};

/// Generates a standard Hugging Face dataset card `README.md` with dataset metadata.
pub fn generate_dataset_card(stats: &CorpusStats, version: &str) -> String {
    let bronze = stats.counts_by_tier.get("bronze").copied().unwrap_or(0);
    let silver = stats.counts_by_tier.get("silver").copied().unwrap_or(0);
    let gold = stats.counts_by_tier.get("gold").copied().unwrap_or(0);
    let diamond = stats.counts_by_tier.get("diamond").copied().unwrap_or(0);

    format!(
        r#"---
annotations_creators:
- machine-generated
language:
- en
- code
license: apache-2.0
multilinguality:
- multilinguistic
size_categories:
- 10K<n<100K
source_datasets:
- gaugeblock-telemetry
tags:
- compiler
- neurosymbolic
- code-synthesis
- dpo
- rust
task_categories:
- text-generation
pretty_name: Gaugeblock Compiler-Verified Synthesis Corpus
---

# Gaugeblock Synthesis Corpus (`{version}`)

Official public telemetry and synthetic compilation traces exported by the [`gaugeblock/rabbet`](https://gaugeblock.dev) telemetry foundry.

## Dataset Summary

- **Total Unique Traces:** {total}
- **Bronze (Compiled attempt #1):** {bronze}
- **Silver (Passed unit tests #1):** {silver}
- **Gold (Oracle repair / DPO pairs):** {gold}
- **Diamond (Survived in git main):** {diamond}
- **Synthetic OSS Mined:** {synthetic}
- **Community Contributed:** {community}

## Shard Files

1. `router_sft.parquet`: Lexical scope + intent $\to$ `dado` pattern ID.
2. `hole_filler_sft.parquet`: Lexical scope + template hole $\to$ verified expression.
3. `hole_filler_dpo.parquet`: Direct Preference Optimization pairs (`prompt`, `chosen`, `rejected`, `compiler_diagnostic`).

## Privacy & Sanitization

All source traces are sanitized with `miter-engine`'s AST-aware entropy filter before inclusion. Zero proprietary literals, IP addresses, credentials, or high-entropy tokens are present.
"#,
        version = version,
        total = stats.total_traces,
        bronze = bronze,
        silver = silver,
        gold = gold,
        diamond = diamond,
        synthetic = stats.synthetic_count,
        community = stats.community_count,
    )
}

/// Prepares a release bundle directory with dataset card and Parquet shards.
pub fn prepare_release_bundle(
    export_dir: &Path,
    stats: &CorpusStats,
    version: &str,
) -> std::io::Result<PathBuf> {
    let readme_path = export_dir.join("README.md");
    let content = generate_dataset_card(stats, version);
    std::fs::write(&readme_path, content)?;
    Ok(readme_path)
}

