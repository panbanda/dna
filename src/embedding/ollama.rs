use super::provider::EmbeddingProvider;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Ollama embedding provider
pub struct OllamaEmbedding {
    model_id: String,
    base_url: String,
    client: reqwest::Client,
}

impl OllamaEmbedding {
    /// Create a new Ollama embedding provider
    pub fn new(model_id: &str, base_url: &str) -> Self {
        Self {
            model_id: model_id.to_string(),
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Estimate dimensions (Ollama doesn't provide this in the response)
    fn estimate_dimensions(&self) -> usize {
        // Common Ollama embedding models
        match self.model_id.as_str() {
            "nomic-embed-text" => 768,
            "mxbai-embed-large" => 1024,
            "all-minilm" => 384,
            _ => 768, // default
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OllamaEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = OllamaEmbedRequest {
            model: self.model_id.clone(),
            prompt: text.to_string(),
        };

        let url = format!("{}/api/embeddings", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send Ollama API request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API error {}: {}", status, text));
        }

        let result: OllamaEmbedResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(result.embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Ollama doesn't have native batch support, so we do sequential requests
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn dimensions(&self) -> usize {
        self.estimate_dimensions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_stores_model_id() {
        let provider = OllamaEmbedding::new("nomic-embed-text", "http://localhost:11434");
        assert_eq!(provider.model_id(), "nomic-embed-text");
    }

    #[test]
    fn creation_stores_base_url() {
        let provider = OllamaEmbedding::new("model", "http://custom:8080");
        assert_eq!(provider.base_url, "http://custom:8080");
    }

    #[test]
    fn dimensions_nomic_embed_text() {
        let provider = OllamaEmbedding::new("nomic-embed-text", "http://localhost:11434");
        assert_eq!(provider.dimensions(), 768);
    }

    #[test]
    fn dimensions_mxbai_embed_large() {
        let provider = OllamaEmbedding::new("mxbai-embed-large", "http://localhost:11434");
        assert_eq!(provider.dimensions(), 1024);
    }

    #[test]
    fn dimensions_all_minilm() {
        let provider = OllamaEmbedding::new("all-minilm", "http://localhost:11434");
        assert_eq!(provider.dimensions(), 384);
    }

    #[test]
    fn dimensions_unknown_model_defaults_to_768() {
        let provider = OllamaEmbedding::new("unknown-model", "http://localhost:11434");
        assert_eq!(provider.dimensions(), 768);
    }
}
