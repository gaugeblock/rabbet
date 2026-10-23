//! Canonical wire contracts and domain models for `rabbet`.

use miter_engine::Language;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Quality tiers defined in `CONTRACTS.md` for compiler-verified synthesis telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryTier {
    /// Compiled and typechecked on attempt #1.
    Bronze,
    /// Passed unit tests on attempt #1.
    Silver,
    /// Required repair or Oracle fallback -> DPO training pair + pattern candidate.
    Gold,
    /// Survived unchanged in git main across subsequent commits.
    Diamond,
}

impl std::fmt::Display for TelemetryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bronze => write!(f, "bronze"),
            Self::Silver => write!(f, "silver"),
            Self::Gold => write!(f, "gold"),
            Self::Diamond => write!(f, "diamond"),
        }
    }
}

/// A compiler diagnostic message emitted during verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompilerDiagnostic {
    /// Relative file path.
    pub file: String,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// Tool-specific error code (e.g. `E0308`).
    pub code: Option<String>,
    /// Error message string.
    pub message: String,
    /// Optional compiler suggestion.
    pub suggestion: Option<String>,
}

/// Detailed compiler and test verification output matching `CONTRACTS.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerificationReport {
    /// Whether the synthesized source parsed into a valid AST.
    pub ast_valid: bool,
    /// Whether static typecheck/compilation succeeded.
    pub typecheck_passed: bool,
    /// Whether cargo/project unit tests passed, if executed.
    pub tests_passed: Option<bool>,
    /// Located diagnostics emitted by the compiler or linter.
    pub diagnostics: Vec<CompilerDiagnostic>,
    /// Elapsed verification duration in milliseconds.
    pub duration_ms: u64,
}

/// Anonymized synthesis trace wire contract from `CONTRACTS.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymizedSynthesisTrace {
    /// Unique identifier for this synthesis trace.
    pub trace_id: Uuid,
    /// Target programming language.
    #[serde(with = "dado_core::language_serde")]
    pub language: Language,
    /// Dynamic verification tier assigned upon completion.
    pub tier: TelemetryTier,
    /// Identified `dado` pattern ID, if matched by System 1 router.
    pub selected_pattern_id: Option<String>,
    /// Scrubbed lexical scope signature.
    pub anonymized_scope_signature: String,
    /// Concrete expressions synthesized into typed holes: (hole_id, anonymized_expression).
    pub filled_holes: Vec<(String, String)>,
    /// Previous failed synthesis attempt, present on Gold DPO traces.
    pub rejected_attempt: Option<String>,
    /// Compiler verification output.
    pub verification: VerificationReport,
}

/// Local outbox spool entry matching `joiner-core`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpoolEntry {
    /// The anonymized trace payload.
    pub trace: AnonymizedSynthesisTrace,
    /// Deterministic SHA-256 hash of the canonical trace JSON.
    pub content_hash: String,
    /// Current synchronization state.
    pub sync_status: String,
    /// Timestamp in epoch milliseconds when spooled.
    pub recorded_at_ms: u64,
}

/// Public batch submission payload for `POST /api/v1/traces`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchIngestRequest {
    /// Client version identifier (e.g. `benchwork 0.1.0-beta.1`).
    pub client_version: String,
    /// List of spooled entries to ingest.
    pub traces: Vec<SpoolEntry>,
}

/// Detailed result item for each trace evaluated during ingestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestResultDetail {
    /// Content hash of the trace.
    pub content_hash: String,
    /// Trace UUID.
    pub trace_id: Uuid,
    /// Outcome status: "accepted", "duplicate", or "rejected".
    pub status: String,
    /// Optional rejection reason.
    pub reason: Option<String>,
}

/// Response returned by `POST /api/v1/traces`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchIngestResponse {
    /// Count of newly inserted traces.
    pub accepted: usize,
    /// Count of already-existing traces ignored via content hash deduplication.
    pub duplicates: usize,
    /// Count of rejected traces (hash mismatch, PII, invalid syntax).
    pub rejected: usize,
    /// Per-trace validation details.
    pub details: Vec<IngestResultDetail>,
}

/// High-level corpus statistics for maintainer dashboard and metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CorpusStats {
    /// Total count of unique traces.
    pub total_traces: usize,
    /// Counts broken down by quality tier (bronze, silver, gold, diamond).
    pub counts_by_tier: std::collections::BTreeMap<String, usize>,
    /// Counts broken down by language.
    pub counts_by_language: std::collections::BTreeMap<String, usize>,
    /// Total synthetic traces extracted from OSS mining.
    pub synthetic_count: usize,
    /// Total community-contributed traces.
    pub community_count: usize,
    /// Total DPO pairs available (Gold traces with rejected attempts).
    pub dpo_pairs_count: usize,
    /// Master SQLite database file size in bytes.
    pub db_size_bytes: u64,
}

/// Serde helper for `Option<Language>`.
pub mod option_language_serde {
    use super::*;
    use miter_engine::Language;
    use serde::{Deserializer, Serializer};

    /// Serializes `Option<Language>`.
    pub fn serialize<S>(
        lang: &Option<Language>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match lang {
            Some(l) => serializer.serialize_some(l.name()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes `Option<Language>`.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Option<Language>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => match s.to_lowercase().as_str() {
                "rust" | "rs" => Ok(Some(Language::Rust)),
                "typescript" | "ts" => Ok(Some(Language::TypeScript)),
                "tsx" => Ok(Some(Language::Tsx)),
                "javascript" | "js" => Ok(Some(Language::JavaScript)),
                "python" | "py" => Ok(Some(Language::Python)),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }
}

/// Query filter for querying traces from the master store.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TraceFilter {
    /// Optional filter by language.
    #[serde(default, with = "option_language_serde")]
    pub language: Option<Language>,
    /// Optional filter by quality tier.
    pub tier: Option<TelemetryTier>,
    /// Filter for synthetic vs community traces.
    pub is_synthetic: Option<bool>,
    /// Optional maximum number of traces to fetch.
    pub limit: Option<usize>,
    /// Optional offset for pagination.
    pub offset: Option<usize>,
}

/// Candidate `dado` pattern proposed by `pattern_miner` from Gold traces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternCandidate {
    /// Proposed pattern ID (e.g. `mined_rust_iterator_filter_01`).
    pub candidate_id: String,
    /// Target language.
    #[serde(with = "dado_core::language_serde")]
    pub language: Language,
    /// Human-readable description.
    pub description: String,
    /// Inferred AST template with typed holes (`?[hole: type]`).
    pub template: String,
    /// Concrete typed holes extracted.
    pub typed_holes: Vec<String>,
    /// Number of distinct occurrences clustered from Gold traces.
    pub occurrence_count: usize,
    /// Example concrete code snippets observed.
    pub example_snippets: Vec<String>,
}
