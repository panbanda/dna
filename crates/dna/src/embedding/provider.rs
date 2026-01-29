use anyhow::Result;

/// Trait for embedding providers
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Get the model identifier
    fn model_id(&self) -> &str;

    /// Get embedding dimensions
    fn dimensions(&self) -> usize;
}
