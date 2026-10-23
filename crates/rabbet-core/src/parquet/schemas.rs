//! Arrow schema definitions for System 1 Router, Hole Filler SFT, and Hole Filler DPO shards.

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::Arc;

/// Schema for `router_sft.parquet` (maps scope signature + intent -> pattern ID).
pub fn router_sft_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("trace_id", DataType::Utf8, false),
        Field::new("language", DataType::Utf8, false),
        Field::new("scope_signature", DataType::Utf8, false),
        Field::new("pattern_id", DataType::Utf8, false),
        Field::new("quality_tier", DataType::Utf8, false),
    ]))
}

/// Schema for `hole_filler_sft.parquet` (maps template + hole + scope -> expression).
pub fn hole_filler_sft_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("trace_id", DataType::Utf8, false),
        Field::new("language", DataType::Utf8, false),
        Field::new("scope_signature", DataType::Utf8, false),
        Field::new("pattern_id", DataType::Utf8, true),
        Field::new("hole_id", DataType::Utf8, false),
        Field::new("expected_type", DataType::Utf8, false),
        Field::new("ast_node_kind", DataType::Utf8, false),
        Field::new("target_expression", DataType::Utf8, false),
        Field::new("quality_tier", DataType::Utf8, false),
    ]))
}

/// Schema for `hole_filler_dpo.parquet` (preference pairs with compiler diagnostic).
pub fn hole_filler_dpo_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("trace_id", DataType::Utf8, false),
        Field::new("language", DataType::Utf8, false),
        Field::new("prompt", DataType::Utf8, false),
        Field::new("chosen", DataType::Utf8, false),
        Field::new("rejected", DataType::Utf8, false),
        Field::new("compiler_diagnostic", DataType::Utf8, true),
    ]))
}
