//! Pipeline for exporting master SQLite records into Arrow RecordBatches and Parquet shards.

use super::schemas::{hole_filler_dpo_schema, hole_filler_sft_schema, router_sft_schema};
use super::writer::write_parquet_file;
use crate::db::MasterStore;
use crate::error::Result;
use crate::model::{AnonymizedSynthesisTrace, TelemetryTier, TraceFilter};
use arrow::array::{ArrayRef, StringBuilder};
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Result summary of exporting Parquet dataset shards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParquetExportResult {
    /// Number of rows written to router_sft.parquet.
    pub router_sft_rows: usize,
    /// Path to router_sft.parquet.
    pub router_sft_path: PathBuf,
    /// Number of rows written to hole_filler_sft.parquet.
    pub hole_filler_sft_rows: usize,
    /// Path to hole_filler_sft.parquet.
    pub hole_filler_sft_path: PathBuf,
    /// Number of rows written to hole_filler_dpo.parquet.
    pub hole_filler_dpo_rows: usize,
    /// Path to hole_filler_dpo.parquet.
    pub hole_filler_dpo_path: PathBuf,
}

/// Exports all 3 canonical Parquet shards from the master SQLite database into `output_dir`.
pub fn export_parquet_shards(
    store: &MasterStore,
    output_dir: &Path,
) -> Result<ParquetExportResult> {
    std::fs::create_dir_all(output_dir)?;
    let all_traces = store.query_traces(&TraceFilter::default())?;

    // 1. Build router_sft
    let (router_batch, router_rows) = build_router_sft_batch(&all_traces)?;
    let router_path = output_dir.join("router_sft.parquet");
    write_parquet_file(&router_path, &[router_batch])?;

    // 2. Build hole_filler_sft
    let (filler_batch, filler_rows) = build_hole_filler_sft_batch(&all_traces)?;
    let filler_path = output_dir.join("hole_filler_sft.parquet");
    write_parquet_file(&filler_path, &[filler_batch])?;

    // 3. Build hole_filler_dpo
    let (dpo_batch, dpo_rows) = build_hole_filler_dpo_batch(&all_traces)?;
    let dpo_path = output_dir.join("hole_filler_dpo.parquet");
    write_parquet_file(&dpo_path, &[dpo_batch])?;

    Ok(ParquetExportResult {
        router_sft_rows: router_rows,
        router_sft_path: router_path,
        hole_filler_sft_rows: filler_rows,
        hole_filler_sft_path: filler_path,
        hole_filler_dpo_rows: dpo_rows,
        hole_filler_dpo_path: dpo_path,
    })
}

fn build_router_sft_batch(
    traces: &[(String, AnonymizedSynthesisTrace)],
) -> Result<(RecordBatch, usize)> {
    let mut b_id = StringBuilder::new();
    let mut b_lang = StringBuilder::new();
    let mut b_scope = StringBuilder::new();
    let mut b_pat = StringBuilder::new();
    let mut b_tier = StringBuilder::new();
    let mut count = 0;

    for (_, t) in traces {
        if let Some(pat_id) = &t.selected_pattern_id {
            b_id.append_value(t.trace_id.to_string());
            b_lang.append_value(format!("{:?}", t.language).to_lowercase());
            b_scope.append_value(&t.anonymized_scope_signature);
            b_pat.append_value(pat_id);
            b_tier.append_value(t.tier.to_string());
            count += 1;
        }
    }

    let schema = router_sft_schema();
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(b_id.finish()),
        Arc::new(b_lang.finish()),
        Arc::new(b_scope.finish()),
        Arc::new(b_pat.finish()),
        Arc::new(b_tier.finish()),
    ];
    let batch = RecordBatch::try_new(schema, arrays)?;
    Ok((batch, count))
}

fn build_hole_filler_sft_batch(
    traces: &[(String, AnonymizedSynthesisTrace)],
) -> Result<(RecordBatch, usize)> {
    let mut b_id = StringBuilder::new();
    let mut b_lang = StringBuilder::new();
    let mut b_scope = StringBuilder::new();
    let mut b_pat = StringBuilder::new();
    let mut b_hole = StringBuilder::new();
    let mut b_type = StringBuilder::new();
    let mut b_kind = StringBuilder::new();
    let mut b_expr = StringBuilder::new();
    let mut b_tier = StringBuilder::new();
    let mut count = 0;

    for (_, t) in traces {
        for (hole_id, expr) in &t.filled_holes {
            b_id.append_value(t.trace_id.to_string());
            b_lang.append_value(format!("{:?}", t.language).to_lowercase());
            b_scope.append_value(&t.anonymized_scope_signature);
            if let Some(pat_id) = &t.selected_pattern_id {
                b_pat.append_value(pat_id);
            } else {
                b_pat.append_null();
            }
            b_hole.append_value(hole_id);
            b_type.append_value("inferred");
            b_kind.append_value("expression");
            b_expr.append_value(expr);
            b_tier.append_value(t.tier.to_string());
            count += 1;
        }
    }

    let schema = hole_filler_sft_schema();
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(b_id.finish()),
        Arc::new(b_lang.finish()),
        Arc::new(b_scope.finish()),
        Arc::new(b_pat.finish()),
        Arc::new(b_hole.finish()),
        Arc::new(b_type.finish()),
        Arc::new(b_kind.finish()),
        Arc::new(b_expr.finish()),
        Arc::new(b_tier.finish()),
    ];
    let batch = RecordBatch::try_new(schema, arrays)?;
    Ok((batch, count))
}

fn build_hole_filler_dpo_batch(
    traces: &[(String, AnonymizedSynthesisTrace)],
) -> Result<(RecordBatch, usize)> {
    let mut b_id = StringBuilder::new();
    let mut b_lang = StringBuilder::new();
    let mut b_prompt = StringBuilder::new();
    let mut b_chosen = StringBuilder::new();
    let mut b_rejected = StringBuilder::new();
    let mut b_diag = StringBuilder::new();
    let mut count = 0;

    for (_, t) in traces {
        if t.tier == TelemetryTier::Gold
            && let Some(rejected) = &t.rejected_attempt
        {
            let chosen = t
                .filled_holes
                .iter()
                .map(|(h, e)| format!("{}: {}", h, e))
                .collect::<Vec<_>>()
                .join(", ");
            let prompt = format!(
                "Scope: {}\nPattern: {:?}",
                t.anonymized_scope_signature, t.selected_pattern_id
            );
            let diag = t
                .verification
                .diagnostics
                .iter()
                .map(|d| d.message.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            b_id.append_value(t.trace_id.to_string());
            b_lang.append_value(format!("{:?}", t.language).to_lowercase());
            b_prompt.append_value(&prompt);
            b_chosen.append_value(&chosen);
            b_rejected.append_value(rejected);
            if diag.is_empty() {
                b_diag.append_null();
            } else {
                b_diag.append_value(&diag);
            }
            count += 1;
        }
    }

    let schema = hole_filler_dpo_schema();
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(b_id.finish()),
        Arc::new(b_lang.finish()),
        Arc::new(b_prompt.finish()),
        Arc::new(b_chosen.finish()),
        Arc::new(b_rejected.finish()),
        Arc::new(b_diag.finish()),
    ];
    let batch = RecordBatch::try_new(schema, arrays)?;
    Ok((batch, count))
}
