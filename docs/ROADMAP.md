# `rabbet` — Roadmap & Strategic Horizons

> **Current Lifecycle:** `0.1.0-beta.2` | **Suite:** `gaugeblock`

This document tracks the milestones, active features, and future horizons for `rabbet`.

---

## 1. Active Milestone: `v0.1.0-beta.2` (Current)

- [x] **4-Stage Ingestion Gate:** Cryptographic SHA-256 validation, PII/secret scrubbing, Tree-sitter AST validation, and SQLite idempotent insertion.
- [x] **Master Store (SQLite + Litestream):** Thread-safe WAL-mode persistence with streaming S3/R2 backup support.
- [x] **Arrow & Parquet Dataset Compactor:** Three canonical training shards (`router_sft`, `hole_filler_sft`, `hole_filler_dpo`).
- [x] **Cold-Start OSS Miner:** Extraction of synthetic `(Scope + TypedHole -> Expression)` pairs from open-source codebases.
- [x] **Gold Trace Pattern Miner:** Structural AST skeleton clustering to propose new candidate `.dado` templates.
- [x] **Admin Web Console:** Embedded `rust-embed` dashboard for corpus metrics and pattern approval.
- [x] **9-Document Architecture:** Canonical specification and zero-drift documentation compliance.

---

## 2. Near-Term Horizon: `v0.2.0-beta.1`

- [ ] **Streaming Parquet Compaction:** Incremental chunked export without buffering full batches in memory.
- [ ] **Automated Hugging Face CI Release:** Automated nightly builds pushing updated shards to Hugging Face Hub.
- [ ] **Multi-Tenant Org Isolation:** Optional organization-level API keys and segregated SQLite databases.
- [ ] **Diamond Tier Ingestion:** Automated webhook listener tracking Git commits to upgrade surviving Gold traces to Diamond.

---

## 3. Long-Term Horizon: `v0.3.0+`

- [ ] **In-Process Micro-Model Training:** Direct integration with Candle or PyTorch for local parameter-efficient fine-tuning (LoRA).
- [ ] **Distributed Consensus Replication:** Raft-based multi-region active-active cluster replication for enterprise deployments.
- [ ] **Cross-Language AST Transfer:** Shared intermediate AST representations enabling synthesis training pairs across Rust, TypeScript, and Python.
