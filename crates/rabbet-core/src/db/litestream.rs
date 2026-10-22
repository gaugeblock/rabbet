//! Litestream continuous SQLite replication configuration generator and monitor.

use std::path::Path;

/// Generates a standard production `litestream.yml` replication configuration.
pub fn generate_litestream_config(
    db_path: &Path,
    s3_bucket: &str,
    s3_endpoint: Option<&str>,
    s3_path: Option<&str>,
) -> String {
    let db_path_str = db_path.to_string_lossy();
    let prefix = s3_path.unwrap_or("rabbet-wal-replicas");

    let endpoint_line = if let Some(ep) = s3_endpoint {
        format!("\n      endpoint: \"{}\"", ep)
    } else {
        String::new()
    };

    format!(
        r#"# Litestream replication configuration for rabbet SQLite master store
dbs:
  - path: "{db_path_str}"
    replicas:
      - type: s3
        bucket: "{s3_bucket}"
        path: "{prefix}"{endpoint_line}
        sync-interval: 1s
        retention: 720h
"#
    )
}

/// Checks if Litestream is installed and available in the system PATH.
pub fn check_litestream_installed() -> bool {
    std::process::Command::new("litestream")
        .arg("version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}
