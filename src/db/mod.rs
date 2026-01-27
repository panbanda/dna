pub mod lance;
pub mod schema;

use crate::services::{Artifact, SearchFilters, SearchResult};
use anyhow::Result;

/// Database trait for artifact storage
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    /// Insert a new artifact
    async fn insert(&self, artifact: &Artifact) -> Result<()>;

    /// Get an artifact by ID
    async fn get(&self, id: &str) -> Result<Option<Artifact>>;

    /// Update an existing artifact
    async fn update(&self, artifact: &Artifact) -> Result<()>;

    /// Delete an artifact
    async fn delete(&self, id: &str) -> Result<bool>;

    /// List artifacts with filters
    async fn list(&self, filters: SearchFilters) -> Result<Vec<Artifact>>;

    /// Semantic search
    async fn search(
        &self,
        query_embedding: &[f32],
        filters: SearchFilters,
    ) -> Result<Vec<SearchResult>>;
}
