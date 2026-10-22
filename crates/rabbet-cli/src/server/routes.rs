//! Axum router configuration and middleware pipeline for Rabbet.

use super::admin_handler::{
    handle_admin_approve, handle_admin_candidates, handle_admin_compact, handle_admin_stats,
};
use super::ingest_handler::{AppState, handle_batch_ingest};
use super::static_assets::get_admin_dashboard_html;
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

async fn render_admin_dashboard() -> Html<&'static str> {
    Html(get_admin_dashboard_html())
}

/// Builds the complete Axum application router.
pub fn build_app_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/traces", post(handle_batch_ingest))
        .route("/admin", get(render_admin_dashboard))
        .route("/api/v1/admin/stats", get(handle_admin_stats))
        .route("/api/v1/admin/compact", post(handle_admin_compact))
        .route("/api/v1/admin/candidates", get(handle_admin_candidates))
        .route("/api/v1/admin/approve/{candidate_id}", post(handle_admin_approve))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

