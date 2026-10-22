//! Strongly typed error definitions for `rabbet-core`.

use thiserror::Error;

/// Core error enum for all rabbet domain, storage, and mining operations.
#[derive(Debug, Error)]
pub enum RabbetError {
    /// SQLite storage error.
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// JSON serialization or deserialization failure.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// File or network I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Apache Arrow / Parquet encoding error.
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    /// Arrow error.
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// Network HTTP / Hugging Face request error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Trace content hash validation mismatch.
    #[error("Hash mismatch for trace {trace_id}: expected {expected}, computed {computed}")]
    HashMismatch {
        trace_id: String,
        expected: String,
        computed: String,
    },

    /// Trace rejected due to PII or secret detection.
    #[error("Security gate violation: {0}")]
    SecurityViolation(String),

    /// Trace rejected due to invalid AST syntax.
    #[error("AST syntax validation failed: {0}")]
    InvalidAst(String),

    /// Invalid request or configuration.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Pattern extraction or mining error.
    #[error("Mining error: {0}")]
    Mining(String),

    /// Hub publish error.
    #[error("Hub publish error: {0}")]
    Publish(String),
}

/// Convenience result alias for `rabbet-core`.
pub type Result<T> = std::result::Result<T, RabbetError>;

