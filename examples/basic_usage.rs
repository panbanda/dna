//! Basic usage example for DNA - Truth artifact management with vector search
//!
//! This example demonstrates how to:
//! 1. Initialize a LanceDB database
//! 2. Add artifacts with embeddings
//! 3. Search for similar artifacts
//! 4. List and filter artifacts
//!
//! Run with: cargo run --example basic_usage

use anyhow::Result;
use dna::db::{lance::LanceDatabase, Database};
use dna::services::{Artifact, ArtifactType, ContentFormat, SearchFilters};
use std::collections::HashMap;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("DNA Basic Usage Example");
    println!("=======================\n");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("example.lance");

    println!("1. Initializing LanceDB at {:?}", db_path);
    let db = LanceDatabase::new(db_path.to_str().unwrap()).await?;
    db.init().await?;
    println!("   Database initialized successfully!\n");

    println!("2. Adding sample artifacts...");
    let artifacts = create_sample_artifacts();
    for artifact in &artifacts {
        db.insert(artifact).await?;
        println!(
            "   Added: [{}] {}",
            artifact.artifact_type,
            artifact.name.as_deref().unwrap_or("unnamed"),
        );
    }
    println!();

    println!("3. Listing all artifacts...");
    let all_artifacts = db.list(SearchFilters::default()).await?;
    println!("   Found {} artifacts\n", all_artifacts.len());

    println!("4. Filtering by artifact type (Intent only)...");
    let intent_filter = SearchFilters {
        artifact_type: Some(ArtifactType::Intent),
        ..Default::default()
    };
    let intents = db.list(intent_filter).await?;
    println!("   Found {} Intent artifacts\n", intents.len());

    println!("5. Semantic search for 'authentication'...");
    let query_embedding = create_auth_like_embedding();
    let search_results = db
        .search(
            &query_embedding,
            SearchFilters {
                limit: Some(3),
                ..Default::default()
            },
        )
        .await?;

    for (i, result) in search_results.iter().enumerate() {
        println!(
            "   {}. [score: {:.4}] {} - {}",
            i + 1,
            result.score,
            result.artifact.artifact_type,
            result.artifact.name.as_deref().unwrap_or("unnamed")
        );
    }
    println!();

    println!("6. Getting artifact by ID...");
    if let Some(first) = all_artifacts.first() {
        let retrieved = db.get(&first.id).await?;
        if let Some(artifact) = retrieved {
            println!(
                "   Retrieved: {}",
                artifact.name.as_deref().unwrap_or(&artifact.id)
            );
        }
    }
    println!();

    println!("7. Updating an artifact...");
    if let Some(mut first) = all_artifacts.into_iter().next() {
        let old_content = first.content.clone();
        first.content = format!("{}\n\nUpdated: This artifact was modified.", old_content);
        db.update(&first).await?;
        println!("   Updated artifact: {}", first.id);
    }
    println!();

    println!("8. Deleting an artifact...");
    let to_delete = artifacts.last().unwrap();
    let deleted = db.delete(&to_delete.id).await?;
    println!(
        "   Deleted artifact {}: {}",
        to_delete.id,
        if deleted { "success" } else { "not found" }
    );
    println!();

    let final_count = db.list(SearchFilters::default()).await?.len();
    println!("Final artifact count: {}", final_count);

    println!("\nExample completed successfully!");
    Ok(())
}

fn create_sample_artifacts() -> Vec<Artifact> {
    vec![
        create_artifact(
            ArtifactType::Intent,
            "User Authentication Intent",
            "Users must be able to securely authenticate using email and password.",
            create_auth_like_embedding(),
            HashMap::from([("domain".to_string(), "auth".to_string())]),
        ),
        create_artifact(
            ArtifactType::Invariant,
            "Password Security Invariant",
            "Passwords must be hashed using bcrypt with a minimum cost factor of 12.",
            create_security_like_embedding(),
            HashMap::from([("domain".to_string(), "auth".to_string())]),
        ),
        create_artifact(
            ArtifactType::Contract,
            "Login API Contract",
            "POST /api/v1/auth/login accepts {email, password} and returns {token, user}.",
            create_api_like_embedding(),
            HashMap::from([
                ("domain".to_string(), "auth".to_string()),
                ("version".to_string(), "v1".to_string()),
            ]),
        ),
    ]
}

fn create_artifact(
    artifact_type: ArtifactType,
    name: &str,
    content: &str,
    embedding: Vec<f32>,
    metadata: HashMap<String, String>,
) -> Artifact {
    let mut artifact = Artifact::new(
        artifact_type,
        content.to_string(),
        ContentFormat::Markdown,
        Some(name.to_string()),
        metadata,
        "example-embedding".to_string(),
    );
    artifact.embedding = Some(embedding);
    artifact
}

fn create_auth_like_embedding() -> Vec<f32> {
    (0..384).map(|i| 0.5 + (i as f32 * 0.001)).collect()
}

fn create_security_like_embedding() -> Vec<f32> {
    (0..384).map(|i| 0.6 + (i as f32 * 0.001)).collect()
}

fn create_api_like_embedding() -> Vec<f32> {
    (0..384).map(|i| 0.3 + (i as f32 * 0.001)).collect()
}
