use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum length for a kind slug (64 characters).
pub const KIND_SLUG_MAX_LENGTH: usize = 64;

/// Minimum length for a kind slug (2 characters).
pub const KIND_SLUG_MIN_LENGTH: usize = 2;

/// Reserved kind slugs that cannot be used.
pub const RESERVED_KIND_SLUGS: &[&str] = &[
    "all",
    "any",
    "artifact",
    "artifacts",
    "config",
    "default",
    "kind",
    "kinds",
    "none",
    "search",
    "system",
];

/// Transform a kind string to kebab-case slug.
pub fn slugify_kind(input: &str) -> String {
    slug::slugify(input)
}

/// Error returned when kind slug validation fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindValidationError {
    /// Slug is empty after slugification.
    Empty,
    /// Slug is too short.
    TooShort { min: usize, actual: usize },
    /// Slug is too long.
    TooLong { max: usize, actual: usize },
    /// Slug is a reserved word.
    Reserved { slug: String },
    /// Slug contains invalid characters (should not happen after slugify).
    InvalidChars { slug: String },
}

impl std::fmt::Display for KindValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KindValidationError::Empty => write!(f, "kind slug cannot be empty"),
            KindValidationError::TooShort { min, actual } => {
                write!(
                    f,
                    "kind slug too short: minimum {} characters, got {}",
                    min, actual
                )
            },
            KindValidationError::TooLong { max, actual } => {
                write!(
                    f,
                    "kind slug too long: maximum {} characters, got {}",
                    max, actual
                )
            },
            KindValidationError::Reserved { slug } => {
                write!(f, "kind slug '{}' is reserved", slug)
            },
            KindValidationError::InvalidChars { slug } => {
                write!(f, "kind slug '{}' contains invalid characters", slug)
            },
        }
    }
}

impl std::error::Error for KindValidationError {}

/// Validate a kind slug.
///
/// Returns Ok(()) if the slug is valid, or an error describing why it's invalid.
/// The slug should already be slugified before calling this function.
pub fn validate_kind_slug(slug: &str) -> Result<(), KindValidationError> {
    if slug.is_empty() {
        return Err(KindValidationError::Empty);
    }

    if slug.len() < KIND_SLUG_MIN_LENGTH {
        return Err(KindValidationError::TooShort {
            min: KIND_SLUG_MIN_LENGTH,
            actual: slug.len(),
        });
    }

    if slug.len() > KIND_SLUG_MAX_LENGTH {
        return Err(KindValidationError::TooLong {
            max: KIND_SLUG_MAX_LENGTH,
            actual: slug.len(),
        });
    }

    if RESERVED_KIND_SLUGS.contains(&slug) {
        return Err(KindValidationError::Reserved {
            slug: slug.to_string(),
        });
    }

    // Verify slug only contains valid characters (lowercase alphanumeric and hyphens)
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(KindValidationError::InvalidChars {
            slug: slug.to_string(),
        });
    }

    // Ensure slug doesn't start or end with hyphen
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(KindValidationError::InvalidChars {
            slug: slug.to_string(),
        });
    }

    Ok(())
}

/// Content format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    pub kind: String,
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
        kind: String,
        content: String,
        format: ContentFormat,
        name: Option<String>,
        metadata: HashMap<String, String>,
        embedding_model: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Self::generate_id(),
            kind,
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
    pub kind: Option<String>,
    pub metadata: HashMap<String, String>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
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

/// Definition of a registered artifact kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindDefinition {
    pub slug: String,
    pub description: String,
}

/// Configuration for registered artifact kinds
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KindsConfig {
    #[serde(default)]
    pub definitions: Vec<KindDefinition>,
}

impl KindsConfig {
    /// Check if a kind is registered
    pub fn has(&self, slug: &str) -> bool {
        self.definitions.iter().any(|d| d.slug == slug)
    }

    /// Add a kind definition, returning false if already exists
    pub fn add(&mut self, slug: String, description: String) -> bool {
        if self.has(&slug) {
            return false;
        }
        self.definitions.push(KindDefinition { slug, description });
        true
    }

    /// Remove a kind definition, returning true if it existed
    pub fn remove(&mut self, slug: &str) -> bool {
        let len = self.definitions.len();
        self.definitions.retain(|d| d.slug != slug);
        self.definitions.len() < len
    }

    /// Get a kind definition by slug
    pub fn get(&self, slug: &str) -> Option<&KindDefinition> {
        self.definitions.iter().find(|d| d.slug == slug)
    }

    /// List all registered kind slugs
    pub fn slugs(&self) -> Vec<&str> {
        self.definitions.iter().map(|d| d.slug.as_str()).collect()
    }
}

