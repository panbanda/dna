use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Artifact type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
    Intent,
    Invariant,
    Contract,
    Algorithm,
    Evaluation,
    Pace,
    Monitor,
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ArtifactType::Intent => "intent",
            ArtifactType::Invariant => "invariant",
            ArtifactType::Contract => "contract",
            ArtifactType::Algorithm => "algorithm",
            ArtifactType::Evaluation => "evaluation",
            ArtifactType::Pace => "pace",
            ArtifactType::Monitor => "monitor",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ArtifactType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "intent" => Ok(ArtifactType::Intent),
            "invariant" => Ok(ArtifactType::Invariant),
            "contract" => Ok(ArtifactType::Contract),
            "algorithm" => Ok(ArtifactType::Algorithm),
            "evaluation" => Ok(ArtifactType::Evaluation),
            "pace" => Ok(ArtifactType::Pace),
            "monitor" => Ok(ArtifactType::Monitor),
            _ => Err(anyhow::anyhow!("Invalid artifact type: {}", s)),
        }
    }
}

/// Content format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
    Markdown,
    Yaml,
    Json,
    OpenApi,
    Text,
}

impl std::fmt::Display for ContentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ContentFormat::Markdown => "markdown",
            ContentFormat::Yaml => "yaml",
            ContentFormat::Json => "json",
            ContentFormat::OpenApi => "openapi",
            ContentFormat::Text => "text",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ContentFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(ContentFormat::Markdown),
            "yaml" | "yml" => Ok(ContentFormat::Yaml),
            "json" => Ok(ContentFormat::Json),
            "openapi" => Ok(ContentFormat::OpenApi),
            "text" | "txt" => Ok(ContentFormat::Text),
            _ => Err(anyhow::anyhow!("Invalid content format: {}", s)),
        }
    }
}

/// Core artifact structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,
    pub name: Option<String>,
    pub content: String,
    pub format: ContentFormat,
    pub metadata: HashMap<String, String>,
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
    pub embedding_model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Artifact {
    /// Generate a new 10-character ID using reduced alphabet
    pub fn generate_id() -> String {
        const ALPHABET: &[char] = &[
            '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j',
            'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        nanoid::nanoid!(10, ALPHABET)
    }

    /// Create a new artifact
    pub fn new(
        artifact_type: ArtifactType,
        content: String,
        format: ContentFormat,
        name: Option<String>,
        metadata: HashMap<String, String>,
        embedding_model: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            artifact_type,
            name,
            content,
            format,
            metadata,
            embedding: None,
            embedding_model,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get file extension based on format
    pub fn file_extension(&self) -> &str {
        match self.format {
            ContentFormat::Markdown => "md",
            ContentFormat::Yaml => "yaml",
            ContentFormat::Json => "json",
            ContentFormat::OpenApi => "yaml",
            ContentFormat::Text => "txt",
        }
    }
}

/// Search filters
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub artifact_type: Option<ArtifactType>,
    pub metadata: HashMap<String, String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub artifact: Artifact,
    pub score: f32,
}

/// Configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            name: "BAAI/bge-small-en-v1.5".to_string(),
            api_key: None,
            base_url: None,
        }
    }
}

