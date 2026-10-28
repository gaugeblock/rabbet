//! CLI command handler for `rabbet publish-hub`.

use crate::args::PublishHubArgs;
use rabbet_core::publish::{PublishOptions, publish_dataset_to_hub};

/// Executes the `rabbet publish-hub` command.
pub async fn exec_publish_hub(
    args: PublishHubArgs,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Publishing dataset bundle at '{}' to Hugging Face Hub: {}",
        args.bundle_dir.display(),
        args.repo_id
    );

    let options = PublishOptions {
        repo_id: args.repo_id,
        token: args.token,
        dry_run: args.dry_run,
    };

    let uploaded = publish_dataset_to_hub(&args.bundle_dir, &options).await?;
    println!("Published {} file(s) to Hub successfully:", uploaded.len());
    for file in uploaded {
        println!("  - {}", file);
    }

    Ok(())
}
