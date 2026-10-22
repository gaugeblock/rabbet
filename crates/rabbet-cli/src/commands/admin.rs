//! CLI command handler for `rabbet admin`.

use crate::args::{AdminArgs, AdminSubcommand};
use rabbet_core::db::MasterStore;
use rabbet_core::miner::export_dado_pattern_file;
use rabbet_core::parquet::export_parquet_shards;

/// Executes the `rabbet admin` subcommands.
pub fn exec_admin(args: AdminArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = MasterStore::open(&args.db_path)?;

    match args.action {
        AdminSubcommand::Stats => {
            let stats = store.get_stats()?;
            println!("Corpus Statistics (Database: {}):", args.db_path.display());
            println!("  Total Unique Traces: {}", stats.total_traces);
            println!("  Tiers:");
            for (tier, count) in &stats.counts_by_tier {
                println!("    - {}: {}", tier, count);
            }
            println!("  Languages:");
            for (lang, count) in &stats.counts_by_language {
                println!("    - {}: {}", lang, count);
            }
            println!("  Synthetic (OSS Mined): {}", stats.synthetic_count);
            println!("  Community Contributed: {}", stats.community_count);
            println!("  Gold DPO Pairs:        {}", stats.dpo_pairs_count);
            println!("  Database File Size:    {:.2} MB", stats.db_size_bytes as f64 / (1024.0 * 1024.0));
        }
        AdminSubcommand::Compact { output_dir } => {
            println!("Triggering Parquet compaction from {}...", args.db_path.display());
            let res = export_parquet_shards(&store, &output_dir)?;
            println!("Compacted Parquet Shards:");
            println!("  router_sft.parquet:      {} rows", res.router_sft_rows);
            println!("  hole_filler_sft.parquet: {} rows", res.hole_filler_sft_rows);
            println!("  hole_filler_dpo.parquet: {} rows", res.hole_filler_dpo_rows);
        }
        AdminSubcommand::Candidates => {
            let candidates = store.list_pattern_candidates(None)?;
            println!("Discovered Pattern Candidates ({} total):", candidates.len());
            for c in candidates {
                println!("  - [{}] {} (count: {})", c.candidate_id, c.description, c.occurrence_count);
            }
        }
        AdminSubcommand::Approve { candidate_id, output_dir } => {
            let candidates = store.list_pattern_candidates(None)?;
            if let Some(cand) = candidates.iter().find(|c| c.candidate_id == candidate_id) {
                let out = export_dado_pattern_file(cand, &output_dir)?;
                store.update_candidate_status(&candidate_id, "approved")?;
                println!("Pattern candidate '{}' approved and written to {}", candidate_id, out.display());
            } else {
                eprintln!("Error: Candidate '{}' not found", candidate_id);
            }
        }
    }

    Ok(())
}

