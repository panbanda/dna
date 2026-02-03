use super::types::{
    estimate_tokens, get_model_info, slugify_kind, Artifact, ContentFormat, ReindexTarget,
    SearchFilters,
};
use super::ServiceError;
use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Service for artifact CRUD operations
pub struct ArtifactService {
    db: Arc<dyn Database>,
    embedding: Arc<dyn EmbeddingProvider>,
}

impl ArtifactService {
    /// Create a new artifact service
    pub fn new(db: Arc<dyn Database>, embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self { db, embedding }
    }

    /// Add a new artifact
    pub async fn add(
        &self,
        kind: String,
        content: String,
        format: ContentFormat,
        name: Option<String>,
        metadata: HashMap<String, String>,
        context: Option<String>,
    ) -> Result<Artifact> {
        // Validate content and context length against model limits
        let model_info = get_model_info(self.embedding.model_id());
        let content_tokens = estimate_tokens(&content);
        if content_tokens > model_info.max_tokens {
            return Err(anyhow::anyhow!(
                "Content exceeds maximum of {} tokens (estimated {}). \
                 Reduce content length or configure a model with a larger context window.",
                model_info.max_tokens,
                content_tokens
            ));
        }

        if let Some(ctx) = &context {
            let context_tokens = estimate_tokens(ctx);
            if context_tokens > model_info.max_tokens {
                return Err(anyhow::anyhow!(
                    "Context exceeds maximum of {} tokens (estimated {}). \
                     Reduce context length or configure a model with a larger context window.",
                    model_info.max_tokens,
                    context_tokens
                ));
            }
        }

        // Create artifact with embedding model info
        let mut artifact = Artifact::new(
            slugify_kind(&kind),
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

        // Store in database
        self.db
            .insert(&artifact)
            .await
            .context("Failed to insert artifact")?;

        Ok(artifact)
    }

    /// Get artifact by ID
    pub async fn get(&self, id: &str) -> Result<Option<Artifact>> {
        self.db.get(id).await.context("Failed to get artifact")
    }

    /// Update an existing artifact
    pub async fn update(
        &self,
        id: &str,
        content: Option<String>,
        name: Option<String>,
        kind: Option<String>,
        metadata: Option<HashMap<String, String>>,
        context: Option<String>,
    ) -> Result<Artifact, ServiceError> {
        // Get existing artifact
        let mut artifact = self
            .get(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Artifact '{}' not found", id)))?;

        // Update fields
        let mut needs_reembed = false;
        if let Some(new_content) = content {
            if new_content != artifact.content {
                artifact.content = new_content;
                needs_reembed = true;
            }
        }

        if let Some(new_name) = name {
            artifact.name = Some(new_name);
        }

        if let Some(new_kind) = kind {
            artifact.kind = slugify_kind(&new_kind);
        }

        if let Some(new_metadata) = metadata {
            for (key, value) in new_metadata {
                if value.is_empty() {
                    artifact.metadata.remove(&key);
                } else {
                    artifact.metadata.insert(key, value);
                }
            }
        }

        // Update context and regenerate context embedding if changed
        let mut needs_context_reembed = false;
        if let Some(new_context) = context {
            if artifact.context.as_ref() != Some(&new_context) {
                artifact.context = Some(new_context);
                needs_context_reembed = true;
            }
        }

        artifact.updated_at = chrono::Utc::now();

        // Validate content and context length against model limits before re-embedding
        let model_info = get_model_info(self.embedding.model_id());
        if needs_reembed {
            let content_tokens = estimate_tokens(&artifact.content);
            if content_tokens > model_info.max_tokens {
                return Err(ServiceError::Validation(format!(
                    "Content exceeds maximum of {} tokens (estimated {}). \
                     Reduce content length or configure a model with a larger context window.",
                    model_info.max_tokens, content_tokens
                )));
            }
        }

        if needs_context_reembed {
            if let Some(ctx) = &artifact.context {
                let context_tokens = estimate_tokens(ctx);
                if context_tokens > model_info.max_tokens {
                    return Err(ServiceError::Validation(format!(
                        "Context exceeds maximum of {} tokens (estimated {}). \
                         Reduce context length or configure a model with a larger context window.",
                        model_info.max_tokens, context_tokens
                    )));
                }
            }
        }

        // Re-embed content if changed
        if needs_reembed {
            let embedding = self
                .embedding
                .embed(&artifact.content)
                .await
                .context("Failed to generate embedding")?;
            artifact.embedding = Some(embedding);
            artifact.embedding_model = self.embedding.model_id().to_string();
        }

        // Re-embed context if changed
        if needs_context_reembed {
            if let Some(ctx) = &artifact.context {
                let context_embedding = self
                    .embedding
                    .embed(ctx)
                    .await
                    .context("Failed to generate context embedding")?;
                artifact.context_embedding = Some(context_embedding);
            }
        }

        // Update in database
        self.db
            .update(&artifact)
            .await
            .context("Failed to update artifact")?;

        Ok(artifact)
    }

    /// Remove an artifact
    pub async fn remove(&self, id: &str) -> Result<bool> {
        self.db
            .delete(id)
            .await
            .context("Failed to delete artifact")
    }

    /// List artifacts with filters
    pub async fn list(&self, filters: SearchFilters) -> Result<Vec<Artifact>> {
        self.db
            .list(filters)
            .await
            .context("Failed to list artifacts")
    }

    /// Reindex all artifacts with current embedding model
    pub async fn reindex(&self) -> Result<usize> {
        self.reindex_filtered(SearchFilters::default(), ReindexTarget::Both)
            .await
    }

    /// Reindex artifacts matching filters with specified target embeddings.
    ///
    /// Returns the number of artifacts reindexed.
    pub async fn reindex_filtered(
        &self,
        filters: SearchFilters,
        target: ReindexTarget,
    ) -> Result<usize> {
        let artifacts = self.list(filters).await?;
        let total = artifacts.len();

        for mut artifact in artifacts {
            self.reindex_artifact_embeddings(&mut artifact, target)
                .await?;
        }

        Ok(total)
    }

    /// Reindex a single artifact by ID.
    ///
    /// Returns the updated artifact, or None if not found.
    pub async fn reindex_by_id(&self, id: &str, target: ReindexTarget) -> Result<Option<Artifact>> {
        let Some(mut artifact) = self.get(id).await? else {
            return Ok(None);
        };

        self.reindex_artifact_embeddings(&mut artifact, target)
            .await?;

        Ok(Some(artifact))
    }

    /// Regenerate embeddings for an artifact and persist changes.
    async fn reindex_artifact_embeddings(
        &self,
        artifact: &mut Artifact,
        target: ReindexTarget,
    ) -> Result<()> {
        let reindex_content = matches!(target, ReindexTarget::Content | ReindexTarget::Both);
        let reindex_context = matches!(target, ReindexTarget::Context | ReindexTarget::Both);

        if reindex_content {
            let embedding = self
                .embedding
                .embed(&artifact.content)
                .await
                .context("Failed to generate content embedding during reindex")?;
            artifact.embedding = Some(embedding);
            artifact.embedding_model = self.embedding.model_id().to_string();
        }

        if reindex_context {
            if let Some(ctx) = &artifact.context {
                let context_embedding = self
                    .embedding
                    .embed(ctx)
                    .await
                    .context("Failed to generate context embedding during reindex")?;
                artifact.context_embedding = Some(context_embedding);
            }
        }

        self.db
            .update(artifact)
            .await
            .context("Failed to update artifact during reindex")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// A simple mock embedding provider for tests
    struct TestEmbedding {
        model_id: &'static str,
        embedding: Vec<f32>,
    }

    impl TestEmbedding {
        fn new(model_id: &'static str, embedding: Vec<f32>) -> Self {
            Self {
                model_id,
                embedding,
            }
        }
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for TestEmbedding {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(self.embedding.clone())
        }

        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| self.embedding.clone()).collect())
        }

        fn model_id(&self) -> &str {
            self.model_id
        }

        fn dimensions(&self) -> usize {
            self.embedding.len()
        }
    }

