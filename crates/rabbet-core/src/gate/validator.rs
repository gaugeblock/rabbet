//! Cryptographic hash verification and structural AST sanity checks.

use crate::error::{RabbetError, Result};
use crate::model::{AnonymizedSynthesisTrace, SpoolEntry};
use sha2::{Digest, Sha256};

/// Computes the canonical SHA-256 content hash of an anonymized trace.
pub fn compute_canonical_trace_hash(trace: &AnonymizedSynthesisTrace) -> Result<String> {
    let json = serde_json::to_vec(trace)?;
    let mut hasher = Sha256::new();
    hasher.update(&json);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies that the client-supplied content hash matches the recomputed trace hash.
pub fn verify_trace_hash(entry: &SpoolEntry) -> Result<()> {
    let computed = compute_canonical_trace_hash(&entry.trace)?;
    if entry.content_hash != computed {
        return Err(RabbetError::HashMismatch {
            trace_id: entry.trace.trace_id.to_string(),
            expected: entry.content_hash.clone(),
            computed,
        });
    }
    Ok(())
}

/// Validates structural non-emptiness and AST token integrity of a trace.
pub fn validate_trace_structure(trace: &AnonymizedSynthesisTrace) -> Result<()> {
    if trace.anonymized_scope_signature.trim().is_empty() {
        return Err(RabbetError::InvalidAst(
            "Anonymized scope signature cannot be empty".to_string(),
        ));
    }

    if trace.filled_holes.is_empty() && trace.rejected_attempt.is_none() {
        return Err(RabbetError::InvalidAst(
            "Trace must contain at least one filled hole or a rejected attempt".to_string(),
        ));
    }

    // Check basic bracket/parenthesis balancing in scope signature
    let mut depth: i32 = 0;
    for ch in trace.anonymized_scope_signature.chars() {
        match ch {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            _ => {}
        }
    }
    if depth != 0 {
        return Err(RabbetError::InvalidAst(
            "Unbalanced delimiter in scope signature".to_string(),
        ));
    }

    Ok(())
}
