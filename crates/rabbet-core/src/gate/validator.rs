//! Cryptographic hash verification and structural AST sanity checks.

use crate::error::{RabbetError, Result};
use crate::model::{AnonymizedSynthesisTrace, SpoolEntry};

/// Computes the canonical SHA-256 content hash of an anonymized trace.
///
/// Delegates to [`SpoolEntry::hash_trace`] so the hash a writer records is byte-for-byte the hash
/// the gate recomputes (`GB-182`); a second copy of this algorithm here is how the two drifted.
pub fn compute_canonical_trace_hash(trace: &AnonymizedSynthesisTrace) -> Result<String> {
    Ok(SpoolEntry::hash_trace(trace)?)
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

    // The rule is "carries evidence", not "carries a hole". A scope-level record is what
    // `storypole verify` writes: its meaning lives in the scope signature plus the verification
    // report, and a task with no target symbols legitimately has no filled holes. Requiring a hole
    // discarded 10 of 18 records on the first real ingest (GB-207), so a verified scope now counts.
    if trace.filled_holes.is_empty() && trace.rejected_attempt.is_none() {
        let verified_scope = trace.verification.ast_valid && trace.verification.typecheck_passed;
        if !verified_scope {
            return Err(RabbetError::InvalidAst(
                "Trace must contain a filled hole, a rejected attempt, or a verified scope"
                    .to_string(),
            ));
        }
    }

    // A hole whose body is empty or a placeholder carries no expression, so a pair built from it
    // trains nothing. Seven spool records reached the corpus this way before joiner started sending
    // real bodies (GB-225); the guard belongs on the receiving side too, because any producer - not
    // just joiner - can hand the corpus an untrainable pair.
    for (hole, body) in &trace.filled_holes {
        let body = body.trim();
        if body.is_empty() || body == "<filled>" {
            return Err(RabbetError::InvalidAst(format!(
                "Filled hole `{hole}` has no expression (placeholder or empty body)"
            )));
        }
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
