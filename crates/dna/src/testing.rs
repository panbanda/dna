//! Test utilities for DNA crate
//!
//! This module provides reusable test doubles for unit and integration testing.
//! It includes mock implementations of `EmbeddingProvider` and `Database` traits.

use crate::db::Database;
use crate::embedding::EmbeddingProvider;
use crate::services::{Artifact, SearchFilters, SearchResult};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Mutex;

/// Test embedding provider that generates deterministic embeddings based on text content.
///
/// Useful for testing search functionality where you need reproducible embeddings.
pub struct TestEmbedding;

#[async_trait::async_trait]
impl EmbeddingProvider for TestEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Deterministic embedding based on text content for testing search
        let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        Ok((0..384)
            .map(|i| ((hash.wrapping_add(i as u32) % 1000) as f32) / 1000.0)
            .collect())
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    fn model_id(&self) -> &str {
        "test-embedding-model"
    }

    fn dimensions(&self) -> usize {
        384
    }
}

/// In-memory database implementation for testing.
///
/// Thread-safe via Mutex, suitable for unit tests.
pub struct TestDatabase {
    artifacts: Mutex<HashMap<String, Artifact>>,
}

impl TestDatabase {
    pub fn new() -> Self {
        Self {
            artifacts: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for TestDatabase {
    fn default() -> Self {
        Self::new()
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
            .filter(|a| filters.after.is_none_or(|dt| a.updated_at > dt))
            .filter(|a| filters.before.is_none_or(|dt| a.updated_at < dt))
            .take(filters.limit.unwrap_or(usize::MAX))
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
            .take(filters.limit.unwrap_or(usize::MAX))
            .map(|a| SearchResult {
                artifact: a,
                score: 0.85,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ContentFormat;

    #[tokio::test]
    async fn test_embedding_is_deterministic() {
        let provider = TestEmbedding;
        let e1 = provider.embed("hello").await.unwrap();
        let e2 = provider.embed("hello").await.unwrap();
        assert_eq!(e1, e2);
    }

    #[tokio::test]
    async fn test_embedding_dimensions() {
        let provider = TestEmbedding;
        let embedding = provider.embed("test").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_database_crud() {
        let db = TestDatabase::new();
        let artifact = Artifact::new(
            "intent".to_string(),
            "content".to_string(),
            ContentFormat::Markdown,
            None,
            HashMap::new(),
            "test".to_string(),
        );

        db.insert(&artifact).await.unwrap();
        let retrieved = db.get(&artifact.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "content");

        assert!(db.delete(&artifact.id).await.unwrap());
        assert!(db.get(&artifact.id).await.unwrap().is_none());
    }
}
