use super::parse_metadata;
use anyhow::Result;
use clap::Args;
use dna::services::{ArtifactService, ConfigService, ContentFormat};
use std::path::PathBuf;

#[derive(Args)]
pub struct AddArgs {
    /// Artifact kind (e.g. intent, contract, algorithm)
    pub kind: String,

    /// Artifact content
    pub content: String,

    /// Optional name slug
    #[arg(long)]
    pub name: Option<String>,

    /// Content format
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Metadata key=value pairs
    #[arg(long = "meta")]
    pub metadata: Vec<String>,
}

#[derive(Args)]
pub struct GetArgs {
    /// Artifact ID
    pub id: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Artifact ID
    pub id: String,

    /// New content
    #[arg(long)]
    pub content: Option<String>,

    /// New name
    #[arg(long)]
    pub name: Option<String>,

    /// New kind
    #[arg(long)]
    pub kind: Option<String>,

    /// Metadata key=value pairs to add/update
    #[arg(long = "meta")]
    pub metadata: Vec<String>,
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
    let metadata = parse_metadata(&args.metadata)?;

    let artifact = service
        .add(args.kind, args.content, format, args.name, metadata)
        .await?;
    println!("Added artifact: {}", artifact.id);
    println!("{}", serde_json::to_string_pretty(&artifact)?);
    Ok(())
}

pub async fn execute_get(args: GetArgs) -> Result<()> {
    let service = create_service().await?;

    if let Some(artifact) = service.get(&args.id).await? {
        println!("{}", serde_json::to_string_pretty(&artifact)?);
    } else {
        println!("Artifact not found: {}", args.id);
    }
    Ok(())
}

pub async fn execute_update(args: UpdateArgs) -> Result<()> {
    let service = create_service().await?;
    let metadata = if args.metadata.is_empty() {
        None
    } else {
        Some(parse_metadata(&args.metadata)?)
    };

    let artifact = service
        .update(&args.id, args.content, args.name, args.kind, metadata)
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