    /// A simple in-memory database for tests
    struct TestDatabase {
        artifacts: Mutex<HashMap<String, Artifact>>,
    }

    impl TestDatabase {
        fn new() -> Self {
            Self {
                artifacts: Mutex::new(HashMap::new()),
            }
        }

        fn with_artifact(artifact: Artifact) -> Self {
            let db = Self::new();
            db.artifacts
                .lock()
                .unwrap()
                .insert(artifact.id.clone(), artifact);
            db
        }
    }

    #[async_trait::async_trait]
    impl crate::db::Database for TestDatabase {
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
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn add_generates_embedding_and_inserts() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![0.1, 0.2, 0.3]));
        let service = ArtifactService::new(db.clone(), embedding);

        let artifact = service
            .add(
                "intent".to_string(),
                "test content".to_string(),
                ContentFormat::Markdown,
                Some("test-name".to_string()),
                HashMap::new(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(artifact.kind, "intent");
        assert_eq!(artifact.content, "test content");
        assert_eq!(artifact.name, Some("test-name".to_string()));
        assert_eq!(artifact.embedding, Some(vec![0.1, 0.2, 0.3]));

        // Verify it was inserted
        let stored = db.get(&artifact.id).await.unwrap();
        assert!(stored.is_some());
    }

    #[tokio::test]
    async fn add_slugifies_kind() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![0.1]));
        let service = ArtifactService::new(db, embedding);

        let artifact = service
            .add(
                "My Custom Type".to_string(),
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(artifact.kind, "my-custom-type");
    }

    #[tokio::test]
    async fn get_returns_none_for_missing() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let result = service.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn remove_returns_true_when_exists() {
        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "model".to_string(),
        );
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let result = service.remove(&artifact_id).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn remove_returns_false_when_missing() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let result = service.remove("nonexistent").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn list_returns_all_artifacts() {
        let a1 = Artifact::new(
            "intent".to_string(),
            "one".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "model".to_string(),
        );
        let a2 = Artifact::new(
            "contract".to_string(),
            "two".to_string(),
            ContentFormat::Json,
            None,
            HashMap::new(),
            "model".to_string(),
        );

        let db = Arc::new(TestDatabase::new());
        db.insert(&a1).await.unwrap();
        db.insert(&a2).await.unwrap();

        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let result = service.list(SearchFilters::default()).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn update_changes_content_and_reembeds() {
        let artifact = Artifact::new(
            "intent".to_string(),
            "old content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "model".to_string(),
        );
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("new-model", vec![0.5, 0.6]));
        let service = ArtifactService::new(db, embedding);

        let updated = service
            .update(
                &artifact_id,
                Some("new content".to_string()),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.content, "new content");
        assert_eq!(updated.embedding, Some(vec![0.5, 0.6]));
        assert_eq!(updated.embedding_model, "new-model");
    }

    #[tokio::test]
    async fn update_only_name_does_not_reembed() {
        let mut artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "original-model".to_string(),
        );
        artifact.embedding = Some(vec![1.0, 2.0]);
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("new-model", vec![0.5, 0.6]));
        let service = ArtifactService::new(db, embedding);

        let updated = service
            .update(
                &artifact_id,
                None,
                Some("new-name".to_string()),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.name, Some("new-name".to_string()));
        // embedding should not change since content didn't change
        assert_eq!(updated.embedding_model, "original-model");
    }

    #[tokio::test]
    async fn update_kind_changes_kind() {
        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "model".to_string(),
        );
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let updated = service
            .update(
                &artifact_id,
                None,
                None,
                Some("contract".to_string()),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.kind, "contract");
    }

    #[tokio::test]
    async fn update_kind_slugifies() {
        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "model".to_string(),
        );
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let updated = service
            .update(
                &artifact_id,
                None,
                None,
                Some("My Custom Kind".to_string()),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.kind, "my-custom-kind");
    }

    #[tokio::test]
    async fn update_nonexistent_returns_error() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let result = service
            .update(
                "nonexistent",
                Some("content".to_string()),
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_with_time_range_filters() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![0.1]));
        let service = ArtifactService::new(db, embedding);

        let after = chrono::Utc::now();
        let filters = SearchFilters {
            after: Some(after),
            ..Default::default()
        };
        let result = service.list(filters).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn reindex_updates_all_artifacts() {
        let a1 = Artifact::new(
            "intent".to_string(),
            "content one".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "old-model".to_string(),
        );
        let a2 = Artifact::new(
            "contract".to_string(),
            "content two".to_string(),
            ContentFormat::Json,
            None,
            HashMap::new(),
            "old-model".to_string(),
        );

        let db = Arc::new(TestDatabase::new());
        db.insert(&a1).await.unwrap();
        db.insert(&a2).await.unwrap();

        let embedding = Arc::new(TestEmbedding::new("new-model", vec![0.9, 0.8, 0.7]));
        let service = ArtifactService::new(db.clone(), embedding);

        let count = service.reindex().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn reindex_returns_zero_for_empty_db() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![0.1]));
        let service = ArtifactService::new(db, embedding);

        let count = service.reindex().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn update_metadata_preserves_existing() {
        let mut initial_metadata = HashMap::new();
        initial_metadata.insert("key1".to_string(), "value1".to_string());

        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            initial_metadata,
            "model".to_string(),
        );
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        let mut new_metadata = HashMap::new();
        new_metadata.insert("key2".to_string(), "value2".to_string());

        let updated = service
            .update(&artifact_id, None, None, None, Some(new_metadata), None)
            .await
            .unwrap();

        assert_eq!(updated.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(updated.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[tokio::test]
    async fn update_removes_label_with_empty_value() {
        let mut initial_metadata = HashMap::new();
        initial_metadata.insert("env".to_string(), "production".to_string());
        initial_metadata.insert("team".to_string(), "platform".to_string());

        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            initial_metadata,
            "model".to_string(),
        );
        let artifact_id = artifact.id.clone();

        let db = Arc::new(TestDatabase::with_artifact(artifact));
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![]));
        let service = ArtifactService::new(db, embedding);

        // Update with empty value for "env" label - should remove it
        let mut remove_label = HashMap::new();
        remove_label.insert("env".to_string(), "".to_string());

        let updated = service
            .update(&artifact_id, None, None, None, Some(remove_label), None)
            .await
            .unwrap();

        // Label "env" should be removed, not set to empty string
        assert!(
            !updated.metadata.contains_key("env"),
            "Label 'env' should be removed when updated with empty value"
        );
        // Label "team" should remain unchanged
        assert_eq!(
            updated.metadata.get("team"),
            Some(&"platform".to_string()),
            "Label 'team' should remain unchanged"
        );
    }

    #[tokio::test]
    async fn add_rejects_content_exceeding_token_limit() {
        let db = Arc::new(TestDatabase::new());
        // Using default BGE model with 512 token limit
        let embedding = Arc::new(TestEmbedding::new("BAAI/bge-small-en-v1.5", vec![0.1]));
        let service = ArtifactService::new(db, embedding);

        // Create content that exceeds 512 tokens (~500 words at 0.75 words/token)
        let long_content = "word ".repeat(500);

        let result = service
            .add(
                "intent".to_string(),
                long_content,
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                None,
            )
            .await;

        assert!(
            result.is_err(),
            "Should reject content exceeding token limit"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("token") || err_msg.contains("limit"),
            "Error should mention token limit: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn add_rejects_context_exceeding_token_limit() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("BAAI/bge-small-en-v1.5", vec![0.1]));
        let service = ArtifactService::new(db, embedding);

        let short_content = "Valid short content".to_string();
        // Context also gets embedded, so should also be validated
        let long_context = "word ".repeat(500);

        let result = service
            .add(
                "intent".to_string(),
                short_content,
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                Some(long_context),
            )
            .await;

        assert!(
            result.is_err(),
            "Should reject context exceeding token limit"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("token") || err_msg.contains("context") || err_msg.contains("limit"),
            "Error should mention token/context limit: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn add_accepts_content_within_token_limit() {
        let db = Arc::new(TestDatabase::new());
        // Using OpenAI model with 8191 token limit
        let embedding = Arc::new(TestEmbedding::new("text-embedding-3-small", vec![0.1]));
        let service = ArtifactService::new(db, embedding);

        // 500 words is well within 8191 tokens
        let content = "word ".repeat(500);

        let result = service
            .add(
                "intent".to_string(),
                content,
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Should accept content within token limit: {:?}",
            result.err()
        );
    }
}
