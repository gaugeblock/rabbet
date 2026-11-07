# `rabbet` — Architectural Design & Decisions

> **Domain:** Telemetry Corpus Hub & Training Foundry | **Suite:** `gaugeblock`

This document captures the key Architectural Decision Records (ADRs) governing `rabbet`.

---

## Architectural Decision Records (ADRs)

### ADR-001: SQLite Master Store with Litestream Streaming Replication
* **Status:** Accepted
* **Context:** `rabbet` ingests telemetry from edge clients and must support low-latency append-only writes, unique hash lookups, and minimal infrastructure overhead for self-hosting.
* **Decision:** Use an embedded SQLite database (`rabbet.db`) in Write-Ahead Logging (`WAL`) mode backed by Litestream continuous WAL replication to S3/R2 object storage instead of hosting a managed PostgreSQL cluster.
* **Consequences:** Eliminates external DB connection pool tuning and cloud DB hosting costs while delivering microsecond local insert latency and automated point-in-time disaster recovery.

### ADR-002: Apache Arrow and Parquet Columnar Shards for Model Training
* **Status:** Accepted
* **Context:** Deep learning training pipelines (PyTorch, Hugging Face `datasets`) require fast, zero-copy, memory-mapped batch loading over millions of synthesis traces.
* **Decision:** Compile verified SQLite traces into 3 canonical Apache Parquet shards (`router_sft.parquet`, `hole_filler_sft.parquet`, `hole_filler_dpo.parquet`) using the `arrow` and `parquet` Rust crates with Snappy compression.
* **Consequences:** Reduces disk storage by $80\text{--}85\%$ compared to uncompressed JSON Lines, supports memory-mapped dataset streaming across multiple GPUs, and conforms to industry-standard Hugging Face Hub dataset format.

### ADR-003: Multi-Stage PII and Secret Scrubber with Lexical Anonymization
* **Status:** Accepted
* **Context:** Telemetry from development environments risks leaking proprietary tokens, API keys, internal paths, and developer names.
* **Decision:** Enforce a multi-stage scrubbing gate at ingestion time. Scopes and expressions undergo token replacement, string literal neutralization, and regex scanning for known credential patterns (AWS, GitHub, Slack tokens). Traces with unscrubbed secrets are rejected immediately.
* **Consequences:** Prevents sensitive data contamination of public training corpora while preserving the exact grammatical AST structure necessary for compiler training.

### ADR-004: Gold Trace Mining for Direct Preference Optimization (DPO)
* **Status:** Accepted
* **Context:** Traditional LLM code fine-tuning relies only on positive demonstrations (SFT), missing critical signals about common compiler errors and invalid syntactic constructs.
* **Decision:** Instrument `Gold` tier traces—which represent repairs where an initial attempt failed but a subsequent attempt succeeded—to record the rejected attempt alongside compiler diagnostics, producing empirical `(prompt, chosen, rejected)` DPO pairs.
* **Consequences:** Enables direct preference alignment that penalizes code generation that fails compilation or typechecking, significantly reducing hallucinations.

### ADR-005: Embedded Static Asset Maintainer Web Dashboard
* **Status:** Accepted
* **Context:** Maintainers need visibility into telemetry intake rates, tier distributions, and pattern candidate review without deploying a separate Node.js frontend server.
* **Decision:** Embed HTML/JS/CSS assets directly into the `rabbet-cli` binary using `rust-embed` and serve them alongside the Axum REST endpoints under `/admin`, protected by bearer token authentication (`RABBET_ADMIN_TOKEN`).
* **Consequences:** Delivers a single zero-dependency self-contained binary providing both headless daemon ingestion and interactive maintainer inspection.
