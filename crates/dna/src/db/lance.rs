use super::{schema, CleanupStats, CompactStats, Database, VersionInfo};
use crate::services::{Artifact, ContentFormat, SearchFilters, SearchResult};
use anyhow::{Context, Result};
use arrow_array::{
    cast::AsArray, Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator,
    TimestampMillisecondArray,
};
use chrono::{TimeZone, Utc};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use tokio::sync::RwLock;

const TABLE_NAME: &str = "artifacts";

/// LanceDB implementation supporting local paths and S3 URIs
pub struct LanceDatabase {
    uri: String,
    connection: RwLock<Option<lancedb::Connection>>,
}

impl LanceDatabase {
    /// Create a new LanceDB instance from a URI (local path or s3://...)
    pub async fn new(uri: &str) -> Result<Self> {
        if !uri.starts_with("s3://") {
            let path = Path::new(uri);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .context("Failed to create database directory")?;
            }
        }

        Ok(Self {
            uri: uri.to_string(),
            connection: RwLock::new(None),
        })
    }

    /// Initialize the database
    pub async fn init(&self) -> Result<()> {
        if !self.uri.starts_with("s3://") {
            tokio::fs::create_dir_all(&self.uri)
                .await
                .context("Failed to create database directory")?;
        }

        let db = lancedb::connect(&self.uri)
            .execute()
            .await
            .context("Failed to connect to LanceDB")?;

        // Create table if it doesn't exist
        let table_names = db.table_names().execute().await?;
        if !table_names.contains(&TABLE_NAME.to_string()) {
            let schema = schema::create_schema();
            db.create_empty_table(TABLE_NAME, schema)
                .execute()
                .await
                .context("Failed to create artifacts table")?;
        }

        *self.connection.write().await = Some(db);
        Ok(())
    }

    /// Get or create a connection
    async fn get_connection(&self) -> Result<lancedb::Connection> {
        let conn = self.connection.read().await;
        if let Some(ref db) = *conn {
            return Ok(db.clone());
        }
        drop(conn);

        let db = lancedb::connect(&self.uri)
            .execute()
            .await
            .context("Failed to connect to LanceDB")?;

        *self.connection.write().await = Some(db.clone());
        Ok(db)
    }

    /// Convert a single artifact to RecordBatch
    fn artifact_to_batch(artifact: &Artifact) -> Result<RecordBatch> {
        schema::artifacts_to_batch(std::slice::from_ref(artifact))
    }

    /// Convert RecordBatch row to Artifact
    fn batch_to_artifacts(batch: &RecordBatch) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::with_capacity(batch.num_rows());

        let ids = batch.column(0).as_string::<i32>();
        let types = batch.column(1).as_string::<i32>();
        let names = batch.column(2).as_string::<i32>();
        let contents = batch.column(3).as_string::<i32>();
        let formats = batch.column(4).as_string::<i32>();
        let metadata_col = batch.column(5).as_string::<i32>();
        let embeddings = batch
            .column(6)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .context("Failed to cast embedding column")?;
        let embedding_models = batch.column(7).as_string::<i32>();
        let contexts = batch.column(8).as_string::<i32>();
        let context_embeddings = batch
            .column(9)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .context("Failed to cast context_embedding column")?;
        let created_ats = batch
            .column(10)
            .as_any()
            .downcast_ref::<TimestampMillisecondArray>()
            .context("Failed to cast created_at column")?;
        let updated_ats = batch
            .column(11)
            .as_any()
            .downcast_ref::<TimestampMillisecondArray>()
            .context("Failed to cast updated_at column")?;

        for i in 0..batch.num_rows() {
            let id = ids.value(i).to_string();
            let kind = types.value(i).to_string();
            let name = if names.is_null(i) {
                None
            } else {
                Some(names.value(i).to_string())
            };
            let content = contents.value(i).to_string();
            let format = ContentFormat::from_str(formats.value(i))?;
            let metadata: HashMap<String, String> =
                serde_json::from_str(metadata_col.value(i)).unwrap_or_default();

            let embedding_list = embeddings.value(i);
            let embedding_array = embedding_list
                .as_any()
                .downcast_ref::<Float32Array>()
                .context("Failed to cast embedding values")?;
            let embedding: Vec<f32> = (0..embedding_array.len())
                .map(|j| embedding_array.value(j))
                .collect();

            let embedding_model = embedding_models.value(i).to_string();

            let context = if contexts.is_null(i) {
                None
            } else {
                Some(contexts.value(i).to_string())
            };

            let context_embedding = if context_embeddings.is_null(i) {
                None
            } else {
                let context_emb_list = context_embeddings.value(i);
                let context_emb_array = context_emb_list
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .context("Failed to cast context_embedding values")?;
                Some(
                    (0..context_emb_array.len())
                        .map(|j| context_emb_array.value(j))
                        .collect(),
                )
            };

            let created_at = Utc.timestamp_millis_opt(created_ats.value(i)).unwrap();
            let updated_at = Utc.timestamp_millis_opt(updated_ats.value(i)).unwrap();

            artifacts.push(Artifact {
                id,
                kind,
                name,
                content,
                format,
                metadata,
                embedding: Some(embedding),
                embedding_model,
                context,
                context_embedding,
                created_at,
                updated_at,
            });
        }

        Ok(artifacts)
    }
}

