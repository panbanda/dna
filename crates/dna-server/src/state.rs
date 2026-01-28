use anyhow::Result;
use dna::db::lance::LanceDatabase;
use dna::db::Database;
use dna::embedding;
use dna::embedding::EmbeddingProvider;
use dna::services::{ArtifactService, ProjectConfig, SearchService};
use figment::providers::{Env, Serialized};
use figment::Figment;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Database>,
    pub embedding: Arc<dyn EmbeddingProvider>,
    pub artifact_service: Arc<ArtifactService>,
    pub search_service: Arc<SearchService>,
    pub config: ProjectConfig,
}

impl AppState {
    pub async fn from_env() -> Result<Self> {
        let config: ProjectConfig = Figment::new()
            .merge(Serialized::defaults(ProjectConfig::default()))
            .merge(Env::prefixed("DNA_").split("__"))
            .extract()?;

        let storage_uri = config
            .storage
            .uri
            .clone()
            .unwrap_or_else(|| ".dna/db/artifacts.lance".to_string());

        let lance_db = LanceDatabase::new(&storage_uri).await?;
        lance_db.init().await?;
        let db: Arc<dyn Database> = Arc::new(lance_db);

        let embedding = embedding::create_provider(&config.model).await?;

        let artifact_service = Arc::new(ArtifactService::new(db.clone(), embedding.clone()));
        let search_service = Arc::new(SearchService::new(db.clone(), embedding.clone()));

        Ok(Self {
            db,
            embedding,
            artifact_service,
            search_service,
            config,
        })
    }
}
