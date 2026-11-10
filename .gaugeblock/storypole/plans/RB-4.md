# Task Plan: GB-244 — `rabbet distill-telemetry`

> **Key:** `GB-244` · **Kind:** Feature · **Priority:** P2 · **Repo:** `rabbet`
> **Scope:** `rabbet-core`, `rabbet-cli`, `gaugeblock-contracts`
> **Depends on:** `GB-243` (the flight recorder that produces the spans) and `GB-182` (the canonical
> `SpoolEntry` envelope this command must write). Neither is optional: without `GB-243` there is no
> `events.jsonl`, and without `GB-182` the record this command emits is a bare trace the ingest gate
> cannot hash.
> **Status:** Planned — blocked on both.

---

## 1. What it reads, and what it writes

| Input | Where | Provenance |
|:--|:--|:--|
| `TelemetrySpan` lines | `.gaugeblock/telemetry/events.jsonl` | `GB-243` D1 (`O_APPEND`, one line per span) |
| Span queries (P50/P95, joins) | `.gaugeblock/telemetry/index.sqlite` | `GB-243` D4 (derived; rebuildable) |
| Rejected and accepted diffs | `.gaugeblock/telemetry/snapshots/<hash>.diff` | `GB-243` §6 (content-addressed) |
| `VerificationReceipt` | `.gaugeblock/storypole/index.sqlite`, `tasks.raw_json` | `storypole verify` |
| Output envelopes | `.gaugeblock/spool/outbox.jsonl` | `GB-182` `SpoolEntry` |
| Output datasets | `.gaugeblock/datasets/{gate_repair_dpo,agent_trajectory_dpo}.parquet` | this ticket |

Two measured facts about the inputs, because both change the implementation:

1. **`VerificationReceipt` has no `modified_files` field.** Its fields are exactly `verifier`,
   `command`, `exit_code`, `diagnostic_count`, `summary`, `verified_at_epoch_ms`, `git_branch`,
   `base_commit_sha`, `head_commit_sha`, `commit_shas`. So "the files the receipt touched" is
   *derived*: `git show --name-only <sha>` over `commit_shas`, falling back to
   `git diff --name-only base..head` when the list is empty, and skipped with a counted reason when
   both are absent. **The label is the commit, not a field.**
2. **Coverage is good but not total.** Of 200 verified tasks in the current index, 71 of the first 80
   carry `commit_shas` (~89%). The remaining ~11% can still be mined when `base_commit_sha` and
   `head_commit_sha` are present. Every skip is counted by reason and printed — a silent 10% loss is
   how a dataset acquires a bias nobody can see.

## 2. Decisions (resolved on the ticket)

| # | Decision | Chosen | Category |
|:--|:--|:--|:--|
| D1 | Repair-pair identification | `Error(A)` → next `Ok(B)` on the same `(task_key, step_name)`, `A != B`, emitted as a `Gold` `SpoolEntry` | `dataset` |
| D2 | Trajectory pruning | `McpTool`/`CliCommand` spans intersected with the receipt's commits define the minimal sufficient chain | `dataset` |
| D3 | Fail-fast ordering export | `fail_probability / p50_duration_ms` per repository and step, exported, not applied | `telemetry` |

**D1.** Pair any `SpanKind::GateStep` with `status == Error` (`worktree_diff_hash = A`,
`first_diagnostic`) against the next `SpanKind::GateStep` on the same `(task_key, step_name)` with
`status == Ok` (`worktree_diff_hash = B`, `A != B`). Emit a `Gold` DPO record — `context` = task plan
+ `first_diagnostic`, `rejected` = snapshot `A`, `chosen` = snapshot `B` — into
`.gaugeblock/spool/outbox.jsonl` wrapped in the canonical `SpoolEntry`, and export to
`gate_repair_dpo.parquet`.

