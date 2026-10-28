//! Public batch trace ingestion handler for `POST /api/v1/traces`.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use rabbet_core::db::MasterStore;
use rabbet_core::gate::process_ingest_batch;
use rabbet_core::model::BatchIngestRequest;
use std::sync::Arc;

/// Shared application state injected into Axum route handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    pub store: MasterStore,
    pub admin_token: Option<String>,
}

/// Ingests a batch of anonymized synthesis traces through the 4-stage validation gate.
pub async fn handle_batch_ingest(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BatchIngestRequest>,
) -> impl IntoResponse {
    tracing::info!(
        "Received telemetry batch with {} traces from client {}",
        payload.traces.len(),
        payload.client_version
    );

    match process_ingest_batch(&state.store, &payload) {
        Ok(resp) => {
            tracing::info!(
                "Batch ingested: {} accepted, {} duplicates, {} rejected",
                resp.accepted,
                resp.duplicates,
                resp.rejected
            );
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(err) => {
            tracing::error!("Batch processing error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": err.to_string()
                })),
            )
                .into_response()
        }
    }
}
