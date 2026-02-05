use anyhow::Result;
use clap::{Args, Subcommand};
use dna::services::{slugify_kind, ConfigService, KindValidationError};
use std::path::PathBuf;

#[derive(Args)]
pub struct LabelArgs {
    #[command(subcommand)]
    pub command: LabelCommands,
}

#[derive(Subcommand)]
pub enum LabelCommands {
    /// Register a new label key
    Add(LabelAddArgs),

    /// List all registered labels
    List,

    /// Show details of a registered label
    Show(LabelShowArgs),

    /// Remove a registered label
    Remove(LabelRemoveArgs),
}

#[derive(Args)]
pub struct LabelAddArgs {
    /// Label key (will be slugified)
    pub key: String,

    /// Description of what this label represents.
    /// Helps LLMs understand when to use this label.
    pub description: String,
}

#[derive(Args)]
pub struct LabelShowArgs {
    /// Label key
    pub key: String,
}

#[derive(Args)]
pub struct LabelRemoveArgs {
    /// Label key
    pub key: String,

    /// Force removal without warning
    #[arg(long, short)]
    pub force: bool,
}

pub async fn execute(args: LabelArgs) -> Result<()> {
    match args.command {
        LabelCommands::Add(add_args) => execute_add(add_args).await,
        LabelCommands::List => execute_list().await,
        LabelCommands::Show(show_args) => execute_show(show_args).await,
        LabelCommands::Remove(remove_args) => execute_remove(remove_args).await,
    }
}

async fn execute_add(args: LabelAddArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let key = slugify_kind(&args.key);
    let description = args.description;

    let added = match config_service.add_label(&key, &description) {
        Ok(added) => added,
        Err(e) => {
            if let Some(validation_error) = e.downcast_ref::<KindValidationError>() {
                return Err(anyhow::anyhow!(
                    "{}",
                    format_validation_error(validation_error)
                ));
            }
            return Err(e);
        },
    };
    if added {
        println!("Added label: {}", key);
        println!("  Description: {}", description);
        println!();
        println!("You can now use:");
        println!("  dna add <kind> <content> --label {}=<value>", key);
    } else {
        println!("Label '{}' already exists.", key);
    }

    Ok(())
}

async fn execute_list() -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let labels = &config.labels.definitions;

    if labels.is_empty() {
        println!("No labels registered. Use 'dna label add <key> <description>' or 'dna init --template <name>'.");
        return Ok(());
    }

    println!("Registered labels ({}):", labels.len());
    for label in labels {
        println!("  {} - {}", label.key, label.description);
    }

    Ok(())
}

async fn execute_show(args: LabelShowArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let key = slugify_kind(&args.key);

    match config.labels.get(&key) {
        Some(label) => {
            println!("Label: {}", label.key);
            println!("Description: {}", label.description);
            println!();
            println!("Usage:");
            println!("  dna add <kind> <content> --label {}=<value>", key);
            println!("  dna list --label {}=<value>", key);
        },
        None => {
            println!("Label '{}' not found.", key);
        },
    }

    Ok(())
}

async fn execute_remove(args: LabelRemoveArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let key = slugify_kind(&args.key);

    let config = config_service.load()?;
    if config.labels.get(&key).is_none() {
        println!("Label '{}' not found.", key);
        return Ok(());
    }

    if !args.force {
        eprintln!(
            "Warning: Removing label '{}' will not update existing artifacts.",
            key
        );
        eprintln!("         Artifacts using this label key will still retain their values,");
        eprintln!("         but new artifacts will not be able to use this key.");
        eprintln!();
        eprintln!("To proceed, re-run with --force or -f");
        return Ok(());
    }

    let removed = config_service.remove_label(&key)?;
    if removed {
        println!("Removed label: {}", key);
    }

    Ok(())
}

fn format_validation_error(error: &KindValidationError) -> String {
    match error {
        KindValidationError::Empty => "Label key cannot be empty".to_string(),
        KindValidationError::TooShort { min, actual } => {
            format!(
                "Label key is too short (minimum {} characters, got {})",
                min, actual
            )
        },
        KindValidationError::TooLong { max, actual } => {
            format!(
                "Label key is too long (maximum {} characters, got {})",
                max, actual
            )
        },
        KindValidationError::Reserved { slug } => {
            format!("Label key '{}' is reserved", slug)
        },
        KindValidationError::InvalidChars { slug } => {
            format!(
                "Label key '{}' contains invalid characters. Use only lowercase letters, numbers, and hyphens.",
                slug
            )
        },
    }
}
