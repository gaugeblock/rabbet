//! Hugging Face Hub client for uploading Parquet shards and dataset metadata.

use crate::error::{RabbetError, Result};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use std::path::Path;

/// Options configuring publication to Hugging Face Hub.
#[derive(Debug, Clone)]
pub struct PublishOptions {
    /// Target repository ID on Hugging Face (e.g. "gaugeblock/synthesis-corpus").
    pub repo_id: String,
    /// Hugging Face API token (HF_TOKEN).
    pub token: Option<String>,
    /// Whether to run in dry-run mode without actually dispatching HTTP requests.
    pub dry_run: bool,
}

/// Dispatches all Parquet shards and `README.md` from `bundle_dir` to Hugging Face Hub.
pub async fn publish_dataset_to_hub(
    bundle_dir: &Path,
    options: &PublishOptions,
) -> Result<Vec<String>> {
    let mut uploaded_files = Vec::new();
    let entries = std::fs::read_dir(bundle_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            if options.dry_run {
                tracing::info!(
                    "[Dry Run] Would upload {} to https://huggingface.co/datasets/{}",
                    file_name,
                    options.repo_id
                );
                uploaded_files.push(file_name);
                continue;
            }

            let token = options.token.as_ref().ok_or_else(|| {
                RabbetError::Publish(
                    "Missing HF_TOKEN required to publish dataset to Hugging Face Hub".to_string(),
                )
            })?;

            upload_file_to_hf_hub(&path, &file_name, &options.repo_id, token).await?;
            uploaded_files.push(file_name);
        }
    }

    Ok(uploaded_files)
}

async fn upload_file_to_hf_hub(
    file_path: &Path,
    remote_path: &str,
    repo_id: &str,
    token: &str,
) -> Result<()> {
    let bytes = std::fs::read(file_path)?;
    let url = format!(
        "https://huggingface.co/api/datasets/{}/upload/main/{}",
        repo_id, remote_path
    );

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))
            .map_err(|e| RabbetError::Publish(e.to_string()))?,
    );

    let response = client
        .post(&url)
        .headers(headers)
        .body(bytes)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(RabbetError::Publish(format!(
            "Hugging Face upload failed with status {}: {}",
            status, body
        )));
    }

    Ok(())
}
