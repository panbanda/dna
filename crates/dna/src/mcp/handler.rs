use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use crate::services::{ArtifactService, ContentFormat, SearchFilters, SearchService};
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
            kind: request.kind,
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

    /// List artifacts by kind/metadata
    async fn dna_list(&self, request: ListRequest) -> Result<CallToolResult, ErrorData> {
        let filters = SearchFilters {
            kind: request.kind,
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
                request.kind,
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
            .update(
                &request.id,
                request.content,
                request.name,
                request.kind,
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
                description: Some("List artifacts by kind/metadata".into()),
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
    kind: Option<String>,
    #[serde(default = "default_limit")]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetRequest {
    id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListRequest {
    kind: Option<String>,
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
    kind: String,
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
    kind: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Artifact, SearchResult};
    use anyhow::Result;
    use std::sync::Mutex;

    struct TestEmbedding;

    #[async_trait::async_trait]
    impl EmbeddingProvider for TestEmbedding {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.1, 0.2, 0.3])
        }

        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
        }

        fn model_id(&self) -> &str {
            "test-model"
        }

        fn dimensions(&self) -> usize {
            3
        }
    }

    struct TestDatabase {
        artifacts: Mutex<HashMap<String, Artifact>>,
    }

    impl TestDatabase {
        fn new() -> Self {
            Self {
                artifacts: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl Database for TestDatabase {
        async fn insert(&self, artifact: &Artifact) -> Result<()> {
            self.artifacts
                .lock()
                .unwrap()
                .insert(artifact.id.clone(), artifact.clone());
            Ok(())
        }

        async fn get(&self, id: &str) -> Result<Option<Artifact>> {
            Ok(self.artifacts.lock().unwrap().get(id).cloned())
        }

        async fn update(&self, artifact: &Artifact) -> Result<()> {
            self.artifacts
                .lock()
                .unwrap()
                .insert(artifact.id.clone(), artifact.clone());
            Ok(())
        }

        async fn delete(&self, id: &str) -> Result<bool> {
            Ok(self.artifacts.lock().unwrap().remove(id).is_some())
        }

        async fn list(&self, _filters: SearchFilters) -> Result<Vec<Artifact>> {
            Ok(self.artifacts.lock().unwrap().values().cloned().collect())
        }

        async fn search(
            &self,
            _query_embedding: &[f32],
            _filters: SearchFilters,
        ) -> Result<Vec<SearchResult>> {
            let artifacts: Vec<_> = self.artifacts.lock().unwrap().values().cloned().collect();
            Ok(artifacts
                .into_iter()
                .map(|a| SearchResult {
                    artifact: a,
                    score: 0.9,
                })
                .collect())
        }
    }

    fn test_handler() -> DnaToolHandler {
        let db: Arc<dyn Database> = Arc::new(TestDatabase::new());
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);
        DnaToolHandler::new(db, embedding, None, None)
    }

    #[test]
    fn is_tool_available_no_filters() {
        let handler = test_handler();
        assert!(handler.is_tool_available("dna_search"));
        assert!(handler.is_tool_available("dna_add"));
    }

    #[test]
    fn is_tool_available_include_filter() {
        let db: Arc<dyn Database> = Arc::new(TestDatabase::new());
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);
        let handler = DnaToolHandler::new(db, embedding, Some(vec!["search".to_string()]), None);
        assert!(handler.is_tool_available("dna_search"));
        assert!(!handler.is_tool_available("dna_add"));
    }

    #[test]
    fn is_tool_available_exclude_filter() {
        let db: Arc<dyn Database> = Arc::new(TestDatabase::new());
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);
        let handler = DnaToolHandler::new(db, embedding, None, Some(vec!["remove".to_string()]));
        assert!(handler.is_tool_available("dna_search"));
        assert!(!handler.is_tool_available("dna_remove"));
    }

    #[tokio::test]
    async fn dna_add_creates_artifact() {
        let handler = test_handler();
        let request = AddRequest {
            kind: "intent".to_string(),
            content: "test content".to_string(),
            format: ContentFormat::Markdown,
            name: Some("test".to_string()),
            metadata: HashMap::new(),
        };

        let result = handler.dna_add(request).await.unwrap();
        assert_eq!(result.is_error, Some(false));
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn dna_get_returns_not_found() {
        let handler = test_handler();
        let request = GetRequest {
            id: "nonexistent".to_string(),
        };

        let result = handler.dna_get(request).await.unwrap();
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn dna_get_returns_artifact() {
        let handler = test_handler();

        // Add first
        let add_request = AddRequest {
            kind: "intent".to_string(),
            content: "get me".to_string(),
            format: ContentFormat::Markdown,
            name: None,
            metadata: HashMap::new(),
        };
        let add_result = handler.dna_add(add_request).await.unwrap();
        let added: serde_json::Value =
            serde_json::from_str(&add_result.content[0].as_text().unwrap().text).unwrap();
        let id = added["id"].as_str().unwrap().to_string();

        // Get it
        let get_request = GetRequest { id };
        let result = handler.dna_get(get_request).await.unwrap();
        assert_eq!(result.is_error, Some(false));
    }

    #[tokio::test]
    async fn dna_list_returns_artifacts() {
        let handler = test_handler();

        handler
            .dna_add(AddRequest {
                kind: "intent".to_string(),
                content: "one".to_string(),
                format: ContentFormat::Markdown,
                name: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        let result = handler
            .dna_list(ListRequest {
                kind: None,
                after: None,
                before: None,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        let text = &result.content[0].as_text().unwrap().text;
        let parsed: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[tokio::test]
    async fn dna_search_returns_results() {
        let handler = test_handler();

        handler
            .dna_add(AddRequest {
                kind: "intent".to_string(),
                content: "searchable".to_string(),
                format: ContentFormat::Markdown,
                name: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        let result = handler
            .dna_search(SearchRequest {
                query: "searchable".to_string(),
                kind: None,
                limit: Some(10),
            })
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
    }

    #[tokio::test]
    async fn dna_remove_nonexistent() {
        let handler = test_handler();
        let result = handler
            .dna_remove(RemoveRequest {
                id: "nonexistent".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("false"));
    }

    #[tokio::test]
    async fn dna_update_existing() {
        let handler = test_handler();

        let add_result = handler
            .dna_add(AddRequest {
                kind: "intent".to_string(),
                content: "original".to_string(),
                format: ContentFormat::Markdown,
                name: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();
        let added: serde_json::Value =
            serde_json::from_str(&add_result.content[0].as_text().unwrap().text).unwrap();
        let id = added["id"].as_str().unwrap().to_string();

        let result = handler
            .dna_update(UpdateRequest {
                id,
                content: Some("updated".to_string()),
                name: None,
                kind: None,
                metadata: None,
            })
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
        let text = &result.content[0].as_text().unwrap().text;
        assert!(text.contains("updated"));
    }

    #[tokio::test]
    async fn dna_changes_returns_artifacts() {
        let handler = test_handler();

        handler
            .dna_add(AddRequest {
                kind: "intent".to_string(),
                content: "changed".to_string(),
                format: ContentFormat::Markdown,
                name: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        let result = handler
            .dna_changes(ChangesRequest {
                after: None,
                before: None,
            })
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn default_limit_is_10() {
        assert_eq!(default_limit(), Some(10));
    }

    #[test]
    fn default_format_is_markdown() {
        assert_eq!(default_format(), ContentFormat::Markdown);
    }

    #[test]
    fn get_info_returns_server_info() {
        let handler = test_handler();
        let info = handler.get_info();
        assert_eq!(info.server_info.name, "dna-server");
        assert!(info.capabilities.tools.is_some());
    }
}
