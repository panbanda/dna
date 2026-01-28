pub mod local;
pub mod ollama;
pub mod openai;
pub mod provider;

pub use provider::EmbeddingProvider;

use crate::services::ModelConfig;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Create an embedding provider from configuration
pub async fn create_provider(config: &ModelConfig) -> Result<Arc<dyn EmbeddingProvider>> {
    match config.provider.as_str() {
        "local" => {
            let provider = local::LocalEmbedding::new(&config.name)
                .await
                .context("Failed to initialize local embedding provider")?;
            Ok(Arc::new(provider))
        },
        "openai" => {
            let api_key_env = config.api_key_env.as_deref().unwrap_or("OPENAI_API_KEY");
            let base_url = config.base_url.as_deref();
            let provider = openai::OpenAIEmbedding::new(&config.name, api_key_env, base_url)
                .context("Failed to initialize OpenAI embedding provider")?;
            Ok(Arc::new(provider))
        },
        "ollama" => {
            let base_url = config
                .base_url
                .as_deref()
                .unwrap_or("http://localhost:11434");
            let provider = ollama::OllamaEmbedding::new(&config.name, base_url);
            Ok(Arc::new(provider))
        },
        _ => Err(anyhow::anyhow!("Unknown provider: {}", config.provider)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn create_provider_local() {
        let config = ModelConfig {
            provider: "local".to_string(),
            name: "BAAI/bge-small-en-v1.5".to_string(),
            api_key_env: None,
            base_url: None,
        };
        let provider = create_provider(&config).await.unwrap();
        assert_eq!(provider.model_id(), "BAAI/bge-small-en-v1.5");
        assert_eq!(provider.dimensions(), 384);
    }

    #[tokio::test]
    async fn create_provider_ollama() {
        let config = ModelConfig {
            provider: "ollama".to_string(),
            name: "nomic-embed-text".to_string(),
            api_key_env: None,
            base_url: None,
        };
        let provider = create_provider(&config).await.unwrap();
        assert_eq!(provider.model_id(), "nomic-embed-text");
        assert_eq!(provider.dimensions(), 768);
    }

    #[tokio::test]
    async fn create_provider_ollama_custom_url() {
        let config = ModelConfig {
            provider: "ollama".to_string(),
            name: "model".to_string(),
            api_key_env: None,
            base_url: Some("http://custom:8080".to_string()),
        };
        let provider = create_provider(&config).await.unwrap();
        assert_eq!(provider.model_id(), "model");
    }

    #[tokio::test]
    async fn create_provider_unknown_returns_error() {
        let config = ModelConfig {
            provider: "unknown".to_string(),
            name: "model".to_string(),
            api_key_env: None,
            base_url: None,
        };
        let result = create_provider(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_provider_openai_without_key_returns_error() {
        let config = ModelConfig {
            provider: "openai".to_string(),
            name: "text-embedding-3-small".to_string(),
            api_key_env: Some("NONEXISTENT_KEY_12345".to_string()),
            base_url: None,
        };
        let result = create_provider(&config).await;
        assert!(result.is_err());
    }
}
