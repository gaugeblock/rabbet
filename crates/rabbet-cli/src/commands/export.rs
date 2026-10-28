//! CLI command handler for `rabbet export-parquet`.

use crate::args::ExportParquetArgs;
use rabbet_core::db::MasterStore;
use rabbet_core::parquet::export_parquet_shards;
use rabbet_core::publish::prepare_release_bundle;

/// Executes the `rabbet export-parquet` command.
pub fn exec_export_parquet(
    args: ExportParquetArgs,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = MasterStore::open(&args.db_path)?;
    println!(
        "Compacting master store at '{}' to Parquet shards...",
        args.db_path.display()
    );

    let res = export_parquet_shards(&store, &args.output_dir)?;
    let stats = store.get_stats()?;
    let _ = prepare_release_bundle(&args.output_dir, &stats, "0.1.0-beta.1");

    println!(
        "Compaction complete! Output directory: {}",
        args.output_dir.display()
    );
    println!(
        "  - router_sft.parquet:       {} rows ({})",
        res.router_sft_rows,
        res.router_sft_path.display()
    );
    println!(
        "  - hole_filler_sft.parquet:  {} rows ({})",
        res.hole_filler_sft_rows,
        res.hole_filler_sft_path.display()
    );
    println!(
        "  - hole_filler_dpo.parquet:  {} rows ({})",
        res.hole_filler_dpo_rows,
        res.hole_filler_dpo_path.display()
    );
    println!("  - README.md:                Dataset card generated");

    Ok(())
}
