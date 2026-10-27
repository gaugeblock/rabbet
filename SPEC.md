# Behavioral Specifications & Invariants

## Formal Type Contracts

- **`args::AdminArgs`** (`struct`): Arguments for `rabbet admin`.
- **`args::AdminSubcommand`** (`enum`): Admin subcommands.
- **`model::AnonymizedSynthesisTrace`** (`struct`): Anonymized synthesis trace wire contract from `CONTRACTS.md`.
- **`ingest_handler::AppState`** (`struct`): Shared application state injected into Axum route handlers.
- **`model::BatchIngestRequest`** (`struct`): Public batch submission payload for `POST /api/v1/traces`.
- **`model::BatchIngestResponse`** (`struct`): Response returned by `POST /api/v1/traces`.
- **`args::Cli`** (`struct`): Public interface entity
- **`cold_miner::ColdMinerOptions`** (`struct`): Configuration options for the cold-start OSS repository miner.
- **`args::Command`** (`enum`): Available CLI subcommands.
- **`model::CompilerDiagnostic`** (`struct`): A compiler diagnostic message emitted during verification.
- **`model::CorpusStats`** (`struct`): High-level corpus statistics for maintainer dashboard and metrics.
- **`args::ExportParquetArgs`** (`struct`): Arguments for `rabbet export-parquet`.
- **`model::IngestResultDetail`** (`struct`): Detailed result item for each trace evaluated during ingestion.
- **`store::MasterStore`** (`struct`): Thread-safe master store handle for rabbet SQLite database.
- **`args::MineOssArgs`** (`struct`): Arguments for `rabbet mine-oss`.
- **`args::MinePatternsArgs`** (`struct`): Arguments for `rabbet mine-patterns`.
- **`exporter::ParquetExportResult`** (`struct`): Result summary of exporting Parquet dataset shards.
- **`model::PatternCandidate`** (`struct`): Candidate `dado` pattern proposed by `pattern_miner` from Gold traces.
- **`pattern_miner::PatternMinerOptions`** (`struct`): Configuration options for pattern discovery.
- **`args::PublishHubArgs`** (`struct`): Arguments for `rabbet publish-hub`.
