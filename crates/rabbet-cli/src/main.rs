//! Binary entrypoint for `rabbet`.

use clap::Parser;
use rabbet_cli::args::{Cli, Command};
use rabbet_cli::commands::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // One span per invocation, so the panel can answer what the suite did rather than only what inference
    // cost (`GB-339`). Opened before any work and closed with the outcome; it cannot fail the command.
    let invocation = gaugeblock_workspace::InvocationSpan::start("rabbet", None);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    // Inside an async block so the result can be inspected before the span closes: with `?` at this level an error
    // returned from `main` and the span was recorded as still running (`GB-339`).
    let result = async {
        match cli.command {
            Command::Serve(args) => exec_serve(args).await?,
            Command::MineOss(args) => exec_mine_oss(args)?,
            Command::ExportParquet(args) => exec_export_parquet(args)?,
            Command::Ingest(args) => exec_ingest(args)?,
            Command::MinePatterns(args) => exec_mine_patterns(args)?,
            Command::PublishHub(args) => exec_publish_hub(args).await?,
            Command::Admin(args) => exec_admin(args)?,
        }
        Ok(())
    }
    .await;

    invocation.finish(result.is_ok());
    result
}
