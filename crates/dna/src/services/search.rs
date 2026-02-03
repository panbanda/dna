use super::types::*;
use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Service for semantic search operations
pub struct SearchService {
    db: Arc<dyn Database>,
    embedding: Arc<dyn EmbeddingProvider>,
}

impl SearchService {
    /// Create a new search service
    pub fn new(db: Arc<dyn Database>, embedding: Arc<dyn EmbeddingProvider>) -> Self {
        Self { db, embedding }
    }

    /// Perform semantic search
    pub async fn search(&self, query: &str, filters: SearchFilters) -> Result<Vec<SearchResult>> {
        // Generate query embedding
        let query_embedding = self
            .embedding
            .embed(query)
            .await
            .context("Failed to generate query embedding")?;

        // Search in database
        self.db
            .search(&query_embedding, filters)
            .await
            .context("Failed to search database")
    }

    /// Check if artifacts have mixed embedding models
    pub async fn check_embedding_consistency(&self) -> Result<Vec<String>> {
        let artifacts = self.db.list(SearchFilters::default()).await?;
        let current_model = self.embedding.model_id();

        let inconsistent: Vec<String> = artifacts
            .into_iter()
            .filter(|a| a.embedding_model != current_model)
            .map(|a| a.id)
            .collect();

        Ok(inconsistent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
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
        artifacts: Mutex<Vec<Artifact>>,
        search_results: Mutex<Vec<SearchResult>>,
    }

    impl TestDatabase {
        fn new() -> Self {
            Self {
                artifacts: Mutex::new(vec![]),
                search_results: Mutex::new(vec![]),
            }
        }

        fn with_artifacts(artifacts: Vec<Artifact>) -> Self {
            Self {
                artifacts: Mutex::new(artifacts),
                search_results: Mutex::new(vec![]),
            }
        }

        fn with_search_results(results: Vec<SearchResult>) -> Self {
            Self {
                artifacts: Mutex::new(vec![]),
                search_results: Mutex::new(results),
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
            Ok(self.search_results.lock().unwrap().clone())
        }

        async fn version(&self) -> Result<u64> {
            Ok(1)
        }

        async fn get_at_version(&self, id: &str, _version: u64) -> Result<Option<Artifact>> {
            self.get(id).await
        }

        async fn list_versions(
            &self,
            _limit: Option<usize>,
        ) -> Result<Vec<crate::db::VersionInfo>> {
            Ok(vec![])
        }

        async fn compact(&self) -> Result<crate::db::CompactStats> {
            Ok(crate::db::CompactStats {
                files_merged: 0,
                bytes_saved: 0,
            })
        }

        async fn cleanup_versions(&self, _keep_versions: usize) -> Result<crate::db::CleanupStats> {
            Ok(crate::db::CleanupStats {
                versions_removed: 0,
                bytes_freed: 0,
            })
        }
    }

    #[tokio::test]
    async fn search_returns_empty_for_empty_db() {
        let db = Arc::new(TestDatabase::new());
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![0.1, 0.2, 0.3]));
        let service = SearchService::new(db, embedding);

        let results = service
            .search("test query", SearchFilters::default())
            .await
            .unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn search_returns_results_from_db() {
        let artifact = Artifact::new(
            "intent".to_string(),
            "result content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "model".to_string(),
        );

        let search_result = SearchResult {
            artifact: artifact.clone(),
            score: 0.95,
        };

        let db = Arc::new(TestDatabase::with_search_results(vec![search_result]));
        let embedding = Arc::new(TestEmbedding::new("test-model", vec![0.1, 0.2, 0.3]));
        let service = SearchService::new(db, embedding);

        let results = service
            .search("query", SearchFilters::default())
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score, 0.95);
    }

    #[tokio::test]
    async fn check_embedding_consistency_returns_mismatched_ids() {
        let mut artifact1 = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "current-model".to_string(),
        );
        artifact1.id = "matching".to_string();

        let mut artifact2 = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "old-model".to_string(),
        );
        artifact2.id = "mismatched".to_string();

        let db = Arc::new(TestDatabase::with_artifacts(vec![artifact1, artifact2]));
        let embedding = Arc::new(TestEmbedding::new("current-model", vec![]));
        let service = SearchService::new(db, embedding);

        let inconsistent = service.check_embedding_consistency().await.unwrap();

        assert_eq!(inconsistent.len(), 1);
        assert_eq!(inconsistent[0], "mismatched");
    }

    #[tokio::test]
    async fn check_embedding_consistency_empty_when_all_match() {
        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "current-model".to_string(),
        );

        let db = Arc::new(TestDatabase::with_artifacts(vec![artifact]));
        let embedding = Arc::new(TestEmbedding::new("current-model", vec![]));
        let service = SearchService::new(db, embedding);

        let inconsistent = service.check_embedding_consistency().await.unwrap();

        assert!(inconsistent.is_empty());
    }
}
