use anyhow::Result;
use clap::Args;
use dna::services::{get_template, list_templates, ConfigService, Template};
use std::path::PathBuf;

#[derive(Args)]
pub struct InitArgs {
    /// Model specification (provider:model)
    #[arg(long)]
    model: Option<String>,

    /// Initialize with a predefined template. Use --list-templates to see available options.
    #[arg(long, value_name = "NAME")]
    template: Option<String>,

    /// List available templates and exit
    #[arg(long)]
    list_templates: bool,

    /// Project root directory
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn print_template_info(template: &Template) {
    println!("  {} - {}", template.name, template.description);
    println!("    Kinds:");
    for kind in template.kinds {
        println!("      {}: {}", kind.slug, kind.description);
    }
    if !template.labels.is_empty() {
        println!("    Labels:");
        for label in template.labels {
            println!("      {}: {}", label.key, label.description);
        }
    }
}

pub async fn execute(args: InitArgs) -> Result<()> {
    // Handle --list-templates
    if args.list_templates {
        println!("Available templates:");
        for name in list_templates() {
            if let Some(template) = get_template(name) {
                print_template_info(template);
            }
        }
        return Ok(());
    }

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

    // Apply template if requested
    let config = if let Some(template_name) = args.template {
        let template = get_template(&template_name).ok_or_else(|| {
            let available = list_templates().join(", ");
            anyhow::anyhow!(
                "Unknown template '{}'. Available templates: {}",
                template_name,
                available
            )
        })?;

        let updated = config_service.init_from_template(template)?;
        println!("  Template '{}' applied:", template.name);
        println!("  Kinds:");
        for kind in &updated.kinds.definitions {
            println!("    {} - {}", kind.slug, kind.description);
        }
        if !updated.labels.definitions.is_empty() {
            println!("  Labels:");
            for label in &updated.labels.definitions {
                println!("    {} - {}", label.key, label.description);
            }
        }
        updated
    } else {
        config
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
