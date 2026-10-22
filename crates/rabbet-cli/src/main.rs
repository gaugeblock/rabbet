//! Binary entrypoint for `rabbet`.

use clap::Parser;
use rabbet_cli::args::{Cli, Command};
use rabbet_cli::commands::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve(args) => exec_serve(args).await?,
        Command::MineOss(args) => exec_mine_oss(args)?,
        Command::ExportParquet(args) => exec_export_parquet(args)?,
        Command::MinePatterns(args) => exec_mine_patterns(args)?,
        Command::PublishHub(args) => exec_publish_hub(args).await?,
        Command::Admin(args) => exec_admin(args)?,
    }

    Ok(())
}
