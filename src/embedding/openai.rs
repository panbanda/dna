use super::provider::EmbeddingProvider;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// OpenAI embedding provider
pub struct OpenAIEmbedding {
    model_id: String,
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIEmbedding {
    /// Create a new OpenAI embedding provider
    pub fn new(model_id: &str, api_key_env: &str, base_url: Option<&str>) -> Result<Self> {
        let api_key = std::env::var(api_key_env)
            .with_context(|| format!("Environment variable {} not set", api_key_env))?;

        Ok(Self {
            model_id: model_id.to_string(),
            api_key,
            base_url: base_url
                .unwrap_or(DEFAULT_BASE_URL)
                .trim_end_matches('/')
                .to_string(),
            client: reqwest::Client::new(),
        })
    }

    /// Get dimensions for known OpenAI models
    fn get_dimensions(&self) -> usize {
        match self.model_id.as_str() {
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            "text-embedding-ada-002" => 1536,
            _ => 1536, // default
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenAIEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let request = EmbeddingRequest {
            input: texts.iter().map(|s| s.to_string()).collect(),
            model: self.model_id.clone(),
        };

        let url = format!("{}/embeddings", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send OpenAI API request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API error {}: {}", status, text));
        }

        let result: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn dimensions(&self) -> usize {
        self.get_dimensions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_env() {
        std::env::set_var("TEST_OPENAI_KEY", "test-key");
    }

    #[test]
    fn dimensions_text_embedding_3_small() {
        setup_env();
        let provider =
            OpenAIEmbedding::new("text-embedding-3-small", "TEST_OPENAI_KEY", None).unwrap();
        assert_eq!(provider.dimensions(), 1536);
    }

    #[test]
    fn dimensions_text_embedding_3_large() {
        setup_env();
        let provider =
            OpenAIEmbedding::new("text-embedding-3-large", "TEST_OPENAI_KEY", None).unwrap();
        assert_eq!(provider.dimensions(), 3072);
    }

    #[test]
    fn dimensions_text_embedding_ada_002() {
        setup_env();
        let provider =
            OpenAIEmbedding::new("text-embedding-ada-002", "TEST_OPENAI_KEY", None).unwrap();
        assert_eq!(provider.dimensions(), 1536);
    }

    #[test]
    fn dimensions_unknown_model_defaults_to_1536() {
        setup_env();
        let provider = OpenAIEmbedding::new("unknown-model", "TEST_OPENAI_KEY", None).unwrap();
        assert_eq!(provider.dimensions(), 1536);
    }

    #[test]
    fn custom_base_url_strips_trailing_slash() {
        setup_env();
        let provider = OpenAIEmbedding::new(
            "text-embedding-3-small",
            "TEST_OPENAI_KEY",
            Some("https://custom.api.example.com/v1/"),
        )
        .unwrap();
        assert_eq!(provider.base_url, "https://custom.api.example.com/v1");
    }

    #[test]
    fn default_base_url() {
        setup_env();
        let provider =
            OpenAIEmbedding::new("text-embedding-3-small", "TEST_OPENAI_KEY", None).unwrap();
        assert_eq!(provider.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn model_id_returns_configured_value() {
        setup_env();
        let provider = OpenAIEmbedding::new("my-model", "TEST_OPENAI_KEY", None).unwrap();
        assert_eq!(provider.model_id(), "my-model");
    }

    #[test]
    fn missing_api_key_env_returns_error() {
        let result = OpenAIEmbedding::new("model", "NONEXISTENT_KEY_12345", None);
        assert!(result.is_err());
    }
}