Keying on the step name is the point: a `clippy` failure later repaired by a formatting-only change
shows up as `fmt Ok`, `clippy` still `Error`, and proximity-only pairing would learn `fmt` as the
fix. Requiring `A != B` rejects the common case of a step simply re-run, which is a rebuild, not a
repair. Records go through the outbox rather than straight to Parquet so the existing ingest gate
(placeholder and secret checks) judges them like any other trace.

**D2.** For every `Verified` task, inspect all `SpanKind::McpTool` and `SpanKind::CliCommand` spans
between `Claimed` and `Verified`. Calls whose queried files or symbols intersect the final receipt's
commits form the **Minimal Sufficient Trajectory** (`chosen`); tasks with high-thrash subsequences
(3+ redundant searches or failed invocations before finding the target) supply the raw trajectory
(`rejected`), training `ModelRole::System1Router` and `Planner` to pick the right `miter`/`storypole`
query on the first turn. The receipt is the label source because it is already a durable, verifiable
fact; a judge model would add inference cost and a second thing that can be wrong. Spans touching
nothing in the receipt are kept — they are what makes the rejected half a real trajectory rather
than a strawman.

**D3.** Compute per-repository, per-step `fail_probability` and P50 duration and export
`fail_probability / p50_duration_ms`, so `outfeed gate --fail-fast-order` can optionally run
high-failure-density, sub-second checks (`docs`, `lockfiles`, `fmt`) before multi-second compiler
passes. This ticket exports the ranking and does not change the default matrix: the canonical order
in `gate_steps()` is part of what Tier 3 means, and flipping it silently would make two gates
disagree about the same word. Parallel execution was rejected — the gate is deliberately sequential
so a lockfile failure is reported before a compile.

---

## 3. Module layout and budget (every file ≤ 250 LOC)

"Today" is `wc -l` on this checkout.

| # | File | Action | Budget | Today | Note |
|:--|:--|:--|:--|:--|:--|
| 1 | `rabbet/crates/rabbet-core/src/distill_repairs.rs` | new | ≤ 210 | — | scan `Error` → `Ok` transitions on `(task_key, GateStep, name)`, load snapshots, build `Gold` `SpoolEntry` records |
| 2 | `rabbet/crates/rabbet-core/src/distill_trajectories.rs` | new | ≤ 220 | — | join `McpTool`/`CliCommand` spans for `Verified` tasks against receipt commits; `Silver` SFT chains and `Gold` pruned-vs-unpruned pairs |
| 3 | `rabbet/crates/rabbet-core/src/distill/mod.rs` | new | ≤ 80 | — | shared types: `DistillOptions`, `DistillReport`, `SkipReason`, the `TelemetryReader` trait |
| 4 | `rabbet/crates/rabbet-core/src/distill/telemetry_reader.rs` | new | ≤ 170 | — | reads `events.jsonl` (streaming) and/or `index.sqlite`; the JSONL path is authoritative so this works before D4's index exists |
| 5 | `rabbet/crates/rabbet-core/src/parquet/schemas.rs` | update | ≤ 110 | 42 | add `gate_repair_dpo_schema()` and `agent_trajectory_dpo_schema()` |
| 6 | `rabbet/crates/rabbet-core/src/parquet/datasets.rs` | new | ≤ 150 | — | builds the `RecordBatch`es and calls the existing `write_parquet_file` (37 LOC, reused unchanged) |
| 6b | `rabbet/crates/rabbet-core/src/parquet/exporter.rs` | **unchanged** | 250 | 204 | **46 lines of headroom — the new datasets do not belong in this file**; growing it is the trap |
| 7 | `rabbet/crates/rabbet-cli/src/commands/distill_telemetry.rs` | new | ≤ 150 | — | `rabbet distill-telemetry [--since <tag>] [--export-parquet] [--json]` |
| 8 | `rabbet/crates/rabbet-cli/src/args.rs` | update | ≤ 220 | 170 | one `Command::DistillTelemetry` variant + a `DistillTelemetryArgs` struct; if it passes 220 the args split into `args/` like `storypole-cli` did in `GB-182` |

