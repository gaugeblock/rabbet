//! Canonical wire contracts and domain models for `rabbet`.

use gaugeblock_contracts::Language;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Telemetry vocabulary lives in `gaugeblock-contracts` (GB-174); re-exported for existing paths.
pub use gaugeblock_contracts::{AnonymizedSynthesisTrace, TelemetryTier};

// Diagnostics and verification live in `gaugeblock-contracts` (GB-174); re-exported for existing paths.
pub use gaugeblock_contracts::{CompilerDiagnostic, VerificationReport};

// The outbox record lives in `gaugeblock-contracts` (GB-182) so every writer and reader agrees.
pub use gaugeblock_contracts::{SpoolEntry, SyncStatus};

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

/// Serde helper for `Option<Language>` over the wire type.
pub mod option_language_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes `Option<Language>` through the wire representation.
    ///
    /// # Errors
    ///
    /// Propagates any error from the underlying serializer.
    pub fn serialize<S>(
        lang: &Option<Language>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        lang.serialize(serializer)
    }

    /// Deserializes `Option<Language>` through the wire representation.
    ///
    /// # Errors
    ///
    /// Propagates any error from the underlying deserializer.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Option<Language>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Language>::deserialize(deserializer)
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