/// Configuration for storage backend
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage URI: local path or s3://bucket/path
    /// Default: ".dna/artifacts.lance" (relative to project root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Project configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub model: ModelConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    mod id_generation {
        use super::*;

        const ALPHABET: &str = "23456789abcdefghjkmnpqrstuvwxyz";

        #[test]
        fn id_length_is_10() {
            let id = Artifact::generate_id();
            assert_eq!(id.len(), 10);
        }

        #[test]
        fn id_uses_valid_alphabet() {
            let id = Artifact::generate_id();
            for ch in id.chars() {
                assert!(
                    ALPHABET.contains(ch),
                    "ID character '{}' is not in the allowed alphabet",
                    ch
                );
            }
        }

        #[test]
        fn id_excludes_ambiguous_chars() {
            const AMBIGUOUS: &[char] = &['0', '1', 'o', 'O', 'i', 'I', 'l', 'L'];
            for _ in 0..100 {
                let id = Artifact::generate_id();
                for ch in id.chars() {
                    assert!(
                        !AMBIGUOUS.contains(&ch),
                        "ID should not contain ambiguous character '{}'",
                        ch
                    );
                }
            }
        }

        #[test]
        fn id_is_lowercase() {
            for _ in 0..100 {
                let id = Artifact::generate_id();
                for ch in id.chars() {
                    if ch.is_alphabetic() {
                        assert!(ch.is_lowercase());
                    }
                }
            }
        }

        #[test]
        fn ids_are_unique() {
            let ids: HashSet<String> = (0..1000).map(|_| Artifact::generate_id()).collect();
            assert_eq!(ids.len(), 1000, "Generated duplicate IDs");
        }

        #[test]
        fn id_is_url_safe() {
            let id = Artifact::generate_id();
            // All characters should be alphanumeric
            assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }

    mod artifact_type {
        use super::*;

        #[test]
        fn serializes_lowercase() {
            let types = [
                (ArtifactType::Intent, "\"intent\""),
                (ArtifactType::Invariant, "\"invariant\""),
                (ArtifactType::Contract, "\"contract\""),
                (ArtifactType::Algorithm, "\"algorithm\""),
                (ArtifactType::Evaluation, "\"evaluation\""),
                (ArtifactType::Pace, "\"pace\""),
                (ArtifactType::Monitor, "\"monitor\""),
            ];

            for (artifact_type, expected_json) in types {
                let json = serde_json::to_string(&artifact_type).unwrap();
                assert_eq!(json, expected_json);
            }
        }

        #[test]
        fn deserializes_lowercase() {
            let types = [
                ("\"intent\"", ArtifactType::Intent),
                ("\"invariant\"", ArtifactType::Invariant),
                ("\"contract\"", ArtifactType::Contract),
                ("\"algorithm\"", ArtifactType::Algorithm),
                ("\"evaluation\"", ArtifactType::Evaluation),
                ("\"pace\"", ArtifactType::Pace),
                ("\"monitor\"", ArtifactType::Monitor),
            ];

            for (json, expected) in types {
                let result: ArtifactType = serde_json::from_str(json).unwrap();
                assert_eq!(result, expected);
            }
        }

        #[test]
        fn from_str_case_insensitive() {
            assert_eq!(
                "Intent".parse::<ArtifactType>().unwrap(),
                ArtifactType::Intent
            );
            assert_eq!(
                "INTENT".parse::<ArtifactType>().unwrap(),
                ArtifactType::Intent
            );
            assert_eq!(
                "intent".parse::<ArtifactType>().unwrap(),
                ArtifactType::Intent
            );
        }

        #[test]
        fn from_str_invalid_returns_error() {
            let result = "invalid".parse::<ArtifactType>();
            assert!(result.is_err());
        }

        #[test]
        fn display_returns_lowercase_for_all_types() {
            assert_eq!(ArtifactType::Intent.to_string(), "intent");
            assert_eq!(ArtifactType::Invariant.to_string(), "invariant");
            assert_eq!(ArtifactType::Contract.to_string(), "contract");
            assert_eq!(ArtifactType::Algorithm.to_string(), "algorithm");
            assert_eq!(ArtifactType::Evaluation.to_string(), "evaluation");
            assert_eq!(ArtifactType::Pace.to_string(), "pace");
            assert_eq!(ArtifactType::Monitor.to_string(), "monitor");
        }

        #[test]
        fn all_seven_types_exist() {
            let types = [
                ArtifactType::Intent,
                ArtifactType::Invariant,
                ArtifactType::Contract,
                ArtifactType::Algorithm,
                ArtifactType::Evaluation,
                ArtifactType::Pace,
                ArtifactType::Monitor,
            ];
            assert_eq!(types.len(), 7);
        }
    }

    mod content_format {
        use super::*;

        #[test]
        fn serializes_lowercase() {
            let formats = [
                (ContentFormat::Markdown, "\"markdown\""),
                (ContentFormat::Yaml, "\"yaml\""),
                (ContentFormat::Json, "\"json\""),
                (ContentFormat::OpenApi, "\"openapi\""),
                (ContentFormat::Text, "\"text\""),
            ];

            for (format, expected) in formats {
                let json = serde_json::to_string(&format).unwrap();
                assert_eq!(json, expected);
            }
        }

        #[test]
        fn from_str_accepts_aliases() {
            assert_eq!(
                "md".parse::<ContentFormat>().unwrap(),
                ContentFormat::Markdown
            );
            assert_eq!(
                "markdown".parse::<ContentFormat>().unwrap(),
                ContentFormat::Markdown
            );
            assert_eq!("yml".parse::<ContentFormat>().unwrap(), ContentFormat::Yaml);
            assert_eq!(
                "yaml".parse::<ContentFormat>().unwrap(),
                ContentFormat::Yaml
            );
            assert_eq!("txt".parse::<ContentFormat>().unwrap(), ContentFormat::Text);
            assert_eq!(
                "text".parse::<ContentFormat>().unwrap(),
                ContentFormat::Text
            );
        }

        #[test]
        fn from_str_invalid_returns_error() {
            let result = "invalid".parse::<ContentFormat>();
            assert!(result.is_err());
        }

        #[test]
        fn display_returns_lowercase_for_all_formats() {
            assert_eq!(ContentFormat::Markdown.to_string(), "markdown");
            assert_eq!(ContentFormat::Yaml.to_string(), "yaml");
            assert_eq!(ContentFormat::Json.to_string(), "json");
            assert_eq!(ContentFormat::OpenApi.to_string(), "openapi");
            assert_eq!(ContentFormat::Text.to_string(), "text");
        }
    }

    mod artifact {
        use super::*;

        #[test]
        fn new_generates_unique_id() {
            let a1 = Artifact::new(
                ArtifactType::Intent,
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            let a2 = Artifact::new(
                ArtifactType::Intent,
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_ne!(a1.id, a2.id);
        }

        #[test]
        fn new_sets_timestamps() {
            let artifact = Artifact::new(
                ArtifactType::Intent,
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_eq!(artifact.created_at, artifact.updated_at);
        }

        #[test]
        fn file_extension_matches_format() {
            let cases = [
                (ContentFormat::Markdown, "md"),
                (ContentFormat::Yaml, "yaml"),
                (ContentFormat::Json, "json"),
                (ContentFormat::OpenApi, "yaml"),
                (ContentFormat::Text, "txt"),
            ];

            for (format, expected_ext) in cases {
                let artifact = Artifact::new(
                    ArtifactType::Intent,
                    "content".to_string(),
                    format,
                    None,
                    HashMap::new(),
                    "model".to_string(),
                );
                assert_eq!(artifact.file_extension(), expected_ext);
            }
        }

        #[test]
        fn json_roundtrip() {
            let artifact = Artifact::new(
                ArtifactType::Contract,
                "Test content with unicode: 'test'".to_string(),
                ContentFormat::Json,
                Some("test-name".to_string()),
                HashMap::from([("key".to_string(), "value".to_string())]),
                "test-model".to_string(),
            );

            let json = serde_json::to_string(&artifact).unwrap();
            let deserialized: Artifact = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.id, artifact.id);
            assert_eq!(deserialized.artifact_type, artifact.artifact_type);
            assert_eq!(deserialized.content, artifact.content);
            assert_eq!(deserialized.name, artifact.name);
        }

        #[test]
        fn handles_unicode_content() {
            let unicode_content = "Hello 'test' test";
            let artifact = Artifact::new(
                ArtifactType::Intent,
                unicode_content.to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_eq!(artifact.content, unicode_content);
        }

        #[test]
        fn handles_empty_content() {
            let artifact = Artifact::new(
                ArtifactType::Intent,
                String::new(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert!(artifact.content.is_empty());
        }

        #[test]
        fn handles_large_content() {
            let large_content = "x".repeat(100_000);
            let artifact = Artifact::new(
                ArtifactType::Intent,
                large_content.clone(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_eq!(artifact.content.len(), 100_000);
        }
    }

    mod model_config {
        use super::*;

        #[test]
        fn default_is_local_bge_small() {
            let config = ModelConfig::default();
            assert_eq!(config.provider, "local");
            assert_eq!(config.name, "BAAI/bge-small-en-v1.5");
            assert!(config.api_key.is_none());
            assert!(config.base_url.is_none());
        }
    }

    mod storage_config {
        use super::*;

        #[test]
        fn default_has_no_uri() {
            let config = StorageConfig::default();
            assert!(config.uri.is_none());
        }
    }

    mod search_filters {
        use super::*;

        #[test]
        fn default_is_empty() {
            let filters = SearchFilters::default();
            assert!(filters.artifact_type.is_none());
            assert!(filters.metadata.is_empty());
            assert!(filters.since.is_none());
            assert!(filters.limit.is_none());
        }
    }
}