**No new dependencies.** `rabbet-core` already declares `arrow 60`, `parquet 60` (`arrow` + `snap`),
`rusqlite 0.40` (`bundled`), `sha2 0.10` and `serde_json` — every piece the miner needs. A command
that adds a dependency to read a text file would be a smell.

**Scrubbing goes through `gaugeblock-security`, not `miter`.** The brief says "scrubs literals via
`miter`"; `rabbet-core` has no `miter` dependency and must not gain one. `miter` and `rabbet` are
different products, so the edge would be lateral, and `miter deps --layers --check` — a Tier 3 gate
step — rejects new lateral edges outside the allowlist (the file currently lists **zero** edges for
`rabbet-core`). The existing path is already correct and already imported:
`gaugeblock_security::scrub_source_text`, used by `rabbet-core/src/gate/scrubber.rs`.

**Reading `storypole`'s index is raw SQL, not a crate dependency.** For the same layering reason,
`rabbet-core` must not depend on `storypole-core`. The receipt is reachable as
`select raw_json from tasks where status = 'verified'` and deserialising the embedded
`verification_receipt`. That keeps `rabbet` behind the *file* contract rather than a Rust type it
would have to version against.

## 4. Dataset shapes

`gate_repair_dpo.parquet` (extends the existing DPO convention in `parquet/schemas.rs`):

| Column | Type | Source |
|:--|:--|:--|
| `task_key` | Utf8, not null | span |
| `step_name` | Utf8, not null | `GateStep.name` (`fmt`/`clippy`/`tests`/`contracts`/`layers`/`lockfiles`/`docs`) |
| `context` | Utf8, not null | task plan excerpt + `first_diagnostic` |
| `rejected` | Utf8, not null | snapshot `A` (scrubbed) |
| `chosen` | Utf8, not null | snapshot `B` (scrubbed) |
| `rejected_hash` / `chosen_hash` | Utf8, not null | the two `worktree_diff_hash` values |
| `quality_tier` | Utf8, not null | literal `gold` |

`agent_trajectory_dpo.parquet`:

| Column | Type | Source |
|:--|:--|:--|
| `task_key` | Utf8, not null | receipt |
| `role` | Utf8, not null | `system1_router` or `planner` |
| `chosen_calls` | Utf8, not null | minimal sufficient call sequence (compact JSON array) |
| `rejected_calls` | Utf8, null | thrashing sequence, when one exists |
| `touched_files` | Utf8, not null | derived from `commit_shas` |
| `quality_tier` | Utf8, not null | `silver` for a chain, `gold` for a pair |

Both shards are written with the existing Snappy `write_parquet_file`, so compression, directory
creation and row counting are not re-implemented.


---

## 5. Command surface

```text
rabbet distill-telemetry [--since <tag>] [--export-parquet] [--json] [--dry-run]
                         [--telemetry <path>] [--storypole <path>] [--datasets <dir>]
```

| Flag | Meaning |
|:--|:--|
| `--since <tag>` | Only spans whose `started_at_ms` is at or after the commit date of `<tag>`; the default is all recorded history |
| `--export-parquet` | Also writes the two shards into `.gaugeblock/datasets/`; without it only the outbox records are appended |
| `--dry-run` | Counts what would be emitted and writes nothing — the flag that makes the yield figures in §7 reproducible |
| `--json` | Machine-readable report: `{ "repair_pairs": n, "trajectories": n, "skipped": {...by reason...}, "parquet_rows": {...} }` |

Defaults resolve through `gaugeblock-workspace`'s canonical paths, not `current_dir` — the same rule
`GB-189` establishes for every other `.gaugeblock/` consumer. Records appended to the outbox are
idempotent: the `SpoolEntry` content hash of an unchanged pair is identical, so a second run adds
nothing and reports `0 new, N duplicate` rather than appending a second copy of the same pair.

