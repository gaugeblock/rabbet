# Contributing to `rabbet`

Thank you for your interest in contributing to `rabbet`! We welcome contributions to our telemetry aggregation engine, SQLite master store, Parquet compactor, and pattern mining algorithms.

---

## 1. Prerequisites

* **Rust Toolchain:** Rust 2024 edition (stable 1.85+ / nightly for bleeding-edge checks).
* **CLI Suite:** Globally installed `storypole`, `chalkline`, and `miter` from `PATH`.
* **Testing Runners:** `cargo-nextest` (recommended) or native `cargo test`.
* **SQLite:** Local SQLite development headers (or bundled via `rusqlite`).

---

## 2. Development Workflow

1. **Pick or Claim a Task:**
   Check available tasks via `storypole list` and claim your ticket:
   ```bash
   storypole claim <KEY> <YOUR_ID> --branch feat/<slug>
   ```
2. **Dedicated Branch:**
   Always develop non-trivial changes on a dedicated feature branch (`feat/<slug>` or `fix/<slug>`).
3. **Inner-Loop Iteration (Tier 1):**
   ```bash
   cargo check -p rabbet-core
   cargo nextest run -p rabbet-core
   ```
4. **Pre-Commit Verification Gate (Tier 2):**
   Before committing, verify code formatting, clippy lints, tests, and documentation drift:
   ```bash
   cargo fmt --check -p rabbet-core -p rabbet-cli
   cargo clippy -p rabbet-core -p rabbet-cli --all-targets -- -D warnings
   cargo test -p rabbet-core -p rabbet-cli
   chalkline docs check -d .
   ```
5. **Commit with Conventional Commits:**
   ```bash
   git commit -m "feat(rabbet-core): implement Parquet dictionary encoding for scopes"
   ```

---

## 3. Engineering Invariants & Coding Standards

* **Hard 250-Line File Limit:** No `.rs` source file may exceed 250 lines. Modularize submodules cleanly.
* **Strict Error Handling:** Library crates (`rabbet-core`) must use strongly typed `RabbetError` via `thiserror`. Never use `.unwrap()` or `.expect()` in library paths.
* **9-Document Compliance:** Keep all 9 canonical documents synchronized without drift. Run `chalkline docs check -d .` to certify.
