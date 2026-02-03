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
        context: Option<String>,
    ) -> Result<Artifact> {
        let mut artifact = Artifact::new(
            self.kind_slug.clone(),
            content.clone(),
            format,
            name,
            metadata,
            self.embedding.model_id().to_string(),
        );

        // Generate content embedding
        let embedding = self
            .embedding
            .embed(&content)
            .await
            .context("Failed to generate embedding")?;
        artifact.embedding = Some(embedding);

        // Set context and generate context embedding if provided
        artifact.context = context.clone();
        if let Some(ctx) = &context {
            let context_embedding = self
                .embedding
                .embed(ctx)
                .await
                .context("Failed to generate context embedding")?;
            artifact.context_embedding = Some(context_embedding);
        }

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
    use crate::testing::{TestDatabase, TestEmbedding};

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
                None,
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
