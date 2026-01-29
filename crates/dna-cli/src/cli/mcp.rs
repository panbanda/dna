use anyhow::Result;
use clap::Args;
use dna::mcp::DnaToolHandler;
use dna::services::ConfigService;
use rmcp::ServiceExt;
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
    let storage_uri = config_service.resolve_storage_uri(&project_root)?;
    let db = std::sync::Arc::new(dna::db::lance::LanceDatabase::new(&storage_uri).await?);
    let embedding = dna::embedding::create_provider(&config.model).await?;

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

    // Log to stderr for stdio servers
    eprintln!("Starting DNA MCP server...");

    // Create handler and start server with stdio transport
    let handler = DnaToolHandler::new(db, embedding, include_tools, exclude_tools);
    let service = handler.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
