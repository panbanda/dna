use anyhow::Result;
use clap::{Args, Subcommand};
use dna::services::{slugify_kind, ConfigService, KindValidationError};
use std::path::PathBuf;

#[derive(Args)]
pub struct KindArgs {
    #[command(subcommand)]
    pub command: KindCommands,
}

#[derive(Subcommand)]
pub enum KindCommands {
    /// Register a new artifact kind
    Add(KindAddArgs),

    /// List all registered kinds
    List,

    /// Show details of a registered kind
    Show(KindShowArgs),

    /// Remove a registered kind
    Remove(KindRemoveArgs),
}

#[derive(Args)]
pub struct KindAddArgs {
    /// Kind name (will be slugified)
    pub name: String,

    /// Description of what artifacts of this kind contain.
    /// Helps LLMs understand when to use this kind.
    pub description: String,
}

#[derive(Args)]
pub struct KindShowArgs {
    /// Kind slug
    pub slug: String,
}

#[derive(Args)]
pub struct KindRemoveArgs {
    /// Kind slug
    pub slug: String,

    /// Force removal without warning
    #[arg(long, short)]
    pub force: bool,
}

pub async fn execute(args: KindArgs) -> Result<()> {
    match args.command {
        KindCommands::Add(add_args) => execute_add(add_args).await,
        KindCommands::List => execute_list().await,
        KindCommands::Show(show_args) => execute_show(show_args).await,
        KindCommands::Remove(remove_args) => execute_remove(remove_args).await,
    }
}

async fn execute_add(args: KindAddArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let slug = slugify_kind(&args.name);
    let description = args.description;

    let added = match config_service.add_kind(&slug, &description) {
        Ok(added) => added,
        Err(e) => {
            // Check if it's a validation error and provide a user-friendly message
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
        println!("Added kind: {}", slug);
        println!("  Description: {}", description);
        println!();
        println!("You can now use:");
        println!(
            "  dna add {} <content>        # add a {} artifact",
            slug, slug
        );
        println!(
            "  dna search <query> --kind {}  # search {} artifacts",
            slug, slug
        );
        println!(
            "  dna list --kind {}            # list {} artifacts",
            slug, slug
        );
        println!();
        println!("API endpoint:  POST /api/v1/kinds/{}/artifacts", slug);
        println!(
            "MCP tool:      dna_{}_search, dna_{}_add",
            slug.replace('-', "_"),
            slug.replace('-', "_")
        );
    } else {
        println!("Kind '{}' already exists.", slug);
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
    let kinds = &config.kinds.definitions;

    if kinds.is_empty() {
        println!("No kinds registered. Use 'dna kind add <name>' or 'dna init --intent-flow'.");
        return Ok(());
    }

    println!("Registered kinds ({}):", kinds.len());
    for kind in kinds {
        println!("  {} - {}", kind.slug, kind.description);
    }

    Ok(())
}

async fn execute_show(args: KindShowArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let config = config_service.load()?;
    let slug = slugify_kind(&args.slug);

    match config.kinds.get(&slug) {
        Some(kind) => {
            println!("Kind: {}", kind.slug);
            println!("Description: {}", kind.description);
            println!();
            let tool_prefix = slug.replace('-', "_");
            println!("CLI:");
            println!("  dna add {} <content>", slug);
            println!("  dna search <query> --kind {}", slug);
            println!("  dna list --kind {}", slug);
            println!();
            println!("API:");
            println!("  GET    /api/v1/kinds/{}/artifacts", slug);
            println!("  POST   /api/v1/kinds/{}/artifacts", slug);
            println!("  POST   /api/v1/kinds/{}/search", slug);
            println!();
            println!("MCP tools:");
            println!("  dna_{}_search", tool_prefix);
            println!("  dna_{}_add", tool_prefix);
            println!("  dna_{}_list", tool_prefix);
        },
        None => {
            println!("Kind '{}' not found.", slug);
        },
    }

    Ok(())
}

async fn execute_remove(args: KindRemoveArgs) -> Result<()> {
    let project_root = PathBuf::from(".");
    let config_service = ConfigService::new(&project_root);

    if !config_service.exists() {
        return Err(anyhow::anyhow!(
            "DNA not initialized. Run 'dna init' first."
        ));
    }

    let slug = slugify_kind(&args.slug);

    // Check if kind exists first
    let config = config_service.load()?;
    if config.kinds.get(&slug).is_none() {
        println!("Kind '{}' not found.", slug);
        return Ok(());
    }

    // Warn about potential orphaned artifacts
    if !args.force {
        eprintln!(
            "Warning: Removing kind '{}' will not delete existing artifacts.",
            slug
        );
        eprintln!("         Artifacts of this kind will become orphaned and may not");
        eprintln!("         appear in kind-filtered searches or API endpoints.");
        eprintln!();
        eprintln!("To proceed, re-run with --force or -f");
        return Ok(());
    }

    let removed = config_service.remove_kind(&slug)?;
    if removed {
        println!("Removed kind: {}", slug);
    }

    Ok(())
}

fn format_validation_error(error: &KindValidationError) -> String {
    match error {
        KindValidationError::Empty => "Kind slug cannot be empty".to_string(),
        KindValidationError::TooShort { min, actual } => {
            format!(
                "Kind slug '{}' is too short (minimum {} characters, got {})",
                "", min, actual
            )
        },
        KindValidationError::TooLong { max, actual } => {
            format!(
                "Kind slug is too long (maximum {} characters, got {})",
                max, actual
            )
        },
        KindValidationError::Reserved { slug } => {
            format!(
                "Kind slug '{}' is reserved. Reserved slugs: all, any, artifact, artifacts, config, default, kind, kinds, none, search, system",
                slug
            )
        },
        KindValidationError::InvalidChars { slug } => {
            format!(
                "Kind slug '{}' contains invalid characters. Use only lowercase letters, numbers, and hyphens.",
                slug
            )
        },
    }
}
