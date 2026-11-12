//! multi-repo: a declared suite with two members, and the spool question asked from inside one of them.
//!
//! `rabbet ingest` with no `--spool` used to resolve through `GaugeblockPaths::discover`, which answers with the
//! checkout it is in and therefore never leaves a member. A member's spool is empty - traces are written to the
//! suite root (`GB-338`) - so the command read nothing, ingested nothing, and exited successfully. An empty
//! corpus and a corpus that was never looked for produce the same silence.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

use rabbet_cli::commands::ingest::default_spool;

/// Both sides canonicalised: the resolver canonicalises before walking ancestors (`GB-327`), and the file
/// itself does not exist, where `canonicalize` fails silently - so the parent is canonicalised instead.
fn canon(path: PathBuf) -> PathBuf {
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => match std::fs::canonicalize(parent) {
            Ok(dir) => dir.join(name),
            Err(_) => path,
        },
        _ => path,
    }
}

/// A member repository, qualified by a `.git` marker and holding its own spool directory.
fn member(root: &Path, name: &str) -> PathBuf {
    let dir = root.join(name);
    fs::create_dir_all(dir.join(".git")).expect("member checkout");
    fs::create_dir_all(dir.join(".gaugeblock").join("spool")).expect("member spool");
    dir
}

/// A suite root that declares `members` and holds a spool of its own.
fn suite(members: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join(".gaugeblock").join("spool")).expect("suite spool");
    let declared: Vec<String> = members.iter().map(|name| format!("\"{name}\"")).collect();
    fs::write(
        root.path().join(".gaugeblock").join("config.toml"),
        format!("[suite]\nmembers = [{}]\n", declared.join(", ")),
    )
    .expect("suite declaration");
    for name in members {
        member(root.path(), name);
    }
    root
}

#[test]
fn ingest_from_a_member_reads_the_suite_spool() {
    let root = suite(&["alpha", "beta"]);
    let alpha = root.path().join("alpha");

    assert_eq!(
        canon(default_spool(&alpha)),
        canon(
            root.path()
                .join(".gaugeblock")
                .join("spool")
                .join("outbox.jsonl")
        ),
        "traces are written at the suite root, so that is the spool a member must read"
    );
}

#[test]
fn the_member_spool_is_not_what_a_member_reads() {
    // The assertion above would pass if both paths agreed, so this pins that they do not: a fix that quietly
    // resolves to the member would satisfy "some spool was found" and fail this.
    let root = suite(&["alpha", "beta"]);
    let alpha = root.path().join("alpha");
    assert_ne!(
        canon(default_spool(&alpha)),
        canon(alpha.join(".gaugeblock").join("spool").join("outbox.jsonl"))
    );
}

#[test]
fn a_repository_with_no_suite_keeps_its_own_spool() {
    // Nothing to centralise into. Falling back is the documented behaviour and it must stay reachable.
    let lone = tempfile::tempdir().expect("tempdir");
    let repo = member(lone.path(), "solo");
    assert_eq!(
        canon(default_spool(&repo)),
        canon(repo.join(".gaugeblock").join("spool").join("outbox.jsonl"))
    );
}
