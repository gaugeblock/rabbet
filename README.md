# Rabbet (`gaugeblock/rabbet`)

> **Public Telemetry Corpus Hub & Open-Weight Training Foundry**  
> Part of the [`gaugeblock`](https://gaugeblock.dev) neurosymbolic code synthesis ecosystem.

`rabbet` is the central collector and fine-tuning foundry for compiler-verified synthesis traces. Operating on the "Federated Edge Clients $\to$ Central Maintainer Foundry" architecture, it ingests anonymized traces from client tooling (`joiner`, `benchwork`), validates payloads through a 4-stage privacy/AST gate, stores ground truth in a single-file SQLite database with Litestream replication, compiles Arrow/Parquet SFT and DPO datasets, clusters recurring Gold traces into new `dado` templates, and publishes open weights and dataset snapshots.

---

## 1. Core Architecture

- **Master Store (SQLite + Litestream):** Single Source of Truth (`rabbet.db`) with WAL mode, unique content hash deduplication, and streaming S3/R2 replication.
- **Public Ingestion Gate (`POST /api/v1/traces`):** 4-stage gate (Hash verification $\to$ PII/Secret re-scrubbing via `miter` $\to$ AST syntax check $\to$ Idempotent DB insert).
- **Parquet Dataset Compactor:** Arrow RecordBatch compaction into 3 canonical shards:
  - `router_sft.parquet`: Lexical scope + intent $\to$ pattern ID.
  - `hole_filler_sft.parquet`: Lexical scope + template hole $\to$ target expression.
  - `hole_filler_dpo.parquet`: Direct Preference Optimization pairs (`prompt`, `chosen`, `rejected`, `compiler_diagnostic`).
- **Cold-Start OSS Miner (`rabbet mine-oss`):** Walks open-source repositories to extract synthetic `(Scope + TypedHole -> Expression)` pairs.
- **AST Skeleton Pattern Miner (`rabbet mine-patterns`):** Discovers recurring structural skeletons in Gold Oracle traces to propose new `.dado` templates.
- **Maintainer Admin Console (`/admin`):** Token-protected embedded web dashboard and REST API (`RABBET_ADMIN_TOKEN`).

---

## 2. CLI Usage

### Start Telemetry Server & Admin Console
```bash
rabbet serve --port 8080 --host 0.0.0.0 --db-path rabbet.db --admin-token "$RABBET_ADMIN_TOKEN"
```

### Mine Synthetic Training Pairs from Open-Source Code
```bash
rabbet mine-oss --path ../miter --max-traces 5000 --db-path rabbet.db
```

### Compact Master Store to Parquet Shards
```bash
rabbet export-parquet --db-path rabbet.db --output-dir dist/parquet
```

### Discover Candidate `dado` Patterns from Gold Traces
```bash
rabbet mine-patterns --db-path rabbet.db --min-frequency 3 --output-dir .dado/patterns
```

### Publish Dataset Bundle to Hugging Face Hub
```bash
rabbet publish-hub --bundle-dir dist/parquet --repo-id gaugeblock/synthesis-corpus --dry-run
```

### Maintainer Admin CLI
```bash
rabbet admin --db-path rabbet.db stats
rabbet admin --db-path rabbet.db compact --output-dir dist/parquet
rabbet admin --db-path rabbet.db candidates
rabbet admin --db-path rabbet.db approve <candidate_id>
```

---

## 3. License

Apache-2.0. Copyright (c) 2026 Colin Wolff & Gaugeblock contributors.
