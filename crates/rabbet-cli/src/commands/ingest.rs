//! `rabbet ingest`: feeds local spool envelopes through the same gate as the HTTP path.
//!
//! Until this existed, a Gold trace written to `.gaugeblock/spool/outbox.jsonl` was unreachable from
//! the command line: `serve` was the only way in, `admin stats` opened a `rabbet.db` that did not
//! exist, and no subcommand mentioned the spool (`GB-216`). So `mine-patterns` and `export-parquet`
//! could not be run against the traces the suite had just produced.
//!
//! This is deliberately a *transport*, not a second gate: the envelopes go through
//! `process_ingest_batch`, which is the same four stages the HTTP endpoint runs - hash verification,
//! PII re-scrub, structural validation, idempotent insert. A local shortcut that skipped them would
//! let a trace into the corpus that the public path would reject.

use crate::args::IngestArgs;
use rabbet_core::db::MasterStore;
use rabbet_core::gate::process_ingest_batch;
use rabbet_core::model::BatchIngestRequest;

/// The spool to read when `--spool` is not given: the suite's, resolved from `cwd`.
///
/// It used to be `GaugeblockPaths::discover(cwd).outbox()`, which resolves through the git common dir and so
/// never leaves the repository the command was run in. Inside a member that is the member's own spool - empty,
/// because traces centralise at the suite root (`GB-338`) - so `rabbet ingest` read nothing and reported
/// nothing wrong. `gaugeblock_workspace::outbox_path` asks the suite question, and the member's spool stays the
/// fallback for a repository that has no suite above it.
///
/// `pub` because the resolution is the part that was wrong, and a test that runs an ingest end to end cannot
/// tell "read the suite spool" from "read a spool that happened to have the same bytes".
#[must_use]
pub fn default_spool(cwd: &std::path::Path) -> std::path::PathBuf {
    gaugeblock_workspace::outbox_path(cwd)
        .unwrap_or_else(|| cwd.join(".gaugeblock").join("spool").join("outbox.jsonl"))
}

/// Executes `rabbet ingest`.
///
/// # Errors
///
/// Propagates a missing or unreadable spool, an unusable database, and any gate failure that is not
/// per-trace (per-trace rejections are counted and reported, not raised).
pub fn exec_ingest(args: IngestArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spool = match &args.spool {
        Some(explicit) => explicit.clone(),
        None => default_spool(&std::env::current_dir()?),
    };
    let text = std::fs::read_to_string(&spool)?;

    let mut entries = Vec::new();
    let mut malformed = 0usize;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str(line) {
            Ok(entry) => entries.push(entry),
            // A malformed line is skipped and counted: one bad record must not block a corpus import,
            // but it must not vanish silently either.
            Err(_) => malformed += 1,
        }
    }

    let total = entries.len();
    let store = MasterStore::open(&args.db_path)?;
    let response = process_ingest_batch(
        &store,
        &BatchIngestRequest {
            client_version: format!("rabbet {}", env!("CARGO_PKG_VERSION")),
            traces: entries,
        },
    )?;

    println!(
        "Spool {} ({} record(s), {} malformed):",
        spool.display(),
        total + malformed,
        malformed
    );
    println!(
        "  accepted {}  duplicates {}  rejected {}",
        response.accepted, response.duplicates, response.rejected
    );
    for detail in response.details.iter().filter(|d| d.status == "rejected") {
        println!(
            "  rejected {}: {}",
            &detail.content_hash[..detail.content_hash.len().min(12)],
            detail.reason.as_deref().unwrap_or("no reason given")
        );
    }
    println!("Database: {}", args.db_path.display());

    Ok(())
}
