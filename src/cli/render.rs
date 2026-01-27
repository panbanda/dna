use crate::services::{ArtifactService, ConfigService, SearchFilters};
use anyhow::Result;
use clap::Args;
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
    let db_path = project_root.join(".dna").join("db").join("artifacts.lance");
    let db = std::sync::Arc::new(crate::db::lance::LanceDatabase::new(&db_path).await?);
    let embedding = crate::embedding::create_provider(&config.model).await?;

    let service = ArtifactService::new(db, embedding);
    let render_service = crate::render::RenderService::new(args.output.clone());

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
