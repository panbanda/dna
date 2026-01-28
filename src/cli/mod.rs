mod artifact;
mod config;
mod init;
mod mcp;
mod render;
mod search;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;

/// Parse metadata key=value pairs from command line arguments
pub fn parse_metadata(pairs: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in pairs {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid metadata format: {}", pair));
        }
        map.insert(parts[0].to_string(), parts[1].to_string());
    }
    Ok(map)
}

#[derive(Parser)]
#[command(name = "dna")]
#[command(about = "Truth artifact management CLI with vector search", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new DNA project
    Init(init::InitArgs),

    /// Manage intent artifacts
    Intent(artifact::ArtifactArgs),

    /// Manage invariant artifacts
    Invariant(artifact::ArtifactArgs),

    /// Manage contract artifacts
    Contract(artifact::ArtifactArgs),

    /// Manage algorithm artifacts
    Algorithm(artifact::ArtifactArgs),

    /// Manage evaluation artifacts
    Evaluation(artifact::ArtifactArgs),

    /// Manage pace artifacts
    Pace(artifact::ArtifactArgs),

    /// Manage monitor artifacts
    Monitor(artifact::ArtifactArgs),

    /// Semantic search across artifacts
    Search(search::SearchArgs),

    /// List artifacts
    List(search::ListArgs),

    /// Show changes since a timestamp or git ref
    Changes(search::ChangesArgs),

    /// Render artifacts to filesystem
    Render(render::RenderArgs),

    /// Reindex all artifacts
    Reindex(search::ReindexArgs),

    /// Configuration management
    Config(config::ConfigArgs),

    /// Start MCP server
    Mcp(mcp::McpArgs),
}

/// Execute the CLI command
pub async fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => init::execute(args).await,
        Commands::Intent(args) => {
            artifact::execute(args, crate::services::ArtifactType::Intent).await
        },
        Commands::Invariant(args) => {
            artifact::execute(args, crate::services::ArtifactType::Invariant).await
        },
        Commands::Contract(args) => {
            artifact::execute(args, crate::services::ArtifactType::Contract).await
        },
        Commands::Algorithm(args) => {
            artifact::execute(args, crate::services::ArtifactType::Algorithm).await
        },
        Commands::Evaluation(args) => {
            artifact::execute(args, crate::services::ArtifactType::Evaluation).await
        },
        Commands::Pace(args) => artifact::execute(args, crate::services::ArtifactType::Pace).await,
        Commands::Monitor(args) => {
            artifact::execute(args, crate::services::ArtifactType::Monitor).await
        },
        Commands::Search(args) => search::execute_search(args).await,
        Commands::List(args) => search::execute_list(args).await,
        Commands::Changes(args) => search::execute_changes(args).await,
        Commands::Render(args) => render::execute(args).await,
        Commands::Reindex(args) => search::execute_reindex(args).await,
        Commands::Config(args) => config::execute(args).await,
        Commands::Mcp(args) => mcp::execute(args).await,
    }
}
