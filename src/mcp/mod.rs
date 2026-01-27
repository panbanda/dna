use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use crate::services::{ArtifactService, SearchService};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// MCP Server for DNA
pub struct McpServer {
    #[allow(dead_code)]
    artifact_service: ArtifactService,
    #[allow(dead_code)]
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
                        "since": {"type": "string"},
                        "limit": {"type": "integer"}
                    }
                }),
            },
            McpTool {
                name: "dna_changes".to_string(),
                description: "Artifacts modified since timestamp/git-ref".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "since": {"type": "string"}
                    },
                    "required": ["since"]
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
        // TODO: Implement full MCP protocol over stdio
        // For now, just output tool list
        println!("{}", serde_json::to_string_pretty(&tools)?);
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct McpTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Artifact, ArtifactType, ContentFormat, SearchFilters, SearchResult};
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct TestEmbedding;

    #[async_trait::async_trait]
    impl EmbeddingProvider for TestEmbedding {
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
    impl Database for TestDatabase {
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
}
