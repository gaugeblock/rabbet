# Changelog

All notable changes to `rabbet` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.1.0-beta.4] - 2026-05-09

### Changed

- Elevated the 9-document architecture to gold standard in `rabbet` (GB-116).

### Documentation
- Elevate all 9 canonical documents to gold standard matching suite specification (`GB-116`).
- Codify 4-stage ingestion gate invariants and 25 boundary type contracts in `SPEC.md`.
- Document complete CLI command and maintainer subcommand matrix in `README.md`.
- Expand architecture topologies, Litestream replication, and 5 concrete ADRs in `ARCHITECTURE.md` and `DESIGN.md`.


## [v0.1.0-beta.3] - 2026-05-09

### Added
- No additions in this release.

### Fixed
- No fixes in this release.

### Changed
- No user-facing changes in this release.

### Maintenance
- Initialize local backlog store with `RB` prefix for task tracking. (`fce65b5`)
- Standardize README and establish 9-doc architecture for consistent project documentation. (`2e38ffb`)
- Update release history with alpha and beta milestones. (`54fe4dd`)

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

