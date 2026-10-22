//! Clap CLI arguments, subcommands, and flags for `rabbet`.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Rabbet: Public Telemetry Corpus Hub & Open-Weight Training Foundry.
#[derive(Debug, Parser)]
#[command(
    name = "rabbet",
    author = "Colin Wolff <colin@gaugeblock.dev>",
    version = "0.1.0-beta.1",
    about = "Public Telemetry Corpus Hub & Open-Weight Training Foundry",
    long_about = "Aggregates AST-anonymized synthesis telemetry, compiles Arrow/Parquet SFT/DPO datasets, discovers new dado templates, and publishes open-weight models."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Starts the public telemetry ingestion server and Maintainer Admin Console.
    Serve(ServeArgs),

    /// Mines synthetic (Scope + TypedHole -> Expression) training pairs from OSS code.
    MineOss(MineOssArgs),

    /// Compiles and exports the 3 canonical Parquet dataset shards from master SQLite.
    ExportParquet(ExportParquetArgs),

    /// Discovers and clusters recurring patterns from Gold Oracle synthesis traces.
    MinePatterns(MinePatternsArgs),

    /// Packages and publishes Parquet dataset shards to Hugging Face Hub.
    PublishHub(PublishHubArgs),

    /// Executes maintainer control plane operations via CLI.
    Admin(AdminArgs),
}

/// Arguments for `rabbet serve`.
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Port to bind the HTTP server.
    #[arg(short, long, default_value = "8080", env = "RABBET_PORT")]
    pub port: u16,

    /// Host address to bind.
    #[arg(long, default_value = "0.0.0.0", env = "RABBET_HOST")]
    pub host: String,

    /// Path to master SQLite database.
    #[arg(long, default_value = "rabbet.db", env = "RABBET_DB_PATH")]
    pub db_path: PathBuf,

    /// Secret bearer token protecting the `/admin` console and admin API.
    #[arg(long, env = "RABBET_ADMIN_TOKEN")]
    pub admin_token: Option<String>,
}

/// Arguments for `rabbet mine-oss`.
#[derive(Debug, Args)]
pub struct MineOssArgs {
    /// Local directory path to walk and mine.
    #[arg(short, long)]
    pub path: PathBuf,

    /// Path to master SQLite database where traces will be inserted.
    #[arg(long, default_value = "rabbet.db")]
    pub db_path: PathBuf,

    /// Maximum number of synthetic traces to extract.
    #[arg(long, default_value = "10000")]
    pub max_traces: usize,
}

/// Arguments for `rabbet export-parquet`.
#[derive(Debug, Args)]
pub struct ExportParquetArgs {
    /// Path to master SQLite database.
    #[arg(long, default_value = "rabbet.db")]
    pub db_path: PathBuf,

    /// Target directory to output router_sft, hole_filler_sft, and hole_filler_dpo shards.
    #[arg(short, long, default_value = "dist/parquet")]
    pub output_dir: PathBuf,
}

/// Arguments for `rabbet mine-patterns`.
#[derive(Debug, Args)]
pub struct MinePatternsArgs {
    /// Path to master SQLite database.
    #[arg(long, default_value = "rabbet.db")]
    pub db_path: PathBuf,

    /// Minimum frequency of identical AST skeletons to trigger candidate proposal.
    #[arg(long, default_value = "3")]
    pub min_frequency: usize,

    /// Optional directory to export candidate `.dado` pattern YAML files.
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,
}

/// Arguments for `rabbet publish-hub`.
#[derive(Debug, Args)]
pub struct PublishHubArgs {
    /// Directory containing exported Parquet shards and README.md.
    #[arg(short, long, default_value = "dist/parquet")]
    pub bundle_dir: PathBuf,

    /// Target Hugging Face repository ID (e.g. "gaugeblock/synthesis-corpus").
    #[arg(long, default_value = "gaugeblock/synthesis-corpus")]
    pub repo_id: String,

    /// Hugging Face API token (or set via HF_TOKEN environment variable).
    #[arg(long, env = "HF_TOKEN")]
    pub token: Option<String>,

    /// Run in dry-run mode without making network requests.
    #[arg(long, default_value = "false")]
    pub dry_run: bool,
}

/// Arguments for `rabbet admin`.
#[derive(Debug, Args)]
pub struct AdminArgs {
    /// Path to master SQLite database.
    #[arg(long, default_value = "rabbet.db")]
    pub db_path: PathBuf,

    #[command(subcommand)]
    pub action: AdminSubcommand,
}

/// Admin subcommands.
#[derive(Debug, Subcommand)]
pub enum AdminSubcommand {
    /// Shows summary corpus statistics across tiers and languages.
    Stats,
    /// Triggers Parquet dataset compaction.
    Compact {
        #[arg(short, long, default_value = "dist/parquet")]
        output_dir: PathBuf,
    },
    /// Lists discovered pattern candidates from Gold traces.
    Candidates,
    /// Approves a pattern candidate and exports it to `.dado` format.
    Approve {
        candidate_id: String,
        #[arg(short, long, default_value = ".dado/patterns")]
        output_dir: PathBuf,
    },
}