## 6. Ordered plan steps

| # | Step | Files | Verification gate |
|:--|:--|:--|:--|
| **D1** | Implement `distill_repairs.rs` and `distill/mod.rs`: pair failed vs. passing `outfeed gate` snapshots into `Gold` DPO `SpoolEntry` records, with scrubbing through `gaugeblock-security`. Fixtures cover: a real repair (`A != B`), a rebuilt-unchanged step (`A == B`, skipped), a missing snapshot (skipped and counted), and an `Ok` with no preceding `Error` (skipped). | `rabbet-core/src/distill_repairs.rs`, `distill/mod.rs`, `distill/telemetry_reader.rs` | `cargo nextest run -p rabbet-core distill_repairs` |
| **D2** | Implement `distill_trajectories.rs`: correlate `McpTool`/`CliCommand` span sequences with receipt commits (via `git show --name-only`), producing `Silver` SFT minimal chains and `Gold` pruned-vs-unpruned pairs. Fixtures cover a task whose receipt has `commit_shas`, one with only `base`/`head`, and one with neither (skipped and counted). | `rabbet-core/src/distill_trajectories.rs` | `cargo nextest run -p rabbet-core distill_trajectories` |
| **D3** | Wire `rabbet distill-telemetry` (args, dispatch, `--json` report), add the two Arrow schemas and `parquet/datasets.rs`, and verify end-to-end under law 12 on real workspace telemetry spans. | `rabbet-cli/src/{args.rs,commands/distill_telemetry.rs,commands/mod.rs,main.rs}`, `rabbet-core/src/parquet/{schemas.rs,datasets.rs}` | `cargo nextest run -p rabbet-core -p rabbet-cli`, then `rabbet distill-telemetry --dry-run --json` on the live `.gaugeblock/telemetry/`, then `rabbet ingest` over the resulting outbox asserting `0 rejected` |

