use super::parse_metadata;
use anyhow::Result;
use clap::Args;
use dna::services::{ArtifactService, ConfigService, ContentFormat};
use std::path::PathBuf;

#[derive(Args)]
pub struct AddArgs {
    /// Artifact kind (must be registered via 'dna kind add')
    pub kind: String,

    /// Artifact content - the full text to be embedded and stored
    pub content: String,

    /// Optional name slug for human-readable identification
    #[arg(long)]
    pub name: Option<String>,

    /// Content format [default: markdown] [possible values: markdown, yaml, json, openapi, text]
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Label as key=value pair for filtering and organization.
    /// Can be repeated. Example: --label domain=auth --label priority=high
    #[arg(long = "label", short = 'l')]
    pub labels: Vec<String>,

    /// Additional context for improved semantic retrieval.
    /// Describe relationships, domain concepts, or purpose.
    /// Gets its own embedding for context-aware search.
    #[arg(long, short = 'c')]
    pub context: Option<String>,
}

#[derive(Args)]
pub struct GetArgs {
    /// Artifact ID
    pub id: String,

    /// Retrieve artifact at specific database version
    #[arg(long)]
    pub version: Option<u64>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Artifact ID to update
    pub id: String,

    /// New content (replaces existing, triggers re-embedding)
    #[arg(long)]
    pub content: Option<String>,

    /// New name slug
    #[arg(long)]
    pub name: Option<String>,

    /// Change artifact kind (must be registered)
    #[arg(long)]
    pub kind: Option<String>,

    /// Add or update label. Use empty value to remove: --label key=
    /// Can be repeated. Example: --label status=done --label draft=
    #[arg(long = "label", short = 'l')]
    pub labels: Vec<String>,

    /// New context (replaces existing, triggers re-embedding).
    /// Use empty string to remove: --context ""
    #[arg(long, short = 'c')]
    pub context: Option<String>,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// Artifact ID
    pub id: String,
}

async fn create_service() -> Result<ArtifactService> {
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

    Ok(ArtifactService::new(db, embedding))
}

pub async fn execute_add(args: AddArgs) -> Result<()> {
    let service = create_service().await?;
    let format: ContentFormat = args.format.parse()?;
    let labels = parse_metadata(&args.labels)?;

    let artifact = service
        .add(
            args.kind,
            args.content,
            format,
            args.name,
            labels,
            args.context,
        )
        .await?;
    println!("Added artifact: {}", artifact.id);
    println!("{}", serde_json::to_string_pretty(&artifact)?);
    Ok(())
}

pub async fn execute_get(args: GetArgs) -> Result<()> {
    let service = create_service().await?;

    let artifact = match args.version {
        Some(version) => service.get_at_version(&args.id, version).await?,
        None => service.get(&args.id).await?,
    };

    if let Some(artifact) = artifact {
        println!("{}", serde_json::to_string_pretty(&artifact)?);
    } else {
        println!("Artifact not found: {}", args.id);
    }
    Ok(())
}

pub async fn execute_update(args: UpdateArgs) -> Result<()> {
    let service = create_service().await?;
    let labels = if args.labels.is_empty() {
        None
    } else {
        Some(parse_metadata(&args.labels)?)
    };

    let artifact = service
        .update(
            &args.id,
            args.content,
            args.name,
            args.kind,
            labels,
            args.context,
        )
        .await?;
    println!("Updated artifact: {}", artifact.id);
    println!("{}", serde_json::to_string_pretty(&artifact)?);
    Ok(())
}

pub async fn execute_remove(args: RemoveArgs) -> Result<()> {
    let service = create_service().await?;

    if service.remove(&args.id).await? {
        println!("Removed artifact: {}", args.id);
    } else {
        println!("Artifact not found: {}", args.id);
    }
    Ok(())
}
