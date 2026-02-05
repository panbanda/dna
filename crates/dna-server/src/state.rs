use anyhow::Result;
use dna::db::lance::LanceDatabase;
use dna::db::Database;
use dna::embedding;
use dna::embedding::EmbeddingProvider;
use dna::mcp::{RegisteredKind, RegisteredLabel};
use dna::services::{ArtifactService, ProjectConfig, SearchService};
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// API documentation branding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocsConfig {
    /// Enable API documentation (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// API title shown in docs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// API description shown in docs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// API version shown in docs (defaults to DNA version)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for ApiDocsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            title: None,
            description: None,
            version: None,
        }
    }
}

/// Server-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind address (default: 0.0.0.0:3000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<String>,
    /// API documentation settings.
    /// Accepts a bool (e.g. `api_docs = false`) or a full config table.
    #[serde(default, deserialize_with = "deserialize_api_docs")]
    pub api_docs: ApiDocsConfig,
}

fn deserialize_api_docs<'de, D>(deserializer: D) -> Result<ApiDocsConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ApiDocsOrBool {
        Bool(bool),
        Config(ApiDocsConfig),
    }

    match ApiDocsOrBool::deserialize(deserializer)? {
        ApiDocsOrBool::Bool(enabled) => Ok(ApiDocsConfig {
            enabled,
            ..Default::default()
        }),
        ApiDocsOrBool::Config(config) => Ok(config),
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Database>,
    pub embedding: Arc<dyn EmbeddingProvider>,
    pub artifact_service: Arc<ArtifactService>,
    pub search_service: Arc<SearchService>,
    pub registered_kinds: Vec<RegisteredKind>,
    pub registered_labels: Vec<RegisteredLabel>,
    pub server_config: ServerConfig,
}

/// Combined configuration for figment extraction
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CombinedConfig {
    #[serde(flatten)]
    project: ProjectConfig,
    #[serde(default)]
    server: ServerConfig,
}

impl AppState {
    pub async fn from_env() -> Result<Self> {
        let mut figment = Figment::new().merge(Serialized::defaults(CombinedConfig::default()));

        // Try loading .dna/config.toml if it exists
        let config_path = std::path::Path::new(".dna/config.toml");
        if config_path.exists() {
            figment = figment.merge(Toml::file(config_path));
        }

        let config: CombinedConfig = figment.merge(Env::prefixed("DNA_").split("__")).extract()?;

        let storage_uri = config
            .project
            .storage
            .uri
            .clone()
            .unwrap_or_else(|| ".dna/db/artifacts.lance".to_string());

        let lance_db = LanceDatabase::new(&storage_uri).await?;
        lance_db.init().await?;
        let db: Arc<dyn Database> = Arc::new(lance_db);

        let embedding = embedding::create_provider(&config.project.model).await?;

        let artifact_service = Arc::new(ArtifactService::new(db.clone(), embedding.clone()));
        let search_service = Arc::new(SearchService::new(db.clone(), embedding.clone()));

        let registered_kinds: Vec<RegisteredKind> = config
            .project
            .kinds
            .definitions
            .iter()
            .map(|d| RegisteredKind {
                slug: d.slug.clone(),
                description: d.description.clone(),
            })
            .collect();

        let registered_labels: Vec<RegisteredLabel> = config
            .project
            .labels
            .definitions
            .iter()
            .map(|d| RegisteredLabel {
                key: d.key.clone(),
                description: d.description.clone(),
            })
            .collect();

        Ok(Self {
            db,
            embedding,
            artifact_service,
            search_service,
            registered_kinds,
            registered_labels,
            server_config: config.server,
        })
    }
}
