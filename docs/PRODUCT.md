# `rabbet` — Product Vision & Strategy

> **Domain:** Telemetry Corpus Hub & Training Foundry | **Suite:** `gaugeblock`

`rabbet` is the central telemetry aggregation hub and training foundry for the `gaugeblock` neurosymbolic suite, transforming edge synthesis traces into compiler-verified open-weight models.

---

## 1. Personas & Problems

| Persona | Pain Points | How `rabbet` Solves It |
|:---|:---|:---|
| **Model Trainer / AI Researcher** | Scraped code datasets are full of syntax bugs, hallucinations, and unverified types; manual data cleaning is tedious. | Provides 100% compiler-verified SFT and empirical DPO datasets formatted as ready-to-train Parquet shards. |
| **Tool Maintainer / Engineer** | Lack of visibility into real-world agent synthesis failures, repair loops, and common hole patterns. | Offers real-time corpus metrics, tier distribution graphs, and automated pattern clustering from Gold traces. |
| **Security & Privacy Officer** | Risk of developer API keys, passwords, or proprietary code leaking into training corpora. | Enforces a zero-trust 4-stage ingestion gate with AST-level anonymization and cryptographic content validation. |
| **Autonomous Systems Agent** | Edge tools (`joiner`, `benchwork`) need reliable, non-blocking telemetry export without slowing down development. | Provides offline outbox spooling with batched, idempotent synchronization over lightweight REST endpoints. |

---

## 2. Jobs to Be Done (JTBD)

1. **Empirical Telemetry Aggregation:** When synthesis tasks execute on edge developer machines, aggregate anonymized AST scopes, hole fills, and compiler verification verdicts without blocking development or risking privacy leakage.
2. **Preference Dataset Compilation:** When fine-tuning micro-models for code synthesis, compile negative repair attempts and positive verified completions into empirical DPO pairs to penalize compilation errors.
3. **Pattern Mining & Grammar Discovery:** When recurring code structures emerge across diverse synthesis traces, cluster structural skeletons to propose candidate `dado` templates and expand System 1 synthesis coverage.
4. **Open-Weight Ecosystem Publishing:** When new telemetry cohorts reach release thresholds, package versioned Parquet bundles and push them to Hugging Face Hub for open research and fine-tuning.

---

## 3. Product Success Metrics

* **PII & Secret Neutralization:** 100% rejection rate for traces containing unscrubbed secrets or raw private tokens.
* **Compilation Validity:** 100% of positive training targets (`chosen`) in exported Parquet shards pass compiler typechecking.
* **Compaction Throughput:** $\ge 50{,}000$ traces processed per second during Parquet RecordBatch export.
* **Self-Contained Deployment:** Single zero-dependency binary runtime providing both ingestion daemon and embedded maintainer UI.
