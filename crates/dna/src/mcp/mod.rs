pub mod handler;

pub use handler::DnaToolHandler;

use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use crate::services::{ArtifactService, ArtifactType, ContentFormat, SearchFilters, SearchService};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::Arc;

/// MCP Server for DNA
pub struct McpServer {
    artifact_service: ArtifactService,
    search_service: SearchService,
    include_tools: Option<Vec<String>>,
    exclude_tools: Option<Vec<String>>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(
        db: Arc<dyn Database>,
        embedding: Arc<dyn EmbeddingProvider>,
        include_tools: Option<Vec<String>>,
        exclude_tools: Option<Vec<String>>,
    ) -> Self {
        let artifact_service = ArtifactService::new(db.clone(), embedding.clone());
        let search_service = SearchService::new(db, embedding);

        Self {
            artifact_service,
            search_service,
            include_tools,
            exclude_tools,
        }
    }

    /// Run the MCP server
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Starting DNA MCP server...");

        // Get available tools
        let tools = self.get_available_tools();
        tracing::info!(
            "Available tools: {:?}",
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );

        // MCP server loop (stdio-based)
        self.stdio_loop(tools).await?;

        Ok(())
    }

    /// Get list of available tools based on filters
    fn get_available_tools(&self) -> Vec<McpTool> {
        let all_tools = vec![
            McpTool {
                name: "dna_search".to_string(),
                description: "Semantic search for truth artifacts".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "type": {"type": "string", "enum": ["intent", "invariant", "contract", "algorithm", "evaluation", "pace", "monitor"]},
                        "filter": {"type": "object"},
                        "limit": {"type": "integer", "default": 10}
                    },
                    "required": ["query"]
                }),
            },
            McpTool {
                name: "dna_get".to_string(),
                description: "Get artifact by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"}
                    },
                    "required": ["id"]
                }),
            },
            McpTool {
                name: "dna_list".to_string(),
                description: "List artifacts by type/metadata".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "type": {"type": "string"},
                        "filter": {"type": "object"},
                        "after": {"type": "string"},
                        "before": {"type": "string"},
                        "limit": {"type": "integer"}
                    }
                }),
            },
            McpTool {
                name: "dna_changes".to_string(),
                description: "Artifacts modified in time range".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "after": {"type": "string"},
                        "before": {"type": "string"}
                    }
                }),
            },
            McpTool {
                name: "dna_add".to_string(),
                description: "Add new artifact".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "type": {"type": "string", "enum": ["intent", "invariant", "contract", "algorithm", "evaluation", "pace", "monitor"]},
                        "content": {"type": "string"},
                        "format": {"type": "string", "default": "markdown"},
                        "name": {"type": "string"},
                        "metadata": {"type": "object"}
                    },
                    "required": ["type", "content"]
                }),
            },
            McpTool {
                name: "dna_update".to_string(),
                description: "Modify existing artifact".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "content": {"type": "string"},
                        "name": {"type": "string"},
                        "metadata": {"type": "object"}
                    },
                    "required": ["id"]
                }),
            },
            McpTool {
                name: "dna_remove".to_string(),
                description: "Delete artifact".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"}
                    },
                    "required": ["id"]
                }),
            },
        ];

        // Apply filters
        all_tools
            .into_iter()
            .filter(|tool| {
                if let Some(ref include) = self.include_tools {
                    include.iter().any(|t| tool.name.contains(t))
                } else if let Some(ref exclude) = self.exclude_tools {
                    !exclude.iter().any(|t| tool.name.contains(t))
                } else {
                    true
                }
            })
            .collect()
    }

    /// Run stdio-based MCP server loop
    async fn stdio_loop(&self, tools: Vec<McpTool>) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        // Send initial capabilities
        let capabilities = McpCapabilities {
            tools: tools.clone(),
        };
        let init_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "capabilities": capabilities
            }
        });
        writeln!(stdout, "{}", serde_json::to_string(&init_response)?)?;
        stdout.flush()?;

        // Process incoming requests
        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<McpRequest>(&line) {
                Ok(request) => self.handle_request(request, &tools).await,
                Err(e) => McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("Parse error: {e}"),
                    }),
                },
            };

            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }

        Ok(())
    }

    /// Handle a single MCP request
    async fn handle_request(&self, request: McpRequest, tools: &[McpTool]) -> McpResponse {
        let result = match request.method.as_str() {
            "tools/list" => Ok(serde_json::to_value(tools).unwrap_or_default()),
            "tools/call" => self.handle_tool_call(request.params).await,
            _ => Err(McpError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        };

        match result {
            Ok(value) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    /// Handle a tool call
    async fn handle_tool_call(
        &self,
        params: Option<serde_json::Value>,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let params = params.ok_or_else(|| McpError {
            code: -32602,
            message: "Missing params".to_string(),
        })?;

        let tool_name = params["name"].as_str().ok_or_else(|| McpError {
            code: -32602,
            message: "Missing tool name".to_string(),
        })?;

        let arguments = params.get("arguments").cloned().unwrap_or_default();

        match tool_name {
            "dna_search" => self.handle_search(arguments).await,
            "dna_get" => self.handle_get(arguments).await,
            "dna_list" => self.handle_list(arguments).await,
            "dna_add" => self.handle_add(arguments).await,
            "dna_update" => self.handle_update(arguments).await,
            "dna_remove" => self.handle_remove(arguments).await,
            "dna_changes" => self.handle_changes(arguments).await,
            _ => Err(McpError {
                code: -32602,
                message: format!("Unknown tool: {tool_name}"),
            }),
        }
    }

    async fn handle_search(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let query = args["query"].as_str().ok_or_else(|| McpError {
            code: -32602,
            message: "Missing query parameter".to_string(),
        })?;

        let limit = args["limit"].as_u64().map(|l| l as usize);
        let artifact_type = args["type"]
            .as_str()
            .map(parse_artifact_type)
            .transpose()
            .map_err(|e| McpError {
                code: -32602,
                message: e.to_string(),
            })?;

        let filters = SearchFilters {
            artifact_type,
            limit,
            ..Default::default()
        };

        let results = self
            .search_service
            .search(query, filters)
            .await
            .map_err(|e| McpError {
                code: -32000,
                message: e.to_string(),
            })?;

        Ok(serde_json::to_value(results).unwrap_or_default())
    }

    async fn handle_get(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let id = args["id"].as_str().ok_or_else(|| McpError {
            code: -32602,
            message: "Missing id parameter".to_string(),
        })?;

        let artifact = self.artifact_service.get(id).await.map_err(|e| McpError {
            code: -32000,
            message: e.to_string(),
        })?;

        Ok(serde_json::to_value(artifact).unwrap_or_default())
    }

    async fn handle_list(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let artifact_type = args["type"]
            .as_str()
            .map(parse_artifact_type)
            .transpose()
            .map_err(|e| McpError {
                code: -32602,
                message: e.to_string(),
            })?;

        let limit = args["limit"].as_u64().map(|l| l as usize);

        let after = args["after"]
            .as_str()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()
            .map_err(|e| McpError {
                code: -32602,
                message: format!("Invalid after format: {e}"),
            })?;

        let before = args["before"]
            .as_str()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()
            .map_err(|e| McpError {
                code: -32602,
                message: format!("Invalid before format: {e}"),
            })?;

        let filters = SearchFilters {
            artifact_type,
            after,
            before,
            limit,
            ..Default::default()
        };

        let artifacts = self
            .artifact_service
            .list(filters)
            .await
            .map_err(|e| McpError {
                code: -32000,
                message: e.to_string(),
            })?;

        Ok(serde_json::to_value(artifacts).unwrap_or_default())
    }

    async fn handle_add(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let artifact_type = args["type"].as_str().ok_or_else(|| McpError {
            code: -32602,
            message: "Missing type parameter".to_string(),
        })?;
        let artifact_type = parse_artifact_type(artifact_type).map_err(|e| McpError {
            code: -32602,
            message: e.to_string(),
        })?;

        let content = args["content"]
            .as_str()
            .ok_or_else(|| McpError {
                code: -32602,
                message: "Missing content parameter".to_string(),
            })?
            .to_string();

        let format = args["format"]
            .as_str()
            .map(|f| match f {
                "markdown" => ContentFormat::Markdown,
                "yaml" => ContentFormat::Yaml,
                "json" => ContentFormat::Json,
                _ => ContentFormat::Markdown,
            })
            .unwrap_or(ContentFormat::Markdown);

        let name = args["name"].as_str().map(|s| s.to_string());
        let metadata = args["metadata"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        let artifact = self
            .artifact_service
            .add(artifact_type, content, format, name, metadata)
            .await
            .map_err(|e| McpError {
                code: -32000,
                message: e.to_string(),
            })?;

        Ok(serde_json::to_value(artifact).unwrap_or_default())
    }

    async fn handle_update(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let id = args["id"].as_str().ok_or_else(|| McpError {
            code: -32602,
            message: "Missing id parameter".to_string(),
        })?;

        let content = args["content"].as_str().map(|s| s.to_string());
        let name = args["name"].as_str().map(|s| s.to_string());
        let metadata = args["metadata"].as_object().map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<String, String>>()
        });

        let artifact = self
            .artifact_service
            .update(id, content, name, metadata)
            .await
            .map_err(|e| McpError {
                code: -32000,
                message: e.to_string(),
            })?;

        Ok(serde_json::to_value(artifact).unwrap_or_default())
    }

    async fn handle_remove(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let id = args["id"].as_str().ok_or_else(|| McpError {
            code: -32602,
            message: "Missing id parameter".to_string(),
        })?;

        let removed = self
            .artifact_service
            .remove(id)
            .await
            .map_err(|e| McpError {
                code: -32000,
                message: e.to_string(),
            })?;

        Ok(serde_json::json!({ "removed": removed }))
    }

    async fn handle_changes(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, McpError> {
        let after = args["after"]
            .as_str()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()
            .map_err(|e| McpError {
                code: -32602,
                message: format!("Invalid after format: {e}"),
            })?;

        let before = args["before"]
            .as_str()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()
            .map_err(|e| McpError {
                code: -32602,
                message: format!("Invalid before format: {e}"),
            })?;

        let filters = SearchFilters {
            after,
            before,
            ..Default::default()
        };

        let artifacts = self
            .artifact_service
            .list(filters)
            .await
            .map_err(|e| McpError {
                code: -32000,
                message: e.to_string(),
            })?;

        Ok(serde_json::to_value(artifacts).unwrap_or_default())
    }
}

fn parse_artifact_type(s: &str) -> Result<ArtifactType> {
    match s.to_lowercase().as_str() {
        "intent" => Ok(ArtifactType::Intent),
        "invariant" => Ok(ArtifactType::Invariant),
        "contract" => Ok(ArtifactType::Contract),
        "algorithm" => Ok(ArtifactType::Algorithm),
        "evaluation" => Ok(ArtifactType::Evaluation),
        "pace" => Ok(ArtifactType::Pace),
        "monitor" => Ok(ArtifactType::Monitor),
        _ => anyhow::bail!("Unknown artifact type: {s}"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpCapabilities {
    tools: Vec<McpTool>,
}

#[derive(Debug, Deserialize)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Artifact, SearchResult};
    use std::sync::Mutex;

    struct TestEmbedding;

    #[async_trait::async_trait]
    impl crate::embedding::EmbeddingProvider for TestEmbedding {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.0; 384])
        }
        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![0.0; 384]).collect())
        }
        fn model_id(&self) -> &str {
            "test-model"
        }
        fn dimensions(&self) -> usize {
            384
        }
    }

    struct TestDatabase {
        artifacts: Mutex<Vec<Artifact>>,
    }

    impl TestDatabase {
        fn new() -> Self {
            Self {
                artifacts: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait::async_trait]
    impl crate::db::Database for TestDatabase {
        async fn insert(&self, artifact: &Artifact) -> Result<()> {
            self.artifacts.lock().unwrap().push(artifact.clone());
            Ok(())
        }
        async fn get(&self, id: &str) -> Result<Option<Artifact>> {
            Ok(self
                .artifacts
                .lock()
                .unwrap()
                .iter()
                .find(|a| a.id == id)
                .cloned())
        }
        async fn update(&self, _artifact: &Artifact) -> Result<()> {
            Ok(())
        }
        async fn delete(&self, _id: &str) -> Result<bool> {
            Ok(false)
        }
        async fn list(&self, _filters: SearchFilters) -> Result<Vec<Artifact>> {
            Ok(self.artifacts.lock().unwrap().clone())
        }
        async fn search(
            &self,
            _query_embedding: &[f32],
            _filters: SearchFilters,
        ) -> Result<Vec<SearchResult>> {
            Ok(vec![])
        }
    }

    #[test]
    fn get_available_tools_returns_all_tools_by_default() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding);
        let server = McpServer::new(db, embedding, None, None);

        let tools = server.get_available_tools();
        assert_eq!(tools.len(), 7);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"dna_search"));
        assert!(tool_names.contains(&"dna_get"));
        assert!(tool_names.contains(&"dna_list"));
        assert!(tool_names.contains(&"dna_changes"));
        assert!(tool_names.contains(&"dna_add"));
        assert!(tool_names.contains(&"dna_update"));
        assert!(tool_names.contains(&"dna_remove"));
    }

    #[test]
    fn get_available_tools_with_include_filter() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding);
        let server = McpServer::new(
            db,
            embedding,
            Some(vec!["search".to_string(), "get".to_string()]),
            None,
        );

        let tools = server.get_available_tools();
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"dna_search"));
        assert!(tool_names.contains(&"dna_get"));
    }

    #[test]
    fn get_available_tools_with_exclude_filter() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding);
        let server = McpServer::new(
            db,
            embedding,
            None,
            Some(vec![
                "add".to_string(),
                "update".to_string(),
                "remove".to_string(),
            ]),
        );

        let tools = server.get_available_tools();
        assert_eq!(tools.len(), 4);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(!tool_names.contains(&"dna_add"));
        assert!(!tool_names.contains(&"dna_update"));
        assert!(!tool_names.contains(&"dna_remove"));
    }

    #[test]
    fn mcp_tool_serializes_correctly() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test_tool"));
        assert!(json.contains("A test tool"));
    }

    #[test]
    fn search_tool_has_correct_schema() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding);
        let server = McpServer::new(db, embedding, None, None);

        let tools = server.get_available_tools();
        let search_tool = tools.iter().find(|t| t.name == "dna_search").unwrap();

        let schema = &search_tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("query")));
    }

    #[test]
    fn parse_artifact_type_works() {
        assert!(matches!(
            parse_artifact_type("intent").unwrap(),
            ArtifactType::Intent
        ));
        assert!(matches!(
            parse_artifact_type("invariant").unwrap(),
            ArtifactType::Invariant
        ));
        assert!(matches!(
            parse_artifact_type("contract").unwrap(),
            ArtifactType::Contract
        ));
        assert!(parse_artifact_type("invalid").is_err());
    }
}
