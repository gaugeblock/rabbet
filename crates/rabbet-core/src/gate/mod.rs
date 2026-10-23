//! Ingestion validation gate for public community telemetry submissions.

pub mod scrubber;
pub mod validator;

use crate::db::MasterStore;
use crate::error::Result;
use crate::model::{BatchIngestRequest, BatchIngestResponse, IngestResultDetail};
pub use scrubber::verify_and_rescrub_trace;
pub use validator::{compute_canonical_trace_hash, validate_trace_structure, verify_trace_hash};

/// Evaluates a submitted batch of traces through the 4-stage gate and persists to master store.
pub fn process_ingest_batch(
    store: &MasterStore,
    batch: &BatchIngestRequest,
) -> Result<BatchIngestResponse> {
    let mut accepted = 0;
    let mut duplicates = 0;
    let mut rejected = 0;
    let mut details = Vec::with_capacity(batch.traces.len());

    for entry in &batch.traces {
        // Stage 1: Verify SHA-256 hash
        if let Err(err) = verify_trace_hash(entry) {
            rejected += 1;
            details.push(IngestResultDetail {
                content_hash: entry.content_hash.clone(),
                trace_id: entry.trace.trace_id,
                status: "rejected".to_string(),
                reason: Some(err.to_string()),
            });
            continue;
        }

        // Stage 2: Re-scrub PII & scan for raw secrets
        let mut scrubbed_entry = entry.clone();
        if let Err(err) = verify_and_rescrub_trace(&mut scrubbed_entry.trace) {
            rejected += 1;
            details.push(IngestResultDetail {
                content_hash: entry.content_hash.clone(),
                trace_id: entry.trace.trace_id,
                status: "rejected".to_string(),
                reason: Some(err.to_string()),
            });
            continue;
        }

        // Stage 3: Structural AST validation
        if let Err(err) = validate_trace_structure(&scrubbed_entry.trace) {
            rejected += 1;
            details.push(IngestResultDetail {
                content_hash: entry.content_hash.clone(),
                trace_id: entry.trace.trace_id,
                status: "rejected".to_string(),
                reason: Some(err.to_string()),
            });
            continue;
        }

        // Stage 4: Idempotent DB insertion
        match store.insert_trace(&scrubbed_entry, false, None) {
            Ok(true) => {
                accepted += 1;
                details.push(IngestResultDetail {
                    content_hash: entry.content_hash.clone(),
                    trace_id: entry.trace.trace_id,
                    status: "accepted".to_string(),
                    reason: None,
                });
            }
            Ok(false) => {
                duplicates += 1;
                details.push(IngestResultDetail {
                    content_hash: entry.content_hash.clone(),
                    trace_id: entry.trace.trace_id,
                    status: "duplicate".to_string(),
                    reason: Some("Trace already exists in master store".to_string()),
                });
            }
            Err(err) => {
                rejected += 1;
                details.push(IngestResultDetail {
                    content_hash: entry.content_hash.clone(),
                    trace_id: entry.trace.trace_id,
                    status: "rejected".to_string(),
                    reason: Some(err.to_string()),
                });
            }
        }
    }

    Ok(BatchIngestResponse {
        accepted,
        duplicates,
        rejected,
        details,
    })
}
