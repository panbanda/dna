use anyhow::Result;
use clap::Args;
use dna::services::{ArtifactService, ConfigService, SearchFilters};
use std::path::PathBuf;

#[derive(Args)]
pub struct RenderArgs {
    /// Group artifacts by metadata keys (comma-separated)
    #[arg(long)]
    by: Option<String>,

    /// Output directory
    #[arg(long, default_value = "dna")]
    output: PathBuf,
}

pub async fn execute(args: RenderArgs) -> Result<()> {
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
    let render_service = dna::render::RenderService::new(args.output.clone());

    // Get all artifacts
    let artifacts = service.list(SearchFilters::default()).await?;

    // Parse grouping keys
    let group_by = args
        .by
        .as_ref()
        .map(|s| {
            s.split(',')
                .map(|k| k.trim().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Render artifacts
    render_service.render_all(&artifacts, &group_by).await?;

    println!(
        "Rendered {} artifacts to {}",
        artifacts.len(),
        args.output.display()
    );

    Ok(())
}
