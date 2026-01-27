use super::provider::EmbeddingProvider;
use anyhow::Result;

/// Local embedding using Candle
pub struct LocalEmbedding {
    model_id: String,
    dimensions: usize,
}

impl LocalEmbedding {
    /// Create a new local embedding provider
    pub async fn new(model_id: &str) -> Result<Self> {
        // TODO: Implement Candle model loading
        // For now, return a placeholder
        tracing::info!("Initializing local embedding model: {}", model_id);

        // Default dimensions for bge-small-en-v1.5
        let dimensions = if model_id.contains("bge-small") {
            384
        } else {
            768
        };

        Ok(Self {
            model_id: model_id.to_string(),
            dimensions,
        })
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for LocalEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // TODO: Implement actual embedding with Candle
        // For now, return zero vector
        tracing::debug!("Embedding text with local model: {} chars", text.len());
        Ok(vec![0.0; self.dimensions])
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // TODO: Implement batch embedding
        tracing::debug!("Batch embedding {} texts", texts.len());
        Ok(texts.iter().map(|_| vec![0.0; self.dimensions]).collect())
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creation_with_bge_small() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        assert_eq!(provider.model_id(), "BAAI/bge-small-en-v1.5");
        assert_eq!(provider.dimensions(), 384);
    }

    #[tokio::test]
    async fn creation_with_other_model_defaults_to_768() {
        let provider = LocalEmbedding::new("some-other-model").await.unwrap();
        assert_eq!(provider.dimensions(), 768);
    }

    #[tokio::test]
    async fn embed_returns_correct_dimensions() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("test text").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn embed_handles_empty_text() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn embed_handles_unicode() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("Hello test test").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn embed_batch_returns_correct_count() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let texts = vec!["one", "two", "three"];
        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 3);
        for emb in embeddings {
            assert_eq!(emb.len(), 384);
        }
    }

    #[tokio::test]
    async fn embed_batch_empty_input() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let texts: Vec<&str> = vec![];
        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert!(embeddings.is_empty());
    }

    #[tokio::test]
    async fn model_id_returns_configured_value() {
        let provider = LocalEmbedding::new("custom-model").await.unwrap();
        assert_eq!(provider.model_id(), "custom-model");
    }
}
