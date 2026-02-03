mod cli;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI first to get verbose flag
    let cli = cli::Cli::parse();

    // Initialize logging based on verbose flag
    // Default to warn level (quiet), info on --verbose, or respect RUST_LOG env var
    let default_level = if cli.verbose { "info" } else { "warn" };
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| default_level.into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Execute command
    cli::execute(cli).await
}
