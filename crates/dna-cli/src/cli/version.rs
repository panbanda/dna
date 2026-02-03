use anyhow::Result;
use clap::Args;
use dna::db::Database;
use dna::services::ConfigService;
use std::path::PathBuf;

#[derive(Args)]
pub struct PruneArgs {
    /// Keep the last N versions [default: 1]
    #[arg(long, default_value = "1")]
    pub keep_versions: usize,

    /// Remove versions older than N days (not yet implemented)
    #[arg(long)]
    pub older_than: Option<u64>,

    /// Show what would be pruned without doing it
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct VersionsArgs {
    /// Show last N versions [default: 20]
    #[arg(long, default_value = "20")]
    pub limit: usize,
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

pub async fn execute_prune(args: PruneArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = dna::db::lance::LanceDatabase::new(&storage_uri).await?;

    if args.older_than.is_some() {
        return Err(anyhow::anyhow!("--older-than is not yet implemented"));
    }

    if args.dry_run {
        println!("Dry run - no changes will be made\n");
    }

    // Compact database
    if args.dry_run {
        println!("Would compact database...");
    } else {
        println!("Compacting database...");
        let compact_stats = db.compact().await?;
        println!("Files merged: {}", compact_stats.files_merged);
        println!("Bytes saved: {}", format_bytes(compact_stats.bytes_saved));
    }

    println!();

    // Cleanup old versions
    if args.dry_run {
        let versions = db.list_versions(None).await?;
        let versions_to_remove = versions.len().saturating_sub(args.keep_versions);
        println!(
            "Would clean up old versions (keeping {})...",
            args.keep_versions
        );
        println!("Versions that would be removed: {}", versions_to_remove);
    } else {
        println!(
            "Cleaning up old versions (keeping {})...",
            args.keep_versions
        );
        let cleanup_stats = db.cleanup_versions(args.keep_versions).await?;
        println!("Versions removed: {}", cleanup_stats.versions_removed);
        println!("Bytes freed: {}", format_bytes(cleanup_stats.bytes_freed));
    }

    Ok(())
}

pub async fn execute_versions(args: VersionsArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = dna::db::lance::LanceDatabase::new(&storage_uri).await?;

    let versions = db.list_versions(Some(args.limit)).await?;

    if versions.is_empty() {
        println!("No versions found.");
        return Ok(());
    }

    println!("Database versions:");
    println!("  {:>8}  Timestamp", "Version");
    for version_info in versions {
        println!(
            "  {:>8}  {}",
            version_info.version,
            version_info.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
    }

    Ok(())
}
