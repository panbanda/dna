use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use crate::services::{ArtifactService, ArtifactType, ContentFormat, SearchFilters, SearchService};
use chrono::{DateTime, Utc};
use rmcp::model::{CallToolResult, Content, PaginatedRequestParams};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// DNA MCP tool handler using rmcp SDK
pub struct DnaToolHandler {
    artifact_service: Arc<ArtifactService>,
    search_service: Arc<SearchService>,
    include_tools: Option<Vec<String>>,
    exclude_tools: Option<Vec<String>>,
}

impl Clone for DnaToolHandler {
    fn clone(&self) -> Self {
        Self {
            artifact_service: Arc::clone(&self.artifact_service),
            search_service: Arc::clone(&self.search_service),
            include_tools: self.include_tools.clone(),
            exclude_tools: self.exclude_tools.clone(),
        }
    }
}

impl DnaToolHandler {
    /// Create a new DNA tool handler
    pub fn new(
        db: Arc<dyn Database>,
        embedding: Arc<dyn EmbeddingProvider>,
        include_tools: Option<Vec<String>>,
        exclude_tools: Option<Vec<String>>,
    ) -> Self {
        let artifact_service = Arc::new(ArtifactService::new(db.clone(), embedding.clone()));
        let search_service = Arc::new(SearchService::new(db, embedding));

        Self {
            artifact_service,
            search_service,
            include_tools,
            exclude_tools,
        }
    }

    /// Check if a tool should be available based on filters
    fn is_tool_available(&self, tool_name: &str) -> bool {
        if let Some(ref include) = self.include_tools {
            include.iter().any(|t| tool_name.contains(t))
        } else if let Some(ref exclude) = self.exclude_tools {
            !exclude.iter().any(|t| tool_name.contains(t))
        } else {
            true
        }
    }

