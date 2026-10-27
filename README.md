# `rabbet` — Telemetry Corpus Hub & Open-Weight Training Foundry

[![Version](https://img.shields.io/badge/version-0.1.0--beta.2-blue.svg)](https://gaugeblock.dev)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![gaugeblock](https://img.shields.io/badge/suite-gaugeblock-orange.svg)](https://github.com/gaugeblock)
[![Documentation](https://img.shields.io/badge/docs-9--doc%20topology-green.svg)](ARCHITECTURE.md)

Part of the **[`gaugeblock`](https://gaugeblock.dev)** neurosymbolic code synthesis ecosystem.

> In carpentry, a **rabbet** (or rebate) is a stepped rectangular groove cut along the edge or face of a workpiece, providing a clean, recessed ledge into which another element fits flush (such as glass in a frame or back panels in cabinetry). In the `gaugeblock` ecosystem, `rabbet` is the central collector and fine-tuning foundry for compiler-verified synthesis traces: operating on a "Federated Edge Clients $\to$ Central Maintainer Foundry" architecture, it provides the secure recess into which anonymized traces from client tools (`joiner`, `benchwork`) are safely seated, scrubbed of PII/secrets, stored in SQLite with Litestream replication, and compacted into Arrow/Parquet SFT and DPO datasets for model fine-tuning.

---

## 1. Architecture Overview

```text
┌────────────────────────────────────────────────────────┐
│ Edge Clients (joiner, benchwork)                       │
│ AnonymizedSynthesisTrace (Bronze / Silver / Gold)      │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ rabbet Ingestion & Foundry Gate                        │
│ - 4-Stage Ingestion Gate: Hash -> Scrub -> AST -> DB   │
│ - Master Store: SQLite WAL + Litestream Replication    │
│ - Dataset Compactor: Arrow RecordBatch -> Parquet      │
│   * router_sft.parquet                                 │
│   * hole_filler_sft.parquet                            │
│   * hole_filler_dpo.parquet                            │
│ - Pattern Miner: Clusters Gold traces into dado packs  │
│ - Maintainer Admin Console & Web Dashboard (/admin)    │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ Open Artifacts: Hugging Face Datasets & Model Weights  │
└────────────────────────────────────────────────────────┘
```

---

## 2. Core Capabilities

* **Master Store (SQLite + Litestream):** Single Source of Truth (`rabbet.db`) with WAL mode, unique content hash deduplication, and streaming S3/R2 replication.
* **Public Ingestion Gate (`POST /api/v1/traces`):** 4-stage gate (Hash verification $\to$ PII/Secret re-scrubbing via `miter` $\to$ AST syntax check $\to$ Idempotent DB insert).
* **Parquet Dataset Compactor:** Arrow RecordBatch compaction into 3 canonical shards:
  * `router_sft.parquet`: Lexical scope + intent $\to$ pattern ID.
  * `hole_filler_sft.parquet`: Lexical scope + template hole $\to$ target expression.
  * `hole_filler_dpo.parquet`: Direct Preference Optimization pairs (`prompt`, `chosen`, `rejected`, `compiler_diagnostic`).
* **Cold-Start OSS Miner (`rabbet mine-oss`):** Walks open-source repositories to extract synthetic `(Scope + TypedHole -> Expression)` pairs.
* **AST Skeleton Pattern Miner (`rabbet mine-patterns`):** Discovers recurring structural skeletons in Gold Oracle traces to propose new `.gaugeblock/dado/patterns` templates.
* **Maintainer Admin Console (`/admin`):** Token-protected embedded web dashboard and REST API (`RABBET_ADMIN_TOKEN`).

---

## 3. Workspace Crate Layout

* **`rabbet`**: Facade crate re-exporting `rabbet-core`.
* **`crates/rabbet-core`**: Core telemetry models, SQLite storage engine, Arrow/Parquet compactor, OSS miner, and pattern extraction algorithms.
* **`crates/rabbet-cli`**: Clap binary entrypoint (`rabbet`) providing ingestion server, dataset export, and maintenance commands.

---

## 4. Installation & CLI Quickstart

```bash
# Install globally via cargo
cargo install --path crates/rabbet-cli

# 1. Start telemetry server and admin console
rabbet serve --port 8080 --host 0.0.0.0 --db-path rabbet.db --admin-token "$RABBET_ADMIN_TOKEN"

# 2. Mine synthetic training pairs from open-source repositories
rabbet mine-oss --path ../miter --max-traces 5000 --db-path rabbet.db

# 3. Compact SQLite master store into Arrow/Parquet shards
rabbet export-parquet --db-path rabbet.db --output-dir dist/parquet

# 4. Discover candidate dado patterns from recurring Gold traces
rabbet mine-patterns --db-path rabbet.db --min-frequency 3 --output-dir .dado/patterns

# 5. Publish dataset bundle to Hugging Face Hub
rabbet publish-hub --bundle-dir dist/parquet --repo-id gaugeblock/synthesis-corpus --dry-run
```

---

## 5. Repository Documentation

`rabbet` adheres strictly to the canonical **9-Document Architecture**:
* **[`CONTRIBUTING.md`](./CONTRIBUTING.md):** Local toolchain prerequisites, dataset generation, and test execution.
* **[`ARCHITECTURE.md`](./ARCHITECTURE.md):** Ingestion gate pipeline, Litestream replication, and Arrow compaction layout.
* **[`SPEC.md`](./SPEC.md):** Telemetry payload wire schemas, privacy scrubbing rules, and Parquet column types.
* **[`AGENTS.md`](./AGENTS.md):** Autonomous systems directives, <=250 LOC limits, and zero-unwrap rules.
* **[`CHANGELOG.md`](./CHANGELOG.md):** Chronological historical release entries and unreleased changes.

---

## 6. Verification & Development Directives

* **File Size Ceiling:** All `.rs` source files must strictly remain under 250 LOC.
* **Tiered Verification Matrix:**
  * Tier 1: `cargo check -p rabbet-core` (<2s).
  * Tier 2: `cargo fmt --check -p rabbet-core && cargo clippy -p rabbet-core && cargo test -p rabbet-core`.
  * Tier 3: `chalkline docs check -d .` before task verification.

---

## 7. License

Licensed under the [Apache License, Version 2.0](LICENSE).
Part of the [`gaugeblock`](https://gaugeblock.dev) suite.
