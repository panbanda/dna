mod artifact;
mod config;
mod init;
mod kind;
mod mcp;
mod render;
mod search;
mod version;

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
    /// Enable verbose output (debug logs)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new DNA project
    Init(init::InitArgs),

    /// Add a new artifact
    Add(artifact::AddArgs),

    /// Get an artifact by ID
    Get(artifact::GetArgs),

    /// Update an existing artifact
    Update(artifact::UpdateArgs),

    /// Remove an artifact
    Remove(artifact::RemoveArgs),

    /// Semantic search across artifacts
    Search(search::SearchArgs),

    /// List artifacts
    List(search::ListArgs),

    /// Show artifact diffs since a date
    Diff(search::DiffArgs),

    /// Render artifacts to filesystem
    Render(render::RenderArgs),

    /// Reindex all artifacts
    Reindex(search::ReindexArgs),

    /// Configuration management
    Config(config::ConfigArgs),

    /// Start MCP server
    Mcp(mcp::McpArgs),

    /// Manage artifact kinds
    Kind(kind::KindArgs),

    /// Compact database and cleanup old versions
    Prune(version::PruneArgs),

    /// List database versions
    Versions(version::VersionsArgs),
}

/// Execute the CLI command
pub async fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => init::execute(args).await,
        Commands::Add(args) => artifact::execute_add(args).await,
        Commands::Get(args) => artifact::execute_get(args).await,
        Commands::Update(args) => artifact::execute_update(args).await,
        Commands::Remove(args) => artifact::execute_remove(args).await,
        Commands::Search(args) => search::execute_search(args).await,
        Commands::List(args) => search::execute_list(args).await,
        Commands::Diff(args) => search::execute_diff(args).await,
        Commands::Render(args) => render::execute(args).await,
        Commands::Reindex(args) => search::execute_reindex(args).await,
        Commands::Config(args) => config::execute(args).await,
        Commands::Mcp(args) => mcp::execute(args).await,
        Commands::Kind(args) => kind::execute(args).await,
        Commands::Prune(args) => version::execute_prune(args).await,
        Commands::Versions(args) => version::execute_versions(args).await,
    }
}