    /// Semantic search for truth artifacts
    async fn dna_search(&self, request: SearchRequest) -> Result<CallToolResult, ErrorData> {
        let filters = SearchFilters {
            artifact_type: request.artifact_type,
            limit: request.limit,
            ..Default::default()
        };

        let results = self
            .search_service
            .search(&request.query, filters)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let content = serde_json::to_string_pretty(&results)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    /// Get artifact by ID
    async fn dna_get(&self, request: GetRequest) -> Result<CallToolResult, ErrorData> {
        let artifact = self
            .artifact_service
            .get(&request.id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let is_error = artifact.is_none();
        let content = match artifact {
            Some(a) => serde_json::to_string_pretty(&a)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None))?,
            None => format!("Artifact not found: {}", request.id),
        };

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(is_error),
            meta: None,
            structured_content: None,
        })
    }

    /// List artifacts by type/metadata
    async fn dna_list(&self, request: ListRequest) -> Result<CallToolResult, ErrorData> {
        let filters = SearchFilters {
            artifact_type: request.artifact_type,
            after: request.after,
            before: request.before,
            limit: request.limit,
            ..Default::default()
        };

        let artifacts = self
            .artifact_service
            .list(filters)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let content = serde_json::to_string_pretty(&artifacts)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    /// Artifacts modified in time range
    async fn dna_changes(&self, request: ChangesRequest) -> Result<CallToolResult, ErrorData> {
        let filters = SearchFilters {
            after: request.after,
            before: request.before,
            ..Default::default()
        };

        let artifacts = self
            .artifact_service
            .list(filters)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let content = serde_json::to_string_pretty(&artifacts)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    /// Add new artifact
    async fn dna_add(&self, request: AddRequest) -> Result<CallToolResult, ErrorData> {
        let artifact = self
            .artifact_service
            .add(
                request.artifact_type,
                request.content,
                request.format,
                request.name,
                request.metadata,
            )
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let content = serde_json::to_string_pretty(&artifact)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    /// Modify existing artifact
    async fn dna_update(&self, request: UpdateRequest) -> Result<CallToolResult, ErrorData> {
        let artifact = self
            .artifact_service
            .update(&request.id, request.content, request.name, request.metadata)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let content = serde_json::to_string_pretty(&artifact)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    /// Delete artifact
    async fn dna_remove(&self, request: RemoveRequest) -> Result<CallToolResult, ErrorData> {
        let removed = self
            .artifact_service
            .remove(&request.id)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let result = serde_json::json!({ "removed": removed });
        let content = serde_json::to_string_pretty(&result)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult {
            content: vec![Content::text(content)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

impl ServerHandler for DnaToolHandler {
    fn get_info(&self) -> rmcp::model::InitializeResult {
        rmcp::model::InitializeResult {
            protocol_version: Default::default(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "dna-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: None,
        }
    }

    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, ErrorData> {
        use rmcp::model::Tool;
        use std::sync::Arc;

        // Helper macro to convert schema to JSON
        macro_rules! schema_to_json {
            ($type:ty) => {{
                let schema = schemars::schema_for!($type);
                let value = serde_json::to_value(schema).unwrap_or_default();
                if let serde_json::Value::Object(map) = value {
                    Arc::new(map)
                } else {
                    Arc::new(serde_json::Map::new())
                }
            }};
        }

        // Get all tools
        let all_tools = vec![
            Tool {
                name: "dna_search".into(),
                description: Some("Semantic search for truth artifacts".into()),
                input_schema: schema_to_json!(SearchRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "dna_get".into(),
                description: Some("Get artifact by ID".into()),
                input_schema: schema_to_json!(GetRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "dna_list".into(),
                description: Some("List artifacts by type/metadata".into()),
                input_schema: schema_to_json!(ListRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "dna_changes".into(),
                description: Some("Artifacts modified in time range".into()),
                input_schema: schema_to_json!(ChangesRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "dna_add".into(),
                description: Some("Add new artifact".into()),
                input_schema: schema_to_json!(AddRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "dna_update".into(),
                description: Some("Modify existing artifact".into()),
                input_schema: schema_to_json!(UpdateRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
            Tool {
                name: "dna_remove".into(),
                description: Some("Delete artifact".into()),
                input_schema: schema_to_json!(RemoveRequest),
                title: None,
                output_schema: None,
                annotations: None,
                icons: None,
                meta: None,
            },
        ];

        // Apply filters
        let tools: Vec<Tool> = all_tools
            .into_iter()
            .filter(|tool| self.is_tool_available(&tool.name))
            .collect();

        Ok(rmcp::model::ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = &params.name;
        let arguments = serde_json::Value::Object(params.arguments.unwrap_or_default());

        if !self.is_tool_available(name) {
            return Err(ErrorData::new(
                rmcp::model::ErrorCode(-32601),
                format!("Tool not available: {}", name),
                None,
            ));
        }

        match name.as_ref() {
            "dna_search" => {
                let request: SearchRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_search(request).await
            },
            "dna_get" => {
                let request: GetRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_get(request).await
            },
            "dna_list" => {
                let request: ListRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_list(request).await
            },
            "dna_changes" => {
                let request: ChangesRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_changes(request).await
            },
            "dna_add" => {
                let request: AddRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_add(request).await
            },
            "dna_update" => {
                let request: UpdateRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_update(request).await
            },
            "dna_remove" => {
                let request: RemoveRequest = serde_json::from_value(arguments)
                    .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
                self.dna_remove(request).await
            },
            _ => Err(ErrorData::new(
                rmcp::model::ErrorCode(-32601),
                format!("Unknown tool: {}", name),
                None,
            )),
        }
    }
}

// Request types

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchRequest {
    query: String,
    #[serde(rename = "type")]
    artifact_type: Option<ArtifactType>,
    #[serde(default = "default_limit")]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetRequest {
    id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListRequest {
    #[serde(rename = "type")]
    artifact_type: Option<ArtifactType>,
    #[serde(default)]
    after: Option<DateTime<Utc>>,
    #[serde(default)]
    before: Option<DateTime<Utc>>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ChangesRequest {
    after: Option<DateTime<Utc>>,
    before: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddRequest {
    #[serde(rename = "type")]
    artifact_type: ArtifactType,
    content: String,
    #[serde(default = "default_format")]
    format: ContentFormat,
    name: Option<String>,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateRequest {
    id: String,
    content: Option<String>,
    name: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RemoveRequest {
    id: String,
}

fn default_limit() -> Option<usize> {
    Some(10)
}

fn default_format() -> ContentFormat {
    ContentFormat::Markdown
}
