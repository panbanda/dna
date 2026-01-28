//! Service-level usage example for DNA
//!
//! This example demonstrates using the higher-level ArtifactService and SearchService
//! which handle embedding generation automatically.
//!
//! Run with: cargo run --example service_usage

use anyhow::Result;
use dna::db::lance::LanceDatabase;
use dna::embedding::EmbeddingProvider;
use dna::services::{ArtifactService, ArtifactType, ContentFormat, SearchFilters, SearchService};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

struct MockEmbedding {
    dimensions: usize,
}

impl MockEmbedding {
    fn new() -> Self {
        Self { dimensions: 384 }
    }

    fn text_to_embedding(&self, text: &str) -> Vec<f32> {
        let hash = text.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        let base = (hash % 1000) as f32 / 1000.0;
        (0..self.dimensions)
            .map(|i| base + (i as f32 * 0.001))
            .collect()
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for MockEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.text_to_embedding(text))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| self.text_to_embedding(t)).collect())
    }

    fn model_id(&self) -> &str {
        "mock-embedding-v1"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("DNA Service Usage Example");
    println!("=========================\n");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("service_example.lance");

    println!("1. Initializing services...");
    let db = Arc::new(LanceDatabase::new(db_path.to_str().unwrap()).await?);
    db.init().await?;

    let embedding: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbedding::new());
    let artifact_service = ArtifactService::new(db.clone(), embedding.clone());
    let search_service = SearchService::new(db.clone(), embedding.clone());
    println!("   Services initialized!\n");

    println!("2. Adding artifacts via ArtifactService...");

    let intent = artifact_service
        .add(
            ArtifactType::Intent,
            "Users should be able to create accounts with email verification.".to_string(),
            ContentFormat::Markdown,
            Some("Account Creation Intent".to_string()),
            HashMap::from([("domain".to_string(), "user-management".to_string())]),
        )
        .await?;
    println!(
        "   Added Intent: {} (ID: {})",
        intent.name.as_ref().unwrap(),
        intent.id
    );

    let contract = artifact_service
        .add(
            ArtifactType::Contract,
            r#"POST /api/users
Request: { "email": string, "password": string }
Response: { "id": string, "email": string, "verified": boolean }"#
                .to_string(),
            ContentFormat::Markdown,
            Some("User Registration API".to_string()),
            HashMap::from([
                ("domain".to_string(), "user-management".to_string()),
                ("version".to_string(), "v1".to_string()),
            ]),
        )
        .await?;
    println!(
        "   Added Contract: {} (ID: {})",
        contract.name.as_ref().unwrap(),
        contract.id
    );

    let invariant = artifact_service
        .add(
            ArtifactType::Invariant,
            "Email addresses must be unique across all user accounts.".to_string(),
            ContentFormat::Markdown,
            Some("Email Uniqueness".to_string()),
            HashMap::from([("domain".to_string(), "user-management".to_string())]),
        )
        .await?;
    println!(
        "   Added Invariant: {} (ID: {})\n",
        invariant.name.as_ref().unwrap(),
        invariant.id
    );

    println!("3. Semantic search for 'email verification'...");
    let results = search_service
        .search(
            "email verification process",
            SearchFilters {
                limit: Some(5),
                ..Default::default()
            },
        )
        .await?;

    for (i, result) in results.iter().enumerate() {
        println!(
            "   {}. [score: {:.4}] [{}] {}",
            i + 1,
            result.score,
            result.artifact.artifact_type,
            result
                .artifact
                .name
                .as_deref()
                .unwrap_or(&result.artifact.id)
        );
    }
    println!();

    println!("4. Updating artifact content...");
    let updated = artifact_service
        .update(
            &intent.id,
            Some("Users should be able to create accounts with email verification. The verification email must be sent within 30 seconds of registration.".to_string()),
            None,
            Some(HashMap::from([("priority".to_string(), "high".to_string())])),
        )
        .await?;
    println!("   Updated: {}", updated.name.as_ref().unwrap());
    println!("   New metadata: {:?}\n", updated.metadata);

    println!("5. Listing artifacts by domain...");
    let all = artifact_service.list(SearchFilters::default()).await?;
    println!("   Total artifacts: {}\n", all.len());

    println!("6. Removing an artifact...");
    let removed = artifact_service.remove(&contract.id).await?;
    println!("   Removed {}: {}\n", contract.id, removed);

    let final_list = artifact_service.list(SearchFilters::default()).await?;
    println!("Final artifact count: {}", final_list.len());

    println!("\nService example completed successfully!");
    Ok(())
}
