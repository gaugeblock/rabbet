//! PII and credential re-scrubbing gate for incoming synthesis traces.

use crate::error::{RabbetError, Result};
use crate::model::AnonymizedSynthesisTrace;
use miter_engine::security::{
    REDACTED_SECRET, is_high_entropy_secret, is_secret_literal, scrub_source_text,
};

/// Validates that a trace does not leak unredacted credentials, re-scrubbing in place.
pub fn verify_and_rescrub_trace(trace: &mut AnonymizedSynthesisTrace) -> Result<()> {
    // 1. Re-scrub scope signature
    let scrubbed_scope = scrub_source_text(&trace.anonymized_scope_signature);
    trace.anonymized_scope_signature = scrubbed_scope;

    // 2. Re-scrub filled holes
    for (_, expr) in trace.filled_holes.iter_mut() {
        let scrubbed_expr = scrub_source_text(expr);
        *expr = scrubbed_expr;
    }

    // 3. Re-scrub rejected attempt if present
    if let Some(rejected) = &mut trace.rejected_attempt {
        let scrubbed_rejected = scrub_source_text(rejected);
        *rejected = scrubbed_rejected;
    }

    // 4. Double check for high entropy tokens that escaped regex filters
    if scan_for_raw_secrets(&trace.anonymized_scope_signature) {
        return Err(RabbetError::SecurityViolation(
            "High entropy secret detected in scope signature".to_string(),
        ));
    }

    for (hole_id, expr) in &trace.filled_holes {
        if scan_for_raw_secrets(expr) {
            return Err(RabbetError::SecurityViolation(format!(
                "High entropy secret detected in filled hole '{hole_id}'",
            )));
        }
    }

    Ok(())
}

/// Scans tokens in code for potential raw unredacted secrets or tokens.
pub fn scan_for_raw_secrets(text: &str) -> bool {
    for word in text.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean_word == REDACTED_SECRET {
            continue;
        }
        if clean_word.len() >= 24 && is_high_entropy_secret(clean_word) {
            return true;
        }
        if is_secret_literal(word) && !word.contains(REDACTED_SECRET) {
            return true;
        }
    }
    false
}
