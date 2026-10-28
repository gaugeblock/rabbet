//! CLI command handler for `rabbet mine-oss`.

use crate::args::MineOssArgs;
use rabbet_core::db::MasterStore;
use rabbet_core::miner::{ColdMinerOptions, mine_local_repo};

/// Executes the `rabbet mine-oss` command.
pub fn exec_mine_oss(args: MineOssArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = MasterStore::open(&args.db_path)?;
    let options = ColdMinerOptions {
        max_traces: args.max_traces,
        ..Default::default()
    };

    println!("Mining synthetic traces from: {}", args.path.display());
    let mined = mine_local_repo(&args.path, &store, &options)?;
    println!(
        "Successfully mined and ingested {} synthetic traces into master store",
        mined
    );

    let stats = store.get_stats()?;
    println!("Updated Corpus Statistics:");
    println!("  Total Traces: {}", stats.total_traces);
    println!("  Synthetic: {}", stats.synthetic_count);

    Ok(())
}
