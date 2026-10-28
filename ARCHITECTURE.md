# `rabbet` — System Architecture

> **Domain:** Telemetry Corpus Hub & Training Foundry | **Suite:** `gaugeblock`

This document outlines the architectural components, subsystem interactions, and data storage workflows of `rabbet`.

---

## 1. System Decomposition

```mermaid
graph TD
    subgraph Clients["Edge Clients"]
        J[joiner-core]
        B[benchwork-core]
    end

    subgraph RabbetServer["rabbet-cli Ingestion Daemon"]
        HTTP[Axum HTTP Router]
        Gate[4-Stage Gate: Scrubber + Tree-sitter]
        AdminUI[Embedded Admin Web Console]
    end

    subgraph Storage["Persistence & Replication"]
        DB[(SQLite Master Store: rabbet.db)]
        LS[Litestream Replication Engine]
        Cloud[(S3 / Cloudflare R2 Bucket)]
    end

    subgraph Foundry["Foundry Engine (rabbet-core)"]
        Compactor[Parquet Compactor Engine]
        OSS[Cold-Start OSS Miner]
        Miner[AST Pattern Miner]
    end

    subgraph Downstream["Training & Suite Artifacts"]
        HF[(Hugging Face Hub)]
        Dado[.gaugeblock/dado/patterns]
    end

    J -->|POST /api/v1/traces| HTTP
    B -->|POST /api/v1/traces| HTTP
    HTTP --> Gate
    Gate --> DB
    AdminUI --> DB
    
    DB -.->|Continuous Streaming| LS
    LS -.->|Snapshot / WAL Sync| Cloud
    
    DB --> Compactor
    Compactor -->|router_sft.parquet\nhole_filler_sft.parquet\nhole_filler_dpo.parquet| HF
    
    OSS -->|Synthetic Traces| DB
    DB --> Miner
    Miner -->|Approved Patterns| Dado
```

---

## 2. Ingestion & Compaction Sequence

```mermaid
sequenceDiagram
    autonumber
    actor Client as Edge Client (joiner)
    participant Server as rabbet Axum Server
    participant Gate as 4-Stage Ingestion Gate
    participant DB as SQLite Master Store
    participant Compactor as Parquet Compactor
    actor Hub as Hugging Face Hub

    Client->>Server: POST /api/v1/traces (BatchIngestRequest)
    Server->>Gate: Evaluate Trace Batch
    Gate->>Gate: Verify SHA-256 Content Hash
    Gate->>Gate: Scrub PII / Tokens via AST Lexicon
    Gate->>Gate: Verify Tree-sitter AST Syntax
    Gate->>DB: Check Content Hash (Dedup)
    alt Unique & Valid
        Gate->>DB: INSERT into traces (WAL Mode)
        Gate-->>Server: Accepted
    else Duplicate Hash
        Gate-->>Server: Duplicate (No-op)
    else Invalid Hash / Syntax / PII
        Gate-->>Server: Rejected with Diagnostic
    end
    Server-->>Client: HTTP 200 (BatchIngestResponse)

    Note over DB,Compactor: Periodic Compaction or CLI Export
    Compactor->>DB: Query Verified Traces (Bronze..Diamond)
    Compactor->>Compactor: Build Arrow RecordBatches
    Compactor->>Compactor: Write Compressed Parquet Shards
    Compactor->>Hub: Publish Dataset Bundle (rabbet publish-hub)
```

---

## 3. Storage Architecture

* **Primary Single Source of Truth:** Embedded SQLite database (`rabbet.db`) running in Write-Ahead Logging (`WAL`) mode with `NORMAL` synchronous pragma. Indexed on `content_hash (UNIQUE)`, `tier`, `language`, and `created_at`.
* **Zero-Downtime Backup & Replication:** Uses Litestream to stream WAL segments directly to object storage (AWS S3, Cloudflare R2, MinIO), guaranteeing Point-In-Time Recovery without heavy relational database administration.
* **Columnar Sharding for Deep Learning:** Parquet files are generated using Apache Arrow `RecordBatch` streams with `SNAPPY` compression. This achieves $\sim 8\times$ compression over raw JSON and enables memory-mapped multi-GPU training loaders in PyTorch.

---

## 4. Crate & Package Topology

| Crate | Path | Responsibility |
|:---|:---|:---|
| [`rabbet`](./src/lib.rs) | `.` | Root facade re-exporting `rabbet_core` domain APIs |
| [`rabbet-cli`](./crates/rabbet-cli) | `crates/rabbet-cli` | Axum HTTP server, static asset embedding (`rust-embed`), Clap CLI |
| [`rabbet-core`](./crates/rabbet-core) | `crates/rabbet-core` | Schemas, SQLite repository, Tree-sitter AST validator, Arrow compactor, OSS & pattern miners |
