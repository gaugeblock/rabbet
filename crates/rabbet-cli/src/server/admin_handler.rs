//! Token-protected Maintainer Admin API handlers (`/api/v1/admin/*`).

use super::ingest_handler::AppState;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use rabbet_core::miner::export_dado_pattern_file;
use rabbet_core::parquet::export_parquet_shards;
use serde_json::json;
use std::sync::Arc;

fn check_auth(state: &AppState, headers: &HeaderMap) -> bool {
    let expected = match &state.admin_token {
        Some(token) if !token.is_empty() => token,
        _ => return true, // No token configured -> open admin
    };

    if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok())
        && let Some(token) = auth_header.strip_prefix("Bearer ")
    {
        return token.trim() == expected;
    }
    false
}

/// Returns summary corpus metrics for the admin console.
pub async fn handle_admin_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_auth(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response();
    }
    match state.store.get_stats() {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Triggers on-demand Parquet compaction into `dist/parquet`.
pub async fn handle_admin_compact(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_auth(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response();
    }
    let output_dir = std::path::PathBuf::from("dist/parquet");
    match export_parquet_shards(&state.store, &output_dir) {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Returns pending pattern candidates mined from Gold traces.
pub async fn handle_admin_candidates(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_auth(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response();
    }
    match state.store.list_pattern_candidates(None) {
        Ok(candidates) => (StatusCode::OK, Json(candidates)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Approves a pattern candidate and exports it to `.dado/patterns`.
pub async fn handle_admin_approve(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(candidate_id): Path<String>,
) -> impl IntoResponse {
    if !check_auth(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response();
    }
    let candidates = match state.store.list_pattern_candidates(None) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    if let Some(cand) = candidates.iter().find(|c| c.candidate_id == candidate_id) {
        let dado_dir = std::path::PathBuf::from(".dado/patterns");
        if let Err(e) = export_dado_pattern_file(cand, &dado_dir) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
        let _ = state
            .store
            .update_candidate_status(&candidate_id, "approved");
        (
            StatusCode::OK,
            Json(json!({"status": "approved", "candidate_id": candidate_id})),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Candidate not found"})),
        )
            .into_response()
    }
}
