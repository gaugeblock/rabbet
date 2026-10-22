//! CLI command handler for `rabbet mine-patterns`.

use crate::args::MinePatternsArgs;
use rabbet_core::db::MasterStore;
use rabbet_core::miner::{discover_pattern_candidates, export_dado_pattern_file, PatternMinerOptions};

/// Executes the `rabbet mine-patterns` command.
pub fn exec_mine_patterns(args: MinePatternsArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = MasterStore::open(&args.db_path)?;
    let options = PatternMinerOptions {
        min_frequency: args.min_frequency,
    };

    println!("Scanning Gold Oracle traces for recurring AST skeletons (min-frequency: {})...", args.min_frequency);
    let candidates = discover_pattern_candidates(&store, &options)?;
    println!("Discovered {} pattern candidate(s):", candidates.len());

    for cand in &candidates {
        println!("  - [{}] {} ({} occurrences)", cand.candidate_id, cand.description, cand.occurrence_count);
        println!("    Template: {}", cand.template.lines().next().unwrap_or(""));
        println!("    Typed Holes: {:?}", cand.typed_holes);

        if let Some(out_dir) = &args.output_dir {
            let exported_path = export_dado_pattern_file(cand, out_dir)?;
            println!("    Exported to: {}", exported_path.display());
        }
    }

    Ok(())
}