Tier 2 for every step: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`,
`chalkline docs check -d rabbet`.

## 7. Measurements to record (law 12)

| What | Where it is recorded |
|:--|:--|
| Repair-pair yield: `Error→Ok` transitions found, pairs emitted, and skip reasons with counts | commit message that lands D1, plus `rabbet/README.md` |
| Trajectory yield: verified tasks scanned, tasks with commits, tasks with a thrashing subsequence, chains vs. pairs | commit message that lands D2 |
| Records rejected by the ingest gate and why | D3 verification output, quoted in the commit |
| Row counts per shard and `.parquet` bytes | D3, in the same commit |
| Fail-fast ranking for the suite root: per-step `fail_probability`, P50 ms, sample count | a durable table in `rabbet/README.md` — a ranking with no sample count is an opinion |

Reported plainly if the yield is bad. A corpus of zero repair pairs is a result: it means either the
gate rarely fails or `GB-243`'s snapshots are not being written, and the two are distinguishable by
the skip reasons.

## 8. Out of scope and risks

Out of scope:

* **Changing the default gate order.** D3 exports a ranking; `outfeed gate --fail-fast-order` is a
  later ticket that consumes it.
* **Training.** This produces datasets. Nothing here runs a model.
* **`Diamond`-tier mining** (survival-in-main), which needs release history, not gate history.
* **Cross-repository repair pairs.** A fix in `miter` that repairs a `benchwork` gate is a real
  signal but it has no shared `(task_key, step_name)` key yet.
* **Deduplicating against the existing three shards.** Those carry hole-filling traces; these carry
  behaviour, and the join key would be `task_key`, which the older shards do not have.

| Risk | Answer |
|:--|:--|
| Snapshots are missing because a gate ran before `GB-243` landed | Every pair is conditional on both snapshots existing; the report counts them, so the dataset is honest from day one rather than padded. |
| A diff snapshot contains a credential | Snapshots are scrubbed at write time by `GB-243` and re-scrubbed at read time here through `gaugeblock_security::scrub_source_text`; a high-entropy hit rejects the record, it does not redact-and-keep. |
| `git show` is slow over hundreds of receipts | Commits are resolved once per run into a `HashMap<task_key, Vec<PathBuf>>` and cached for the process; one `git show --name-only` per sha, batched with `--no-walk`. |
| Re-running the command duplicates records | Idempotent by `SpoolEntry` content hash (§5), proven by a test that runs the miner twice and asserts the second run appends nothing. |
| Trajectory labels are wrong because a task touched a file incidentally | The receipt's commits define "touched"; an incidentally-touched file is still touched. Where a task has no commits, it is skipped rather than guessed. |

---

## 9. Architectural alignment (2026-11-07)

Authoritative where it disagrees with earlier sections. The design intent was stated explicitly after
this plan was written; the two-stream boundary below is the part that had been implicit.

### 9.1 This command is the only bridge between two different files

`.gaugeblock/telemetry/events.jsonl` and `.gaugeblock/spool/outbox.jsonl` must **never** be merged, and
this command is the sole path from one to the other:

- the **flight recorder** is high-frequency raw operational observation — every CLI run, every
  `*-mcp` tool call, every gate step, every `sawhorse` request, appended in under 100 microseconds;
- the **synthesis spool** is curated, scrubbed, training-ready `SpoolEntry` records, each carrying a
  provenance claim;
- `rabbet distill-telemetry` reads the first, decides what is worth learning from, and **promotes** it
  into the second. Nothing else writes to the spool from the telemetry side, and nothing reads the
  spool to answer an operational question.

That boundary shapes three decisions already in this plan, and they are now load-bearing rather than
taste:

1. Being a *separate* command rather than a hook in the write path is what keeps the hot path cheap.
2. Its output must be `SpoolEntry`, not `TelemetrySpan` — the envelope (`GB-182`) is what makes a
   promoted record ingestible and hash-stable.
3. Provenance must name the producer. The spool now receives records from three producers, so a
   reader has to be able to tell which one earned a record:
   | Producer | Record |
   |:--|:--|
   | `storypole verify` | `Silver` — deterministic function changes from the `base..head` range |
   | `joiner fill` | `Silver` first-pass syntheses, `Gold` DPO syntax-repair pairs |
   | `rabbet distill-telemetry` (this ticket) | `Gold` gate-repair DPO, `Gold` MCP tool-trajectory DPO |

### 9.2 The two pairings, stated as the thing being learned

- **Gate-repair pairs.** A failed gate run and the passing run that repairs it, with the diff snapshots
  from `.gaugeblock/telemetry/snapshots/*.diff`. The lesson is "this diagnostic was fixed by this
  change", which is why the failure must be paired with the *repairing* run and not any later pass.
- **MCP trajectory pairs.** A minimal trajectory and a thrashing one that reached the same place. The
  lesson is routing, not content, so the record must preserve the tool sequence and its costs rather
  than the arguments.

The detectors for "thrashing" live in this command's read path, not in the flight recorder: the
recorder's job is to have recorded both trajectories faithfully, and it must not judge at append time.
The runtime thrash *alert* (`GB-308`) is a different consumer of the same spans and can disagree with
the offline distillation threshold without either being wrong — one is a live hint, the other is a
training filter.

### 9.3 One spool, two exports

Promotion writes `.gaugeblock/spool/outbox.jsonl`; the parquet exports
(`gate_repair_dpo.parquet`, `agent_trajectory_dpo.parquet`) are derived views of the same records, not a
second source of truth. If a record is pruned from the spool the export must be regenerated rather than
kept, or the dataset will contain a lesson the spool says was retracted.

### 9.4 Panel 4 depends on this being one idempotent command

The benchwork spool table exposes a `[ ⚡ Run Distill Telemetry ]` action (`GB-310`). That makes this
command's idempotency a UI requirement rather than a nicety: running it twice must not duplicate
records, and its exit must leave the spool counts the panel displays in a consistent state.
