use super::provider::EmbeddingProvider;
use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Local embedding using Candle with BERT-based models
pub struct LocalEmbedding {
    model_id: String,
    model: Arc<BertModel>,
    tokenizer: Arc<Tokenizer>,
    device: Device,
    dimensions: usize,
}

impl LocalEmbedding {
    /// Create a new local embedding provider
    pub async fn new(model_id: &str) -> Result<Self> {
        tracing::info!("Initializing local embedding model: {}", model_id);

        let device = Device::Cpu;

        // Download model files from HuggingFace
        let api = Api::new().context("Failed to create HuggingFace API client")?;
        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

        tracing::debug!("Downloading model files from HuggingFace...");

        let config_path = repo
            .get("config.json")
            .context("Failed to download config.json")?;
        let tokenizer_path = repo
            .get("tokenizer.json")
            .context("Failed to download tokenizer.json")?;
        let weights_path = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))
            .context("Failed to download model weights")?;

        // Load config
        let config_str =
            std::fs::read_to_string(&config_path).context("Failed to read config.json")?;
        let config: Config =
            serde_json::from_str(&config_str).context("Failed to parse config.json")?;
        let dimensions = config.hidden_size;

        // Load tokenizer
        let tokenizer =
            Tokenizer::from_file(&tokenizer_path).map_err(|e| anyhow::anyhow!("{}", e))?;

        // Load model weights
        let vb = if weights_path
            .extension()
            .is_some_and(|ext| ext == "safetensors")
        {
            unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)
                    .context("Failed to load safetensors")?
            }
        } else {
            VarBuilder::from_pth(&weights_path, DType::F32, &device)
                .context("Failed to load pytorch model")?
        };

        let model = BertModel::load(vb, &config).context("Failed to load BERT model")?;

        tracing::info!("Loaded model {} with {} dimensions", model_id, dimensions);

        Ok(Self {
            model_id: model_id.to_string(),
            model: Arc::new(model),
            tokenizer: Arc::new(tokenizer),
            device,
            dimensions,
        })
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for LocalEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        tracing::debug!("Batch embedding {} texts", texts.len());

        let model = Arc::clone(&self.model);
        let tokenizer = Arc::clone(&self.tokenizer);
        let device = self.device.clone();

        // Move computation to blocking task since it's CPU-intensive
        let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();

        let result = tokio::task::spawn_blocking(move || -> Result<Vec<Vec<f32>>> {
            // Tokenize all texts
            let encodings = tokenizer
                .encode_batch(texts_owned.clone(), true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

            // Find max length for padding
            let max_len = encodings
                .iter()
                .map(|e| e.get_ids().len())
                .max()
                .unwrap_or(0);

            // Build padded input tensors
            let mut all_input_ids = Vec::new();
            let mut all_attention_masks = Vec::new();
            let mut all_token_type_ids = Vec::new();

            for encoding in &encodings {
                let ids = encoding.get_ids();
                let attention = encoding.get_attention_mask();
                let type_ids = encoding.get_type_ids();

                // Pad to max_len
                let mut padded_ids = ids.to_vec();
                let mut padded_attention = attention.to_vec();
                let mut padded_type_ids = type_ids.to_vec();

                padded_ids.resize(max_len, 0);
                padded_attention.resize(max_len, 0);
                padded_type_ids.resize(max_len, 0);

                all_input_ids.extend(padded_ids);
                all_attention_masks.extend(padded_attention);
                all_token_type_ids.extend(padded_type_ids);
            }

            let batch_size = texts_owned.len();

            let input_ids = Tensor::from_vec(all_input_ids, (batch_size, max_len), &device)?
                .to_dtype(DType::U32)?;
            let attention_mask =
                Tensor::from_vec(all_attention_masks, (batch_size, max_len), &device)?
                    .to_dtype(DType::U32)?;
            let token_type_ids =
                Tensor::from_vec(all_token_type_ids, (batch_size, max_len), &device)?
                    .to_dtype(DType::U32)?;

            // Run model forward pass
            let embeddings = model.forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

            // Mean pooling over sequence dimension
            let attention_mask_f32 = attention_mask.to_dtype(DType::F32)?;
            let mask_expanded = attention_mask_f32
                .unsqueeze(2)?
                .broadcast_as(embeddings.shape())?;

            let masked = embeddings.mul(&mask_expanded)?;
            let sum = masked.sum(1)?;
            let count = mask_expanded.sum(1)?.clamp(1e-9, f64::MAX)?;
            let pooled = sum.broadcast_div(&count)?;

            // L2 normalize
            let norm = pooled
                .sqr()?
                .sum_keepdim(1)?
                .sqrt()?
                .clamp(1e-12, f64::MAX)?;
            let normalized = pooled.broadcast_div(&norm)?;

            // Convert to Vec<Vec<f32>>
            let flat: Vec<f32> = normalized.flatten_all()?.to_vec1()?;
            let dim = normalized.dim(1)?;

            let result: Vec<Vec<f32>> = flat.chunks(dim).map(|chunk| chunk.to_vec()).collect();

            Ok(result)
        })
        .await
        .context("Embedding task panicked")??;

        Ok(result)
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

    // Integration tests - require network access to download models from HuggingFace.
    // Run with: cargo test --package dna -- --ignored

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn creation_with_bge_small() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        assert_eq!(provider.model_id(), "BAAI/bge-small-en-v1.5");
        assert_eq!(provider.dimensions(), 384);
    }

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn embed_returns_correct_dimensions() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("test text").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn embed_handles_empty_text() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn embed_handles_unicode() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    #[ignore = "requires network access to download model"]
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
    #[ignore = "requires network access to download model"]
    async fn embed_batch_empty_input() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let texts: Vec<&str> = vec![];
        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert!(embeddings.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn embeddings_are_normalized() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();
        let embedding = provider.embed("test text").await.unwrap();

        // Check L2 norm is approximately 1.0
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Expected normalized embedding with norm ~1.0, got {}",
            norm
        );
    }

    #[tokio::test]
    #[ignore = "requires network access to download model"]
    async fn similar_texts_have_similar_embeddings() {
        let provider = LocalEmbedding::new("BAAI/bge-small-en-v1.5").await.unwrap();

        let emb1 = provider.embed("The quick brown fox").await.unwrap();
        let emb2 = provider.embed("A fast brown fox").await.unwrap();
        let emb3 = provider.embed("Database query optimization").await.unwrap();

        // Cosine similarity (embeddings are already normalized)
        let sim_12: f32 = emb1.iter().zip(&emb2).map(|(a, b)| a * b).sum();
        let sim_13: f32 = emb1.iter().zip(&emb3).map(|(a, b)| a * b).sum();

        assert!(
            sim_12 > sim_13,
            "Similar texts should have higher similarity: {} vs {}",
            sim_12,
            sim_13
        );
    }
}
