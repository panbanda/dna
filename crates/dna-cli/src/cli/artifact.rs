use super::parse_metadata;
use anyhow::Result;
use clap::{Args, Subcommand};
use dna::services::{ArtifactService, ArtifactType, ConfigService, ContentFormat};
use std::path::PathBuf;

#[derive(Args)]
pub struct ArtifactArgs {
    #[command(subcommand)]
    command: ArtifactCommands,
}

#[derive(Subcommand)]
enum ArtifactCommands {
    /// Add a new artifact
    Add {
        /// Artifact content
        content: String,

        /// Optional name slug
        #[arg(long)]
        name: Option<String>,

        /// Content format
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Metadata key=value pairs
        #[arg(long = "meta")]
        metadata: Vec<String>,
    },

    /// Get an artifact by ID
    Get {
        /// Artifact ID
        id: String,
    },

    /// Update an artifact
    Update {
        /// Artifact ID
        id: String,

        /// New content
        #[arg(long)]
        content: Option<String>,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// Metadata key=value pairs to add/update
        #[arg(long = "meta")]
        metadata: Vec<String>,
    },

    /// Remove an artifact
    Remove {
        /// Artifact ID
        id: String,
    },

    /// List artifacts
    List {
        /// Filter by metadata key=value
        #[arg(long = "filter")]
        filters: Vec<String>,

        /// Show only artifacts since timestamp
        #[arg(long)]
        since: Option<String>,

        /// Limit number of results
        #[arg(long)]
        limit: Option<usize>,
    },
}

pub async fn execute(args: ArtifactArgs, artifact_type: ArtifactType) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;

    // Create database and embedding provider
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db, embedding);

    match args.command {
        ArtifactCommands::Add {
            content,
            name,
            format,
            metadata,
        } => {
            let format: ContentFormat = format.parse()?;
            let metadata = parse_metadata(&metadata)?;

            let artifact = service
                .add(artifact_type, content, format, name, metadata)
                .await?;
            println!("Added artifact: {}", artifact.id);
            println!("{}", serde_json::to_string_pretty(&artifact)?);
        },

        ArtifactCommands::Get { id } => {
            if let Some(artifact) = service.get(&id).await? {
                println!("{}", serde_json::to_string_pretty(&artifact)?);
            } else {
                println!("Artifact not found: {}", id);
            }
        },

        ArtifactCommands::Update {
            id,
            content,
            name,
            metadata,
        } => {
            let metadata = if metadata.is_empty() {
                None
            } else {
                Some(parse_metadata(&metadata)?)
            };

            let artifact = service.update(&id, content, name, metadata).await?;
            println!("Updated artifact: {}", artifact.id);
            println!("{}", serde_json::to_string_pretty(&artifact)?);
        },

        ArtifactCommands::Remove { id } => {
            if service.remove(&id).await? {
                println!("Removed artifact: {}", id);
            } else {
                println!("Artifact not found: {}", id);
            }
        },

        ArtifactCommands::List {
            filters,
            since,
            limit,
        } => {
            let metadata_filters = parse_metadata(&filters)?;
            let since_dt = since
                .as_ref()
                .map(|s| {
                    chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
                })
                .transpose()?;

            let search_filters = dna::services::SearchFilters {
                artifact_type: Some(artifact_type),
                metadata: metadata_filters,
                since: since_dt,
                limit,
            };

            let artifacts = service.list(search_filters).await?;
            println!("Found {} artifacts:", artifacts.len());
            for artifact in artifacts {
                println!("  {} - {}", artifact.id, artifact.artifact_type);
            }
        },
    }

    Ok(())
}
