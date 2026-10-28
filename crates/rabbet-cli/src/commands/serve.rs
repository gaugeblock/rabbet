//! CLI command handler for `rabbet serve`.

use crate::args::ServeArgs;
use crate::server::run_server;

/// Executes the `rabbet serve` command.
pub async fn exec_serve(args: ServeArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_server(args).await
}
