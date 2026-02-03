use super::types::*;
use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Service for kind-scoped artifact operations.
///
/// Wraps ArtifactService but pre-filters by a specific kind slug,
/// providing a convenient typed interface per registered kind.
pub struct KindService {
    kind_slug: String,
    db: Arc<dyn Database>,
    embedding: Arc<dyn EmbeddingProvider>,
}

impl KindService {
    pub fn new(
        kind_slug: String,
        db: Arc<dyn Database>,
        embedding: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            kind_slug,
            db,
            embedding,
        }
    }

    pub fn kind_slug(&self) -> &str {
        &self.kind_slug
    }

    /// Add a new artifact of this kind
    pub async fn add(
        &self,
        content: String,
        format: ContentFormat,
        name: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<Artifact> {
        let mut artifact = Artifact::new(
            self.kind_slug.clone(),
            content.clone(),
            format,
            name,
            metadata,
            self.embedding.model_id().to_string(),
        );

        let embedding = self
            .embedding
            .embed(&content)
            .await
            .context("Failed to generate embedding")?;
        artifact.embedding = Some(embedding);

        self.db
            .insert(&artifact)
            .await
            .context("Failed to insert artifact")?;

        Ok(artifact)
    }

    /// List artifacts of this kind
    pub async fn list(&self, limit: Option<usize>) -> Result<Vec<Artifact>> {
        let filters = SearchFilters {
            kind: Some(self.kind_slug.clone()),
            limit,
            ..Default::default()
        };
        self.db
            .list(filters)
            .await
            .context("Failed to list artifacts")
    }

    /// Search artifacts of this kind
    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let embedding = self
            .embedding
            .embed(query)
            .await
            .context("Failed to generate query embedding")?;

        let filters = SearchFilters {
            kind: Some(self.kind_slug.clone()),
            limit,
            ..Default::default()
        };

        self.db
            .search(&embedding, filters)
            .await
            .context("Failed to search artifacts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        async fn list(&self, filters: SearchFilters) -> Result<Vec<Artifact>> {
            let all: Vec<_> = self.artifacts.lock().unwrap().values().cloned().collect();
            Ok(all
                .into_iter()
                .filter(|a| filters.kind.as_ref().is_none_or(|k| a.kind == *k))
                .collect())
        }
        async fn search(
            &self,
            _query_embedding: &[f32],
            filters: SearchFilters,
        ) -> Result<Vec<SearchResult>> {
            let all: Vec<_> = self.artifacts.lock().unwrap().values().cloned().collect();
            Ok(all
                .into_iter()
                .filter(|a| filters.kind.as_ref().is_none_or(|k| a.kind == *k))
                .map(|a| SearchResult {
                    artifact: a,
                    score: 0.9,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn add_creates_artifact_with_correct_kind() {
        let db = Arc::new(TestDatabase::new());
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);
        let service = KindService::new("evaluation".to_string(), db, embedding);

        let artifact = service
            .add(
                "test content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
            )
            .await
            .unwrap();

        assert_eq!(artifact.kind, "evaluation");
    }

    #[tokio::test]
    async fn list_filters_by_kind() {
        let db = Arc::new(TestDatabase::new());
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);

        // Insert artifacts of different kinds
        let a1 = Artifact::new(
            "evaluation".to_string(),
            "eval".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "test".to_string(),
        );
        let a2 = Artifact::new(
            "intent".to_string(),
            "intent".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "test".to_string(),
        );
        db.insert(&a1).await.unwrap();
        db.insert(&a2).await.unwrap();

        let service = KindService::new("evaluation".to_string(), db, embedding);
        let results = service.list(None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, "evaluation");
    }

    #[tokio::test]
    async fn search_filters_by_kind() {
        let db = Arc::new(TestDatabase::new());
        let embedding: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbedding);

        let a1 = Artifact::new(
            "evaluation".to_string(),
            "eval content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "test".to_string(),
        );
        let a2 = Artifact::new(
            "intent".to_string(),
            "intent content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "test".to_string(),
        );
        db.insert(&a1).await.unwrap();
        db.insert(&a2).await.unwrap();

        let service = KindService::new("evaluation".to_string(), db, embedding);
        let results = service.search("eval", None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].artifact.kind, "evaluation");
    }
}