/// Project configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub model: ModelConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub kinds: KindsConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    mod slugify {
        use super::*;

        #[test]
        fn transforms_to_kebab_case() {
            assert_eq!(slugify_kind("My Custom Type"), "my-custom-type");
            assert_eq!(slugify_kind("intent"), "intent");
            assert_eq!(slugify_kind("UPPER CASE"), "upper-case");
            assert_eq!(slugify_kind("already-slugged"), "already-slugged");
            assert_eq!(slugify_kind("with  extra   spaces"), "with-extra-spaces");
        }

        #[test]
        fn handles_special_characters() {
            assert_eq!(slugify_kind("hello/world"), "hello-world");
            assert_eq!(slugify_kind("test_case"), "test-case");
        }

        #[test]
        fn handles_empty_string() {
            assert_eq!(slugify_kind(""), "");
        }
    }

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
            assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
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
                "intent".to_string(),
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            let a2 = Artifact::new(
                "intent".to_string(),
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
                "intent".to_string(),
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_eq!(artifact.created_at, artifact.updated_at);
        }

        #[test]
        fn new_stores_kind() {
            let artifact = Artifact::new(
                "custom-kind".to_string(),
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_eq!(artifact.kind, "custom-kind");
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
                    "intent".to_string(),
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
                "contract".to_string(),
                "Test content with unicode: 'test'".to_string(),
                ContentFormat::Json,
                Some("test-name".to_string()),
                HashMap::from([("key".to_string(), "value".to_string())]),
                "test-model".to_string(),
            );

            let json = serde_json::to_string(&artifact).unwrap();
            let deserialized: Artifact = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.id, artifact.id);
            assert_eq!(deserialized.kind, artifact.kind);
            assert_eq!(deserialized.content, artifact.content);
            assert_eq!(deserialized.name, artifact.name);
        }

        #[test]
        fn json_serializes_kind_field() {
            let artifact = Artifact::new(
                "my-kind".to_string(),
                "content".to_string(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );

            let json = serde_json::to_string(&artifact).unwrap();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["kind"], "my-kind");
        }

        #[test]
        fn handles_unicode_content() {
            let unicode_content = "Hello 'test' test";
            let artifact = Artifact::new(
                "intent".to_string(),
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
                "intent".to_string(),
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
                "intent".to_string(),
                large_content.clone(),
                ContentFormat::Markdown,
                None,
                HashMap::new(),
                "model".to_string(),
            );
            assert_eq!(artifact.content.len(), 100_000);
        }

        #[test]
        fn accepts_any_kind_string() {
            let kinds = ["intent", "custom-kind", "my-special-type", "x"];
            for kind in kinds {
                let artifact = Artifact::new(
                    kind.to_string(),
                    "content".to_string(),
                    ContentFormat::Markdown,
                    None,
                    HashMap::new(),
                    "model".to_string(),
                );
                assert_eq!(artifact.kind, kind);
            }
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
            assert!(filters.kind.is_none());
            assert!(filters.metadata.is_empty());
            assert!(filters.after.is_none());
            assert!(filters.before.is_none());
            assert!(filters.limit.is_none());
        }
    }

    mod kind_validation {
        use super::*;

        #[test]
        fn valid_slugs_pass() {
            let valid = ["intent", "my-kind", "custom-type-123", "ab"];
            for slug in valid {
                assert!(
                    validate_kind_slug(slug).is_ok(),
                    "Expected '{}' to be valid",
                    slug
                );
            }
        }

        #[test]
        fn empty_slug_rejected() {
            assert_eq!(validate_kind_slug(""), Err(KindValidationError::Empty));
        }

        #[test]
        fn too_short_rejected() {
            assert_eq!(
                validate_kind_slug("a"),
                Err(KindValidationError::TooShort { min: 2, actual: 1 })
            );
        }

        #[test]
        fn too_long_rejected() {
            let long_slug = "a".repeat(65);
            assert_eq!(
                validate_kind_slug(&long_slug),
                Err(KindValidationError::TooLong {
                    max: 64,
                    actual: 65
                })
            );
        }

        #[test]
        fn max_length_slug_accepted() {
            let max_slug = "a".repeat(64);
            assert!(validate_kind_slug(&max_slug).is_ok());
        }

        #[test]
        fn reserved_slugs_rejected() {
            for reserved in RESERVED_KIND_SLUGS {
                assert_eq!(
                    validate_kind_slug(reserved),
                    Err(KindValidationError::Reserved {
                        slug: reserved.to_string()
                    }),
                    "Expected '{}' to be rejected as reserved",
                    reserved
                );
            }
        }

        #[test]
        fn invalid_chars_rejected() {
            let invalid = ["UPPER", "with_underscore", "with space", "with.dot"];
            for slug in invalid {
                assert!(
                    matches!(
                        validate_kind_slug(slug),
                        Err(KindValidationError::InvalidChars { .. })
                    ),
                    "Expected '{}' to be rejected for invalid chars",
                    slug
                );
            }
        }

        #[test]
        fn leading_hyphen_rejected() {
            assert!(matches!(
                validate_kind_slug("-leading"),
                Err(KindValidationError::InvalidChars { .. })
            ));
        }

        #[test]
        fn trailing_hyphen_rejected() {
            assert!(matches!(
                validate_kind_slug("trailing-"),
                Err(KindValidationError::InvalidChars { .. })
            ));
        }

        #[test]
        fn error_display_messages() {
            assert_eq!(
                KindValidationError::Empty.to_string(),
                "kind slug cannot be empty"
            );
            assert!(KindValidationError::TooShort { min: 2, actual: 1 }
                .to_string()
                .contains("too short"));
            assert!(KindValidationError::TooLong {
                max: 64,
                actual: 65
            }
            .to_string()
            .contains("too long"));
            assert!(KindValidationError::Reserved {
                slug: "all".to_string()
            }
            .to_string()
            .contains("reserved"));
        }
    }
}
