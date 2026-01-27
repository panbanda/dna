use crate::services::ConfigService;
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct McpArgs {
    /// Include only specified tools (comma-separated)
    #[arg(long)]
    include: Option<String>,

    /// Exclude specified tools (comma-separated)
    #[arg(long)]
    exclude: Option<String>,
}

pub async fn execute(args: McpArgs) -> Result<()> {
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

    // Parse tool filters
    let include_tools = args.include.as_ref().map(|s| {
        s.split(',')
            .map(|t| t.trim().to_string())
            .collect::<Vec<_>>()
    });

    let exclude_tools = args.exclude.as_ref().map(|s| {
        s.split(',')
            .map(|t| t.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Start MCP server
    let server = crate::mcp::McpServer::new(db, embedding, include_tools, exclude_tools);
    server.run().await?;

    Ok(())
}
