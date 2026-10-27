# Changelog

All notable changes to `rabbet` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Documentation
- Standardize README to 9-section template and establish AST-grounded 9-document architecture

## [0.1.0-beta.2] - 2026-10-24

### Changed
- `rabbet-core`: upgrade `arrow` and `parquet` to 60.0 and `rusqlite` to 0.40.2.
- `rabbet-cli`: prune `tokio` features to minimal runtime dependencies and harmonize suite versions.

## [0.1.0-beta.1] - 2026-10-22

### Added
- `rabbet-core`: SQLite master store with WAL mode and Litestream continuous replication.
- `rabbet-core`: reverse AST cold miner extracting synthetic `(Scope + TypedHole -> TargetExpression)` training pairs using `miter` and `dado`.
- `rabbet-core`: ingestion gate with double-checked SHA-256 verification and secret/PII scrubber preventing contaminated telemetry egress.
- `rabbet-core`: Arrow RecordBatch Parquet exporter generating `router_sft.parquet`, `hole_filler_sft.parquet`, and `hole_filler_dpo.parquet` dataset shards.
- `rabbet-core`: recurring Gold trace pattern clustering engine to discover candidate deterministic `dado` patterns.
- `rabbet-cli`: public ingestion gate server (`rabbet serve`), token-authenticated Maintainer Admin Console (`/admin`), and dataset subcommands (`mine-oss`, `export-parquet`, `publish-hub`, `admin`).

## [0.0.1-alpha.1] - 2026-10-21

### Added
- Initial namespace reservation on crates.io and GitHub repository scaffold.

