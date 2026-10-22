//! Axum HTTP daemon bootstrap and lifecycle management.

pub mod admin_handler;
pub mod ingest_handler;
pub mod routes;
pub mod static_assets;

use crate::args::ServeArgs;
use ingest_handler::AppState;
use rabbet_core::db::MasterStore;
use routes::build_app_router;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Runs the Rabbet HTTP server with public ingest gate and Maintainer Admin Console.
pub async fn run_server(args: ServeArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let store = MasterStore::open(&args.db_path)?;
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Rabbet server running on http://{}", addr);
    tracing::info!("Public Ingest: POST http://{}/api/v1/traces", addr);
    tracing::info!("Admin Console: http://{}/admin", addr);

    if args.admin_token.is_some() {
        tracing::info!("Admin console and /api/v1/admin/* protected by RABBET_ADMIN_TOKEN");
    } else {
        tracing::warn!("RABBET_ADMIN_TOKEN not set; admin endpoints are open to all callers");
    }

    let state = Arc::new(AppState {
        store,
        admin_token: args.admin_token,
    });

    let app = build_app_router(state);
    axum::serve(listener, app).await?;
    Ok(())
}

