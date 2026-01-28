use anyhow::Result;
use clap::{Args, Subcommand};
use dna::services::ConfigService;
use std::path::PathBuf;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Configure embedding model
    Model {
        /// Model specification (provider:model) or empty to show current
        spec: Option<String>,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

pub async fn execute(args: ConfigArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    match args.command {
        ConfigCommands::Model { spec } => {
            if let Some(model_spec) = spec {
                let parts: Vec<&str> = model_spec.split(':').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!("Model must be in format provider:model"));
                }

                config_service.update_model(parts[0].to_string(), parts[1].to_string())?;

                println!("Updated model configuration:");
                println!("  Provider: {}", parts[0]);
                println!("  Model: {}", parts[1]);
                println!(
                    "\nNote: Run 'dna reindex' to re-embed existing artifacts with the new model."
                );
            } else {
                let config = config_service.load()?;
                println!("Current model configuration:");
                println!("  Provider: {}", config.model.provider);
                println!("  Model: {}", config.model.name);
            }
        },

        ConfigCommands::Get { key } => {
            let value = config_service.get(&key)?;
            println!("{}", value);
        },

        ConfigCommands::Set { key, value } => {
            config_service.set(&key, value.clone())?;
            println!("Set {} = {}", key, value);
        },
    }

    Ok(())
}