#[async_trait::async_trait]
impl Database for LanceDatabase {
    async fn insert(&self, artifact: &Artifact) -> Result<()> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let batch = Self::artifact_to_batch(artifact)?;
        let schema = batch.schema();

        table
            .add(RecordBatchIterator::new(vec![Ok(batch)], schema))
            .execute()
            .await
            .context("Failed to insert artifact")?;

        tracing::debug!("Inserted artifact: {}", artifact.id);
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Artifact>> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let filter = format!("id = '{}'", id.replace('\'', "''"));
        let mut stream = table.query().only_if(filter).execute().await?;

        if let Some(batch) = stream.try_next().await? {
            let artifacts = Self::batch_to_artifacts(&batch)?;
            if let Some(artifact) = artifacts.into_iter().next() {
                return Ok(Some(artifact));
            }
        }

        Ok(None)
    }

    async fn update(&self, artifact: &Artifact) -> Result<()> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        // Delete existing and re-insert (LanceDB doesn't have native update)
        let filter = format!("id = '{}'", artifact.id.replace('\'', "''"));
        table
            .delete(&filter)
            .await
            .context("Failed to delete old artifact during update")?;

        let batch = Self::artifact_to_batch(artifact)?;
        let schema = batch.schema();

        table
            .add(RecordBatchIterator::new(vec![Ok(batch)], schema))
            .execute()
            .await
            .context("Failed to insert updated artifact")?;

        tracing::debug!("Updated artifact: {}", artifact.id);
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        // First check if artifact exists
        let exists = self.get(id).await?.is_some();
        if !exists {
            return Ok(false);
        }

        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let filter = format!("id = '{}'", id.replace('\'', "''"));
        table
            .delete(&filter)
            .await
            .context("Failed to delete artifact")?;

        tracing::debug!("Deleted artifact: {}", id);
        Ok(true)
    }

    async fn list(&self, filters: SearchFilters) -> Result<Vec<Artifact>> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let mut query = table.query();

        // Build filter string
        let mut filter_parts: Vec<String> = Vec::new();

        if let Some(kind) = &filters.kind {
            filter_parts.push(format!("kind = '{}'", kind.replace('\'', "''")));
        }

        if let Some(after) = &filters.after {
            filter_parts.push(format!(
                "updated_at >= arrow_cast({}, 'Timestamp(Millisecond, None)')",
                after.timestamp_millis()
            ));
        }

        if let Some(before) = &filters.before {
            filter_parts.push(format!(
                "updated_at < arrow_cast({}, 'Timestamp(Millisecond, None)')",
                before.timestamp_millis()
            ));
        }

        for (key, value) in &filters.metadata {
            // Filter on JSON metadata field
            filter_parts.push(format!(
                "metadata LIKE '%\"{}\":\"{}\"%'",
                key.replace('\'', "''"),
                value.replace('\'', "''")
            ));
        }

        if !filter_parts.is_empty() {
            query = query.only_if(filter_parts.join(" AND "));
        }

        if let Some(limit) = filters.limit {
            query = query.limit(limit);
        }

        let mut stream = query.execute().await?;
        let mut artifacts = Vec::new();

        while let Some(batch) = stream.try_next().await? {
            let batch_artifacts = Self::batch_to_artifacts(&batch)?;
            artifacts.extend(batch_artifacts);
        }

        Ok(artifacts)
    }

    async fn search(
        &self,
        query_embedding: &[f32],
        filters: SearchFilters,
    ) -> Result<Vec<SearchResult>> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let limit = filters.limit.unwrap_or(10);

        let mut query = table
            .vector_search(query_embedding.to_vec())
            .context("Failed to create vector search")?
            .limit(limit)
            .column("embedding");

        // Build filter string
        let mut filter_parts: Vec<String> = Vec::new();

        if let Some(kind) = &filters.kind {
            filter_parts.push(format!("kind = '{}'", kind.replace('\'', "''")));
        }

        if let Some(after) = &filters.after {
            filter_parts.push(format!(
                "updated_at >= arrow_cast({}, 'Timestamp(Millisecond, None)')",
                after.timestamp_millis()
            ));
        }

        if let Some(before) = &filters.before {
            filter_parts.push(format!(
                "updated_at < arrow_cast({}, 'Timestamp(Millisecond, None)')",
                before.timestamp_millis()
            ));
        }

        if !filter_parts.is_empty() {
            query = query.only_if(filter_parts.join(" AND "));
        }

        let mut stream = query.execute().await?;
        let mut results = Vec::new();

        while let Some(batch) = stream.try_next().await? {
            // Get distance column (added by vector search)
            let distance_col = batch.column_by_name("_distance");

            let artifacts = Self::batch_to_artifacts(&batch)?;

            for (i, artifact) in artifacts.into_iter().enumerate() {
                let score = if let Some(dist) = distance_col {
                    let dist_array = dist.as_any().downcast_ref::<Float32Array>().unwrap();
                    1.0 / (1.0 + dist_array.value(i)) // Convert distance to similarity score
                } else {
                    1.0
                };

                results.push(SearchResult { artifact, score });
            }
        }

        Ok(results)
    }

    async fn version(&self) -> Result<u64> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let version = table
            .version()
            .await
            .context("Failed to get table version")?;
        Ok(version)
    }

    async fn get_at_version(&self, id: &str, version: u64) -> Result<Option<Artifact>> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        // checkout() mutates the table in place, so we need to restore after querying
        table
            .checkout(version)
            .await
            .context("Failed to checkout version")?;

        let filter = format!("id = '{}'", id.replace('\'', "''"));
        let mut stream = table.query().only_if(filter).execute().await?;

        let result = if let Some(batch) = stream.try_next().await? {
            let artifacts = Self::batch_to_artifacts(&batch)?;
            artifacts.into_iter().next()
        } else {
            None
        };

        // Restore to latest version
        table
            .checkout_latest()
            .await
            .context("Failed to restore to latest version")?;

        Ok(result)
    }

    async fn list_versions(&self, limit: Option<usize>) -> Result<Vec<VersionInfo>> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let lance_versions = table
            .list_versions()
            .await
            .context("Failed to list versions")?;

        let mut versions: Vec<VersionInfo> = lance_versions
            .into_iter()
            .map(|v| VersionInfo {
                version: v.version,
                timestamp: v.timestamp,
            })
            .collect();

        // Sort by version descending (most recent first)
        versions.sort_by(|a, b| b.version.cmp(&a.version));

        Ok(match limit {
            Some(n) => versions.into_iter().take(n).collect(),
            None => versions,
        })
    }

    async fn compact(&self) -> Result<CompactStats> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let metrics = table
            .optimize(lancedb::table::OptimizeAction::Compact {
                options: lancedb::table::CompactionOptions::default(),
                remap_options: None,
            })
            .await
            .context("Failed to compact table")?;

        Ok(CompactStats {
            files_merged: metrics.compaction.map(|c| c.files_removed).unwrap_or(0),
            bytes_saved: 0, // LanceDB doesn't report bytes saved
        })
    }

    async fn cleanup_versions(&self, keep_versions: usize) -> Result<CleanupStats> {
        let db = self.get_connection().await?;
        let table = db
            .open_table(TABLE_NAME)
            .execute()
            .await
            .context("Failed to open artifacts table")?;

        let metrics = table
            .optimize(lancedb::table::OptimizeAction::Prune {
                older_than: None,
                delete_unverified: Some(false),
                error_if_tagged_old_versions: None,
            })
            .await
            .context("Failed to cleanup old versions")?;

        // Note: LanceDB prune doesn't directly support keep_versions count,
        // so we use time-based pruning. The keep_versions parameter is ignored.
        let _ = keep_versions;

        let (versions_removed, bytes_freed) = match metrics.prune {
            Some(p) => (p.bytes_removed as usize, p.bytes_removed),
            None => (0, 0),
        };

        Ok(CleanupStats {
            versions_removed,
            bytes_freed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Artifact, ContentFormat};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_artifact(content: &str, embedding: Vec<f32>) -> Artifact {
        let mut artifact = Artifact::new(
            "intent".to_string(),
            content.to_string(),
            ContentFormat::Markdown,
            Some(format!("test-{}", content)),
            HashMap::new(),
            "test-model".to_string(),
        );
        artifact.embedding = Some(embedding);
        artifact
    }

    fn create_embedding(base: f32) -> Vec<f32> {
        (0..384).map(|i| base + (i as f32 * 0.001)).collect()
    }

    #[tokio::test]
    async fn new_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("subdir").join("test.lance");
        let _db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        assert!(temp_dir.path().join("subdir").exists());
    }

    #[tokio::test]
    async fn init_creates_database_directory() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();
        assert!(db_path.exists());
    }

    // TDD: Insert then get should return the artifact
    #[tokio::test]
    async fn insert_then_get_returns_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let artifact = create_test_artifact("hello world", create_embedding(0.1));
        let id = artifact.id.clone();

        db.insert(&artifact).await.unwrap();
        let retrieved = db.get(&id).await.unwrap();

        assert!(
            retrieved.is_some(),
            "Expected to retrieve inserted artifact"
        );
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.content, "hello world");
        assert_eq!(retrieved.kind, "intent");
    }

    // TDD: Insert then list should include the artifact
    #[tokio::test]
    async fn insert_then_list_includes_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let artifact = create_test_artifact("list test", create_embedding(0.2));
        let id = artifact.id.clone();

        db.insert(&artifact).await.unwrap();
        let results = db.list(SearchFilters::default()).await.unwrap();

        assert!(
            !results.is_empty(),
            "Expected list to include inserted artifact"
        );
        assert!(
            results.iter().any(|a| a.id == id),
            "Expected to find artifact by ID in list"
        );
    }

    // TDD: List with kind filter should filter results
    #[tokio::test]
    async fn list_filters_by_kind() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let intent = create_test_artifact("intent content", create_embedding(0.1));
        let intent_id = intent.id.clone();

        let mut contract = Artifact::new(
            "contract".to_string(),
            "contract content".to_string(),
            ContentFormat::Json,
            None,
            HashMap::new(),
            "test-model".to_string(),
        );
        contract.embedding = Some(create_embedding(0.2));

        db.insert(&intent).await.unwrap();
        db.insert(&contract).await.unwrap();

        let filters = SearchFilters {
            kind: Some("intent".to_string()),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();

        assert_eq!(results.len(), 1, "Expected only intent artifacts");
        assert_eq!(results[0].id, intent_id);
    }

    // TDD: Update changes artifact content
    #[tokio::test]
    async fn update_changes_artifact_content() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let mut artifact = create_test_artifact("original", create_embedding(0.1));
        let id = artifact.id.clone();

        db.insert(&artifact).await.unwrap();

        artifact.content = "updated content".to_string();
        artifact.name = Some("updated-name".to_string());
        db.update(&artifact).await.unwrap();

        let retrieved = db.get(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.content, "updated content");
        assert_eq!(retrieved.name, Some("updated-name".to_string()));
    }

    // TDD: Delete removes artifact
    #[tokio::test]
    async fn delete_removes_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let artifact = create_test_artifact("to delete", create_embedding(0.1));
        let id = artifact.id.clone();

        db.insert(&artifact).await.unwrap();
        assert!(
            db.get(&id).await.unwrap().is_some(),
            "Artifact should exist before delete"
        );

        let deleted = db.delete(&id).await.unwrap();
        assert!(deleted, "Delete should return true for existing artifact");

        let retrieved = db.get(&id).await.unwrap();
        assert!(
            retrieved.is_none(),
            "Artifact should not exist after delete"
        );
    }

    // TDD: Delete returns false for nonexistent
    #[tokio::test]
    async fn delete_returns_false_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let deleted = db.delete("nonexistent-id").await.unwrap();
        assert!(
            !deleted,
            "Delete should return false for nonexistent artifact"
        );
    }

    // TDD: Get returns None for nonexistent
    #[tokio::test]
    async fn get_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let result = db.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    // TDD: Search finds similar vectors
    #[tokio::test]
    async fn search_finds_similar_vectors() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert artifacts with different embeddings
        let similar = create_test_artifact("similar content", create_embedding(0.5));
        let similar_id = similar.id.clone();
        let dissimilar = create_test_artifact("different content", create_embedding(0.9));

        db.insert(&similar).await.unwrap();
        db.insert(&dissimilar).await.unwrap();

        // Search with query close to similar artifact
        let query_embedding = create_embedding(0.5);
        let results = db
            .search(&query_embedding, SearchFilters::default())
            .await
            .unwrap();

        assert!(!results.is_empty(), "Search should return results");
        assert_eq!(
            results[0].artifact.id, similar_id,
            "Most similar artifact should be first"
        );
        assert!(results[0].score >= 0.0, "Score should be non-negative");
    }

    // TDD: Search respects limit
    #[tokio::test]
    async fn search_respects_limit() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert multiple artifacts
        for i in 0..5 {
            let artifact =
                create_test_artifact(&format!("content {}", i), create_embedding(i as f32 * 0.1));
            db.insert(&artifact).await.unwrap();
        }

        let filters = SearchFilters {
            limit: Some(2),
            ..Default::default()
        };
        let results = db.search(&create_embedding(0.0), filters).await.unwrap();

        assert_eq!(results.len(), 2, "Search should respect limit");
    }

    // TDD: List respects limit
    #[tokio::test]
    async fn list_respects_limit() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        for i in 0..5 {
            let artifact =
                create_test_artifact(&format!("content {}", i), create_embedding(i as f32 * 0.1));
            db.insert(&artifact).await.unwrap();
        }

        let filters = SearchFilters {
            limit: Some(3),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();

        assert_eq!(results.len(), 3, "List should respect limit");
    }

    // TDD: Empty database returns empty results
    #[tokio::test]
    async fn empty_database_list_returns_empty() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let results = db.list(SearchFilters::default()).await.unwrap();
        assert!(results.is_empty());
    }

    // TDD: Insert multiple artifacts
    #[tokio::test]
    async fn insert_multiple_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        for i in 0..3 {
            let artifact =
                create_test_artifact(&format!("content {}", i), create_embedding(i as f32 * 0.1));
            db.insert(&artifact).await.unwrap();
        }

        let results = db.list(SearchFilters::default()).await.unwrap();
        assert_eq!(results.len(), 3, "Should have 3 artifacts");
    }

    // TDD: List filters by after timestamp
    #[tokio::test]
    async fn list_filters_by_after_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert an artifact
        let artifact = create_test_artifact("old content", create_embedding(0.1));
        db.insert(&artifact).await.unwrap();

        // Query with after in the future - should return empty
        let future_time = chrono::Utc::now() + chrono::Duration::hours(1);
        let filters = SearchFilters {
            after: Some(future_time),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();
        assert!(
            results.is_empty(),
            "No artifacts should be newer than future time"
        );

        // Query with after in the past - should return the artifact
        let past_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let filters = SearchFilters {
            after: Some(past_time),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();
        assert_eq!(results.len(), 1, "Artifact should be newer than past time");
    }

    // TDD: List filters by before timestamp
    #[tokio::test]
    async fn list_filters_by_before_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert an artifact
        let artifact = create_test_artifact("old content", create_embedding(0.1));
        db.insert(&artifact).await.unwrap();

        // Query with before in the past - should return empty
        let past_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let filters = SearchFilters {
            before: Some(past_time),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();
        assert!(
            results.is_empty(),
            "No artifacts should be older than past time"
        );

        // Query with before in the future - should return the artifact
        let future_time = chrono::Utc::now() + chrono::Duration::hours(1);
        let filters = SearchFilters {
            before: Some(future_time),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();
        assert_eq!(
            results.len(),
            1,
            "Artifact should be older than future time"
        );
    }

    // TDD: List filters by time range
    #[tokio::test]
    async fn list_filters_by_time_range() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert an artifact
        let artifact = create_test_artifact("content", create_embedding(0.1));
        db.insert(&artifact).await.unwrap();

        // Query with after and before surrounding the artifact time
        let past_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let future_time = chrono::Utc::now() + chrono::Duration::hours(1);
        let filters = SearchFilters {
            after: Some(past_time),
            before: Some(future_time),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();
        assert_eq!(results.len(), 1, "Artifact should be in time range");

        // Query with time range excluding the artifact
        let far_past = chrono::Utc::now() - chrono::Duration::hours(2);
        let near_past = chrono::Utc::now() - chrono::Duration::hours(1);
        let filters = SearchFilters {
            after: Some(far_past),
            before: Some(near_past),
            ..Default::default()
        };
        let results = db.list(filters).await.unwrap();
        assert!(results.is_empty(), "Artifact should not be in time range");
    }

    // TDD: Artifact metadata is preserved
    #[tokio::test]
    async fn artifact_metadata_is_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("domain".to_string(), "auth".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let mut artifact = create_test_artifact("with metadata", create_embedding(0.1));
        artifact.metadata = metadata.clone();
        let id = artifact.id.clone();

        db.insert(&artifact).await.unwrap();
        let retrieved = db.get(&id).await.unwrap().unwrap();

        assert_eq!(retrieved.metadata.get("domain"), Some(&"auth".to_string()));
        assert_eq!(
            retrieved.metadata.get("priority"),
            Some(&"high".to_string())
        );
    }

    // TDD: version() returns a number after operations
    #[tokio::test]
    async fn version_returns_number_after_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let initial_version = db.version().await.unwrap();
        assert!(initial_version > 0, "Initial version should be positive");

        // Insert an artifact
        let artifact = create_test_artifact("version test", create_embedding(0.1));
        db.insert(&artifact).await.unwrap();

        let new_version = db.version().await.unwrap();
        assert!(
            new_version >= initial_version,
            "Version should increase or stay same after insert"
        );
    }

    // TDD: get_at_version() retrieves historical state
    #[tokio::test]
    async fn get_at_version_retrieves_historical_state() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert first artifact and record version
        let artifact1 = create_test_artifact("original content", create_embedding(0.1));
        let id = artifact1.id.clone();
        db.insert(&artifact1).await.unwrap();
        let version_after_insert = db.version().await.unwrap();

        // Update the artifact
        let mut artifact2 = artifact1.clone();
        artifact2.content = "updated content".to_string();
        db.update(&artifact2).await.unwrap();

        // Current state should show updated content
        let current = db.get(&id).await.unwrap().unwrap();
        assert_eq!(current.content, "updated content");

        // Historical state should show original content
        let historical = db.get_at_version(&id, version_after_insert).await.unwrap();
        assert!(historical.is_some(), "Should find artifact at old version");
        assert_eq!(
            historical.unwrap().content,
            "original content",
            "Historical version should have original content"
        );
    }

    // TDD: compact() completes without error
    #[tokio::test]
    async fn compact_completes_without_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert some artifacts to have data to compact
        for i in 0..3 {
            let artifact =
                create_test_artifact(&format!("compact test {}", i), create_embedding(i as f32));
            db.insert(&artifact).await.unwrap();
        }

        // Compact should complete without error
        let stats = db.compact().await.unwrap();
        // Verify stats struct is populated (values depend on internal state)
        let _ = stats.files_merged;
        let _ = stats.bytes_saved;
    }

    // TDD: list_versions returns at least current version
    #[tokio::test]
    async fn list_versions_returns_versions() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        let versions = db.list_versions(None).await.unwrap();
        assert!(!versions.is_empty(), "Should have at least one version");

        // With limit
        let limited = db.list_versions(Some(1)).await.unwrap();
        assert!(limited.len() <= 1, "Should respect limit");
    }

    // TDD: cleanup_versions completes without error
    #[tokio::test]
    async fn cleanup_versions_completes_without_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db = LanceDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        db.init().await.unwrap();

        // Insert and update to create versions
        let artifact = create_test_artifact("cleanup test", create_embedding(0.1));
        let id = artifact.id.clone();
        db.insert(&artifact).await.unwrap();

        let mut updated = artifact.clone();
        updated.content = "updated for cleanup".to_string();
        db.update(&updated).await.unwrap();

        // Cleanup should complete without error
        let stats = db.cleanup_versions(1).await.unwrap();
        // Verify stats struct is populated (values depend on internal state)
        let _ = stats.versions_removed;
        let _ = stats.bytes_freed;

        // Data should still be accessible
        let retrieved = db.get(&id).await.unwrap();
        assert!(retrieved.is_some(), "Data should still exist after cleanup");
    }
}
