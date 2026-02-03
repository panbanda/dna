pub mod lance;
pub mod schema;

use crate::services::{Artifact, SearchFilters, SearchResult};
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Information about a database version
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: u64,
    pub timestamp: DateTime<Utc>,
}

/// Statistics from compaction
#[derive(Debug, Clone)]
pub struct CompactStats {
    pub files_merged: usize,
    pub bytes_saved: u64,
}

/// Statistics from cleanup
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub versions_removed: usize,
    pub bytes_freed: u64,
}

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

    /// Get the current database version number
    async fn version(&self) -> Result<u64>;

    /// Get artifact at a specific version
    async fn get_at_version(&self, id: &str, version: u64) -> Result<Option<Artifact>>;

    /// List all database versions with metadata
    async fn list_versions(&self, limit: Option<usize>) -> Result<Vec<VersionInfo>>;

    /// Compact the database (merge small files)
    async fn compact(&self) -> Result<CompactStats>;

    /// Cleanup old versions, keeping the specified number of recent versions
    async fn cleanup_versions(&self, keep_versions: usize) -> Result<CleanupStats>;
}
