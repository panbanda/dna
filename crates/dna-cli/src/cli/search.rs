use super::parse_metadata;
use anyhow::Result;
use clap::Args;
use dna::services::{ArtifactService, ConfigService, SearchFilters, SearchService};
use std::path::PathBuf;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    query: String,

    /// Filter by artifact kind
    #[arg(long)]
    kind: Option<String>,

    /// Filter by metadata key=value
    #[arg(long = "filter")]
    filters: Vec<String>,

    /// Limit number of results
    #[arg(long, default_value = "10")]
    limit: usize,
}

#[derive(Args)]
pub struct ListArgs {
    /// Filter by artifact kind
    #[arg(long)]
    kind: Option<String>,

    /// Filter by metadata key=value
    #[arg(long = "filter")]
    filters: Vec<String>,

    /// Show only artifacts updated after timestamp
    #[arg(long)]
    after: Option<String>,

    /// Show only artifacts updated before timestamp
    #[arg(long)]
    before: Option<String>,

    /// Limit number of results
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Args)]
pub struct ChangesArgs {
    /// Show artifacts updated after timestamp
    #[arg(long)]
    after: Option<String>,

    /// Show artifacts updated before timestamp
    #[arg(long)]
    before: Option<String>,
}

/// Arguments for the reindex command.
///
/// The reindex command regenerates embeddings for artifacts in the database.
/// Use this when the embedding model changes, artifacts become stale, or you
/// need to rebuild the vector index.
///
/// # Embedding Types
///
/// - **Content embeddings**: Derived from the artifact's content field. Used for
///   semantic search queries like "find authentication code".
/// - **Context embeddings**: Derived from metadata, relationships, and structural
///   information. Used for contextual queries like "find artifacts related to X".
///
/// # Examples
///
/// Reindex everything (both content and context embeddings):
/// ```sh
/// dna reindex --all
/// ```
///
/// Reindex only content embeddings for faster semantic search:
/// ```sh
/// dna reindex --content
/// ```
///
/// Reindex a specific artifact by ID:
/// ```sh
/// dna reindex --id abc123
/// ```
///
/// Reindex all "spec" artifacts modified in the last week:
/// ```sh
/// dna reindex --content --kind spec --since 2024-01-15
/// ```
///
/// Preview what would be reindexed without making changes:
/// ```sh
/// dna reindex --all --dry-run
/// ```
///
/// Force reindex even if the embedding model hasn't changed:
/// ```sh
/// dna reindex --all --force
/// ```
#[derive(Args)]
pub struct ReindexArgs {
    /// Reindex all embeddings (content + context).
    /// Use this for a full rebuild after model changes or database corruption.
    #[arg(long)]
    pub all: bool,

    /// Reindex content embeddings only.
    /// Content embeddings are derived from the artifact's main content field
    /// and are used for semantic search. Faster than --all when you only need
    /// to update search functionality.
    #[arg(long)]
    pub content: bool,

    /// Reindex context embeddings only.
    /// Context embeddings capture metadata, relationships, and structural info.
    /// Use this when artifact relationships or metadata have changed but content
    /// remains the same.
    #[arg(long)]
    pub context: bool,

    /// Only reindex artifacts of this kind (e.g., "spec", "code", "doc").
    /// Useful for targeted reindexing when only certain artifact types need updates.
    #[arg(long)]
    pub kind: Option<String>,

    /// Only reindex artifacts matching this label (can be repeated).
    /// Labels are key=value pairs. Multiple labels are AND-ed together.
    /// Example: --label team=platform --label priority=high
    #[arg(long = "label", short = 'l')]
    pub labels: Vec<String>,

    /// Reindex a specific artifact by its ID.
    /// Use this for surgical updates to individual artifacts.
    #[arg(long)]
    pub id: Option<String>,

    /// Only reindex artifacts modified after this date (YYYY-MM-DD).
    /// Useful for incremental reindexing after bulk imports or migrations.
    #[arg(long)]
    pub since: Option<String>,

    /// Show what would be reindexed without actually doing it.
    /// Outputs the list of artifacts that match the filters.
    #[arg(long)]
    pub dry_run: bool,

    /// Reindex even if the embedding model hasn't changed.
    /// By default, reindex skips artifacts whose embeddings are already
    /// up-to-date with the current model. Use --force to override this.
    #[arg(long)]
    pub force: bool,
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
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;

    let search_service = SearchService::new(db, embedding);

    let metadata = parse_metadata(&args.filters)?;

    let filters = SearchFilters {
        kind: args.kind,
        metadata,
        after: None,
        before: None,
        limit: Some(args.limit),
    };

    let results = search_service.search(&args.query, filters).await?;

    println!("Found {} results:", results.len());
    for result in results {
        println!("\n  ID: {}", result.artifact.id);
        println!("  Kind: {}", result.artifact.kind);
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
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db, embedding);

    let metadata = parse_metadata(&args.filters)?;
    let after = args
        .after
        .as_ref()
        .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()?;
    let before = args
        .before
        .as_ref()
        .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()?;

    let filters = SearchFilters {
        kind: args.kind,
        metadata,
        after,
        before,
        limit: args.limit,
    };

    let artifacts = service.list(filters).await?;

    println!("Found {} artifacts:", artifacts.len());
    for artifact in artifacts {
        println!(
            "  {} - {} ({})",
            artifact.id, artifact.kind, artifact.format
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
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db, embedding);

    let after = args
        .after
        .as_ref()
        .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()?;
    let before = args
        .before
        .as_ref()
        .map(|s| chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc)))
        .transpose()?;

    let filters = SearchFilters {
        after,
        before,
        ..Default::default()
    };

    let artifacts = service.list(filters).await?;

    println!("Found {} changed artifacts:", artifacts.len());
    for artifact in artifacts {
        println!(
            "  {} - {} (updated: {})",
            artifact.id, artifact.kind, artifact.updated_at
        );
    }

    Ok(())
}

pub async fn execute_reindex(args: ReindexArgs) -> Result<()> {
    // Show help if no action flags are provided
    if !args.all && !args.content && !args.context && args.id.is_none() {
        println!("Specify what to reindex:");
        println!("  --all        Reindex all embeddings (content + context)");
        println!("  --content    Reindex content embeddings only");
        println!("  --context    Reindex context embeddings only");
        println!("  --id <ID>    Reindex specific artifact");
        println!();
        println!("Optional filters:");
        println!("  --kind <KIND>        Only artifacts of this kind");
        println!("  --label <KEY=VALUE>  Only artifacts with this label");
        println!("  --since <DATE>       Only artifacts modified after date");
        println!();
        println!("Options:");
        println!("  --dry-run    Show what would be reindexed");
        println!("  --force      Reindex even if model unchanged");
        return Ok(());
    }

    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;

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

    // TODO: Implement filtering logic for kind, labels, id, since, dry_run
    // TODO: Implement separate content vs context reindexing

    println!("Reindexing artifacts...");
    let count = service.reindex().await?;
    println!("Reindexed {} artifacts.", count);

    Ok(())
}
