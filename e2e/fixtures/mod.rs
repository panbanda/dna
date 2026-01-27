/// Test fixtures and helpers for DNA CLI tests
///
/// This module provides reusable test data, mock objects, and helper functions
/// to facilitate comprehensive testing across the DNA codebase.

use dna::services::{Artifact, ArtifactType, ContentFormat};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test artifact builder for creating fixtures
pub struct TestArtifactBuilder {
    artifact_type: ArtifactType,
    name: Option<String>,
    content: String,
    format: ContentFormat,
    metadata: HashMap<String, String>,
}

impl TestArtifactBuilder {
    pub fn new(artifact_type: ArtifactType) -> Self {
        Self {
            artifact_type,
            name: None,
            content: String::from("Test artifact content"),
            format: ContentFormat::Markdown,
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn with_format(mut self, format: ContentFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Artifact {
        Artifact::new(
            self.artifact_type,
            self.content,
            self.format,
            self.name,
            self.metadata,
            "test-model".to_string(),
        )
    }
}

/// Test environment for isolated DNA testing
pub struct TestEnv {
    pub temp_dir: TempDir,
    pub dna_dir: PathBuf,
    pub db_dir: PathBuf,
    pub config_path: PathBuf,
}

impl TestEnv {
    /// Create a new isolated test environment
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let dna_dir = temp_dir.path().join(".dna");
        let db_dir = dna_dir.join("db");
        let config_path = dna_dir.join("config.toml");

        std::fs::create_dir_all(&db_dir)?;

        Ok(Self {
            temp_dir,
            dna_dir,
            db_dir,
            config_path,
        })
    }

    /// Get the root path of the test environment
    pub fn root(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Initialize DNA in this test environment
    pub fn init_dna(&self) -> anyhow::Result<()> {
        self.write_default_config()
    }

    /// Write default configuration
    pub fn write_default_config(&self) -> anyhow::Result<()> {
        let config = r#"
[model]
provider = "local"
name = "BAAI/bge-small-en-v1.5"
"#;
        std::fs::write(&self.config_path, config)?;
        Ok(())
    }

    /// Write custom configuration
    pub fn write_config(&self, config: &str) -> anyhow::Result<()> {
        std::fs::write(&self.config_path, config)?;
        Ok(())
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new().expect("Failed to create test environment")
    }
}

/// Sample artifact fixtures for testing
pub mod samples {
    use super::*;

    pub fn intent_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Intent)
            .with_name("user-authentication")
            .with_content("The system authenticates users via email and password")
            .with_metadata("domain", "auth")
            .with_metadata("priority", "high")
            .build()
    }

    pub fn invariant_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Invariant)
            .with_name("valid-payment")
            .with_content("Users must have a valid payment method before completing checkout")
            .with_metadata("domain", "checkout")
            .with_metadata("priority", "critical")
            .build()
    }

    pub fn contract_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Contract)
            .with_name("payment-api")
            .with_content("POST /api/payments returns 201 on success with payment ID")
            .with_format(ContentFormat::OpenApi)
            .with_metadata("service", "payment-service")
            .build()
    }

    pub fn algorithm_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Algorithm)
            .with_name("price-calculation")
            .with_content("Price = base_price * quantity * (1 - discount_rate)")
            .with_metadata("domain", "pricing")
            .build()
    }

    pub fn evaluation_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Evaluation)
            .with_name("checkout-success")
            .with_content("Given valid cart, when checkout, then order created")
            .with_metadata("domain", "checkout")
            .build()
    }

    pub fn pace_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Pace)
            .with_name("payment-api-stability")
            .with_content("Payment API contracts require 2-week deprecation notice")
            .with_metadata("service", "payment-service")
            .build()
    }

    pub fn monitor_artifact() -> Artifact {
        TestArtifactBuilder::new(ArtifactType::Monitor)
            .with_name("api-latency")
            .with_content("P99 API latency < 200ms")
            .with_metadata("service", "all")
            .with_metadata("slo", "true")
            .build()
    }

    pub fn all_artifact_types() -> Vec<Artifact> {
        vec![
            intent_artifact(),
            invariant_artifact(),
            contract_artifact(),
            algorithm_artifact(),
            evaluation_artifact(),
            pace_artifact(),
            monitor_artifact(),
        ]
    }
}

/// Mock embedding provider for testing
pub struct MockEmbeddingProvider {
    pub model_id: String,
    pub dimensions: usize,
}

impl MockEmbeddingProvider {
    pub fn new() -> Self {
        Self {
            model_id: "mock-model".to_string(),
            dimensions: 384,
        }
    }

    /// Generate deterministic fake embeddings for testing
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        (0..self.dimensions)
            .map(|i| ((hash.wrapping_add(i as u32) % 1000) as f32) / 1000.0)
            .collect()
    }

    pub fn embed_batch(&self, texts: &[&str]) -> Vec<Vec<f32>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }
}

impl Default for MockEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Assertion helpers
pub mod assertions {
    /// Assert that an ID follows the DNA format (10 chars, lowercase alphanumeric)
    pub fn assert_valid_id(id: &str) {
        assert_eq!(id.len(), 10, "ID should be exactly 10 characters");
        assert!(
            id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "ID should only contain lowercase alphanumeric characters"
        );
        const ALPHABET: &str = "23456789abcdefghjkmnpqrstuvwxyz";
        assert!(
            id.chars().all(|c| ALPHABET.contains(c)),
            "ID contains invalid characters (ambiguous chars not allowed)"
        );
    }

    /// Assert that a slug is valid (lowercase, hyphens, alphanumeric)
    pub fn assert_valid_slug(slug: &str) {
        assert!(
            slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "Slug should only contain lowercase alphanumeric and hyphens"
        );
        if !slug.is_empty() {
            assert!(!slug.starts_with('-') && !slug.ends_with('-'), "Slug should not start or end with hyphen");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_builder() {
        let artifact = TestArtifactBuilder::new(ArtifactType::Intent)
            .with_name("test")
            .with_content("Test content")
            .build();

        assert_eq!(artifact.artifact_type, ArtifactType::Intent);
        assert_eq!(artifact.name.unwrap(), "test");
        assert_eq!(artifact.content, "Test content");
    }

    #[test]
    fn test_env_creation() {
        let env = TestEnv::new().unwrap();
        assert!(env.root().exists());
        assert!(env.dna_dir.ends_with(".dna"));
    }

    #[test]
    fn test_mock_embedding() {
        let provider = MockEmbeddingProvider::new();
        let embedding = provider.embed("test");
        assert_eq!(embedding.len(), 384);

        let embedding2 = provider.embed("test");
        assert_eq!(embedding, embedding2);
    }

    #[test]
    fn test_valid_id_assertion() {
        assertions::assert_valid_id("k7v3m9xnp2");
        assertions::assert_valid_id("2abc3def4g");
    }

    #[test]
    #[should_panic(expected = "ID should be exactly 10 characters")]
    fn test_invalid_id_length() {
        assertions::assert_valid_id("short");
    }

    #[test]
    #[should_panic(expected = "ID contains invalid characters")]
    fn test_invalid_id_chars() {
        assertions::assert_valid_id("k7v3m9xnp1"); // '1' is ambiguous, not in alphabet
    }

    #[test]
    fn test_samples_all_types() {
        let artifacts = samples::all_artifact_types();
        assert_eq!(artifacts.len(), 7);
    }
}
