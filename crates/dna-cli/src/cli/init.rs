use anyhow::Result;
use clap::Args;
use dna::services::ConfigService;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitArgs {
    /// Model specification (provider:model)
    #[arg(long)]
    model: Option<String>,

    /// Project root directory
    #[arg(default_value = ".")]
    path: PathBuf,
}

pub async fn execute(args: InitArgs) -> Result<()> {
    let project_root = args.path;

    // Create .dna directory
    let dna_dir = project_root.join(".dna");
    tokio::fs::create_dir_all(&dna_dir).await?;

    // Initialize config service
    let config_service = ConfigService::new(&project_root);

    // Parse model if provided
    let config = if let Some(model_spec) = args.model {
        let parts: Vec<&str> = model_spec.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Model must be in format provider:model"));
        }

        let mut config = dna::services::ProjectConfig::default();
        config.model.provider = parts[0].to_string();
        config.model.name = parts[1].to_string();
        config_service.save(&config)?;
        config
    } else {
        config_service.init()?
    };

    // Initialize database
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = dna::db::lance::LanceDatabase::new(&storage_uri).await?;
    db.init().await?;

    println!("Initialized DNA project at {}", project_root.display());
    println!("  Provider: {}", config.model.provider);
    println!("  Model: {}", config.model.name);
    println!("  Storage: {}", storage_uri);

    Ok(())
}
