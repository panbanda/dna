use crate::services::{ArtifactService, ConfigService, SearchFilters, SearchService};
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    query: String,

    /// Filter by artifact type
    #[arg(long)]
    r#type: Option<String>,

    /// Filter by metadata key=value
    #[arg(long = "filter")]
    filters: Vec<String>,

    /// Limit number of results
    #[arg(long, default_value = "10")]
    limit: usize,
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter by artifact type
    #[arg(long)]
    r#type: Option<String>,

    /// Filter by metadata key=value
    #[arg(long = "filter")]
    filters: Vec<String>,

    /// Show only artifacts since timestamp
    #[arg(long)]
    since: Option<String>,

    /// Limit number of results
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Args)]
pub struct ChangesArgs {
    /// Timestamp or git ref
    #[arg(long)]
    since: String,
}

#[derive(Args)]
pub struct ReindexArgs {
    /// Force reindex even if model hasn't changed
    #[arg(long)]
    force: bool,
}

pub async fn execute_search(args: SearchArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let db_path = project_root.join(".dna").join("db").join("artifacts.lance");
    let db = std::sync::Arc::new(crate::db::lance::LanceDatabase::new(&db_path).await?);
    let embedding = crate::embedding::create_provider(&config.model).await?;

    let search_service = SearchService::new(db, embedding);

    let artifact_type = args.r#type.as_ref().map(|s| s.parse()).transpose()?;
    let metadata = parse_metadata(&args.filters)?;

    let filters = SearchFilters {
        artifact_type,
        metadata,
        since: None,
        limit: Some(args.limit),
    };

    let results = search_service.search(&args.query, filters).await?;

    println!("Found {} results:", results.len());
    for result in results {
        println!("\n  ID: {}", result.artifact.id);
        println!("  Type: {}", result.artifact.artifact_type);
        println!("  Score: {:.4}", result.score);
        println!(
            "  Content: {}...",
            &result.artifact.content[..result.artifact.content.len().min(100)]
        );
    }

    Ok(())
}

pub async fn execute_list(args: ListArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let db_path = project_root.join(".dna").join("db").join("artifacts.lance");
    let db = std::sync::Arc::new(crate::db::lance::LanceDatabase::new(&db_path).await?);
    let embedding = crate::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db, embedding);

    let artifact_type = args.r#type.as_ref().map(|s| s.parse()).transpose()?;
    let metadata = parse_metadata(&args.filters)?;
    let since = args
        .since
        .as_ref()
        .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()?;

    let filters = SearchFilters {
        artifact_type,
        metadata,
        since,
        limit: args.limit,
    };

    let artifacts = service.list(filters).await?;

    println!("Found {} artifacts:", artifacts.len());
    for artifact in artifacts {
        println!(
            "  {} - {} ({})",
            artifact.id, artifact.artifact_type, artifact.format
        );
    }

    Ok(())
}

pub async fn execute_changes(args: ChangesArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let db_path = project_root.join(".dna").join("db").join("artifacts.lance");
    let db = std::sync::Arc::new(crate::db::lance::LanceDatabase::new(&db_path).await?);
    let embedding = crate::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db, embedding);

    // Try parsing as RFC3339 timestamp first
    let since = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&args.since) {
        dt.with_timezone(&chrono::Utc)
    } else {
        // TODO: Parse git ref and get commit timestamp
        return Err(anyhow::anyhow!(
            "Git ref parsing not yet implemented. Use RFC3339 timestamp."
        ));
    };

    let artifacts = service.changes(since).await?;

    println!("Found {} changed artifacts:", artifacts.len());
    for artifact in artifacts {
        println!(
            "  {} - {} (updated: {})",
            artifact.id, artifact.artifact_type, artifact.updated_at
        );
    }

    Ok(())
}

pub async fn execute_reindex(args: ReindexArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let db_path = project_root.join(".dna").join("db").join("artifacts.lance");
    let db = std::sync::Arc::new(crate::db::lance::LanceDatabase::new(&db_path).await?);
    let embedding = crate::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db.clone(), embedding.clone());
    let search_service = SearchService::new(db, embedding);

    if !args.force {
        let inconsistent = search_service.check_embedding_consistency().await?;
        if inconsistent.is_empty() {
            println!("All artifacts are indexed with the current model.");
            return Ok(());
        }
        println!(
            "Found {} artifacts with stale embeddings.",
            inconsistent.len()
        );
    }

    println!("Reindexing artifacts...");
    let count = service.reindex().await?;
    println!("Reindexed {} artifacts.", count);

    Ok(())
}

fn parse_metadata(pairs: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in pairs {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid metadata format: {}", pair));
        }
        map.insert(parts[0].to_string(), parts[1].to_string());
    }
    Ok(map)
}
