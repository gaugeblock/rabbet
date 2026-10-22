//! # `rabbet-core` — Public Telemetry Corpus Hub & Open-Weight Training Foundry
//!
//! Provides the master SQLite database store, 4-stage ingestion validation gate,
//! Arrow/Parquet dataset compilation, cold-start OSS code miner, and Hugging Face publisher.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod db;
pub mod error;
pub mod gate;
pub mod miner;
pub mod model;
pub mod parquet;
pub mod publish;

pub use db::{MasterStore, generate_litestream_config, initialize_database};
pub use error::{RabbetError, Result};
pub use gate::{
    compute_canonical_trace_hash, process_ingest_batch, validate_trace_structure,
    verify_and_rescrub_trace, verify_trace_hash,
};
pub use miner::{
    ColdMinerOptions, PatternMinerOptions, discover_pattern_candidates, export_dado_pattern_file,
    extract_ast_skeleton, extract_synthetic_traces_from_source, mine_local_repo,
};
pub use model::{
    AnonymizedSynthesisTrace, BatchIngestRequest, BatchIngestResponse, CorpusStats,
    IngestResultDetail, PatternCandidate, SpoolEntry, TelemetryTier, TraceFilter,
};
pub use parquet::{
    ParquetExportResult, export_parquet_shards, hole_filler_dpo_schema, hole_filler_sft_schema,
    router_sft_schema, write_parquet_file,
};
pub use publish::{
    PublishOptions, generate_dataset_card, prepare_release_bundle, publish_dataset_to_hub,
};
