//! Parquet file writer using Snappy compression and Arrow RecordBatches.

use crate::error::Result;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;

/// Writes one or more Arrow RecordBatches to a `.parquet` file with Snappy compression.
pub fn write_parquet_file(output_path: &Path, batches: &[RecordBatch]) -> Result<usize> {
    if batches.is_empty() {
        return Ok(0);
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(output_path)?;
    let schema = batches[0].schema();
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
    let mut total_rows = 0;

    for batch in batches {
        writer.write(batch)?;
        total_rows += batch.num_rows();
    }

    writer.close()?;
    Ok(total_rows)
}
