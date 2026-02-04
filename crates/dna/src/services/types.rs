use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
    /// Additional context for improved semantic retrieval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Embedding of the context (same dimensions as content embedding)
    #[serde(skip)]
    pub context_embedding: Option<Vec<f32>>,
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
            context: None,
            context_embedding: None,
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

/// Information about an embedding model's capabilities
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub max_tokens: usize,
    pub dimensions: usize,
}

/// Estimate token count from text (conservative estimate).
///
/// Uses a rough heuristic of ~0.75 words per token for English text.
/// This is conservative to avoid exceeding model limits.
pub fn estimate_tokens(text: &str) -> usize {
    let words = text.split_whitespace().count();
    (words as f64 / 0.75).ceil() as usize
}

/// Get model info from registry, with fallback for unknown models
pub fn get_model_info(model: &str) -> ModelInfo {
    match model {
        "BAAI/bge-small-en-v1.5" => ModelInfo {
            max_tokens: 512,
            dimensions: 384,
        },
        "BAAI/bge-base-en-v1.5" => ModelInfo {
            max_tokens: 512,
            dimensions: 768,
        },
        "text-embedding-3-small" => ModelInfo {
            max_tokens: 8191,
            dimensions: 1536,
        },
        "text-embedding-3-large" => ModelInfo {
            max_tokens: 8191,
            dimensions: 3072,
        },
        "nomic-embed-text" => ModelInfo {
            max_tokens: 8192,
            dimensions: 768,
        },
        "voyage-3" => ModelInfo {
            max_tokens: 32000,
            dimensions: 1024,
        },
        // Conservative default for unknown models
        _ => ModelInfo {
            max_tokens: 512,
            dimensions: 384,
        },
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

/// Specifies which embeddings to regenerate during reindexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReindexTarget {
    /// Regenerate content embeddings only.
    Content,
    /// Regenerate context embeddings only.
    Context,
    /// Regenerate both content and context embeddings.
    Both,
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
    /// Enable auto-pruning after mutations (default: false preserves history)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_prune: Option<bool>,
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

/// A kind definition within a template
#[derive(Debug, Clone)]
pub struct TemplateKind {
    pub slug: &'static str,
    pub description: &'static str,
}

/// A project template defining a set of artifact kinds
#[derive(Debug, Clone)]
pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub kinds: &'static [TemplateKind],
}

/// Intent template: truth-driven governance based on intent-starter pattern
pub static TEMPLATE_INTENT: Template = Template {
    name: "intent",
    description: "Truth-driven governance for system identity",
    kinds: &[
        TemplateKind {
            slug: "intent",
            description: "Declarative 'must' statement: one user-observable outcome or rule. No implementation. Ex: 'Orders must not ship until payment confirmed'",
        },
        TemplateKind {
            slug: "contract",
            description: "External promise: one endpoint, event, or interface. Ex: 'POST /orders returns 201 with order_id'",
        },
        TemplateKind {
            slug: "algorithm",
            description: "Computation rule: one formula or threshold. Ex: 'discount = 0.1 when qty > 10'",
        },
        TemplateKind {
            slug: "evaluation",
            description: "Executable test: one invariant, scenario, or regression. Use --label type=invariant|scenario|regression. Ex: 'Account balance >= 0' or 'Given expired token, then 401'",
        },
        TemplateKind {
            slug: "pace",
            description: "Change governance: one concern as fast/medium/slow. Ex: 'auth model: slow'",
        },
        TemplateKind {
            slug: "monitor",
            description: "Operational observable: one metric or SLO. Ex: 'p99_latency < 200ms'",
        },
        TemplateKind {
            slug: "glossary",
            description: "Domain term: one concept with precise meaning. Ex: 'ICP: B2B SaaS, 50-500 employees, Series A+'",
        },
        TemplateKind {
            slug: "integration",
            description: "External binding: one provider, API, or SLA term. Use --label provider=x. Ex: 'Payment provider: Stripe'",
        },
        TemplateKind {
            slug: "reporting",
            description: "Reportable requirement: one business or compliance query. Ex: 'Revenue by segment must be queryable'",
        },
        TemplateKind {
            slug: "compliance",
            description: "Regulatory or legal obligation: one requirement from GDPR, HIPAA, PCI-DSS, SOC2, etc. Use --label regulation=x. Ex: 'PII must be deletable within 30 days of request'",
        },
        TemplateKind {
            slug: "constraint",
            description: "Technical limit or boundary: one capacity, performance, or architectural constraint. Ex: 'Max upload size: 100MB' or 'Must run stateless for horizontal scaling'",
        },
    ],
};

/// Agentic template: safety and governance for AI agents and LLM systems
pub static TEMPLATE_AGENTIC: Template = Template {
    name: "agentic",
    description: "Safety and governance for AI agents and LLM systems",
    kinds: &[
        TemplateKind {
            slug: "behavior",
            description: "Model capability, style, or grounding rule. What it does and how. Use --label aspect=capability|grounding|agency|tone. Ex: 'Summarize up to 100k tokens' or 'Must cite source documents'",
        },
        TemplateKind {
            slug: "boundary",
            description: "Safety limit or content policy. What it must not do. Use --label type=filter|policy|redline. Ex: 'Reject injection patterns' or 'Never generate CSAM'",
        },
        TemplateKind {
            slug: "threat",
            description: "Attack vector with mitigation. Use --label owasp=LLM01-10. Ex: 'LLM01: Prompt injection via role hijacking - validate system prompt immutability'",
        },
        TemplateKind {
            slug: "eval",
            description: "Verification benchmark or criteria. Use --label type=safety|bias|accuracy|redteam. Ex: 'Safety score >= 95% on HarmBench' or 'Gender bias < 0.1 on WinoBias'",
        },
        TemplateKind {
            slug: "governance",
            description: "Oversight, transparency, audit, or provenance. Use --label aspect=oversight|disclosure|audit|provenance. Ex: 'Human review for appeals' or 'Must identify as AI when asked'",
        },
    ],
};

/// All available templates
pub static TEMPLATES: &[&Template] = &[&TEMPLATE_INTENT, &TEMPLATE_AGENTIC];

/// Get a template by name
pub fn get_template(name: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.name == name).copied()
}

/// List all available template names
pub fn list_templates() -> Vec<&'static str> {
    TEMPLATES.iter().map(|t| t.name).collect()
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

        #[test]
        fn default_has_no_auto_prune() {
            let config = StorageConfig::default();
            assert!(config.auto_prune.is_none());
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

    mod model_registry {
        use super::*;

        #[test]
        fn returns_known_limits_for_bge_small() {
            let info = get_model_info("BAAI/bge-small-en-v1.5");
            assert_eq!(info.max_tokens, 512);
            assert_eq!(info.dimensions, 384);
        }

        #[test]
        fn returns_known_limits_for_bge_base() {
            let info = get_model_info("BAAI/bge-base-en-v1.5");
            assert_eq!(info.max_tokens, 512);
            assert_eq!(info.dimensions, 768);
        }

        #[test]
        fn returns_known_limits_for_openai_small() {
            let info = get_model_info("text-embedding-3-small");
            assert_eq!(info.max_tokens, 8191);
            assert_eq!(info.dimensions, 1536);
        }

        #[test]
        fn returns_known_limits_for_openai_large() {
            let info = get_model_info("text-embedding-3-large");
            assert_eq!(info.max_tokens, 8191);
            assert_eq!(info.dimensions, 3072);
        }

        #[test]
        fn returns_known_limits_for_nomic() {
            let info = get_model_info("nomic-embed-text");
            assert_eq!(info.max_tokens, 8192);
            assert_eq!(info.dimensions, 768);
        }

        #[test]
        fn returns_known_limits_for_voyage() {
            let info = get_model_info("voyage-3");
            assert_eq!(info.max_tokens, 32000);
            assert_eq!(info.dimensions, 1024);
        }

        #[test]
        fn unknown_model_gets_conservative_default() {
            let info = get_model_info("unknown-model-xyz");
            assert_eq!(
                info.max_tokens, 512,
                "Unknown models should default to 512 tokens"
            );
            assert_eq!(
                info.dimensions, 384,
                "Unknown models should default to 384 dimensions"
            );
        }
    }

    mod token_estimation {
        use super::*;

        #[test]
        fn empty_text_returns_zero() {
            assert_eq!(estimate_tokens(""), 0);
        }

        #[test]
        fn single_word_returns_at_least_one() {
            let tokens = estimate_tokens("hello");
            assert!(tokens >= 1, "Single word should estimate at least 1 token");
        }

        #[test]
        fn word_count_correlates_with_tokens() {
            let short = estimate_tokens("one two three");
            let long = estimate_tokens("one two three four five six seven eight nine ten");
            assert!(long > short, "More words should estimate more tokens");
        }

        #[test]
        fn estimates_conservatively() {
            // ~0.75 words per token means 100 words should be ~133 tokens
            let text = "word ".repeat(100);
            let tokens = estimate_tokens(&text);
            // Should be at least 100 tokens (conservative)
            assert!(
                tokens >= 100,
                "100 words should estimate at least 100 tokens, got {}",
                tokens
            );
        }
    }

    mod templates {
        use super::*;

        #[test]
        fn get_template_returns_intent() {
            let template = get_template("intent");
            assert!(template.is_some());
            let template = template.unwrap();
            assert_eq!(template.name, "intent");
            assert!(!template.kinds.is_empty());
        }

        #[test]
        fn get_template_returns_none_for_unknown() {
            assert!(get_template("unknown").is_none());
            assert!(get_template("").is_none());
            assert!(get_template("Intent").is_none()); // case sensitive
        }

        #[test]
        fn list_templates_includes_all() {
            let templates = list_templates();
            assert!(templates.contains(&"intent"));
            assert!(templates.contains(&"agentic"));
            assert_eq!(templates.len(), 2);
        }

        #[test]
        fn get_agentic_template() {
            let template = get_template("agentic");
            assert!(template.is_some());
            let template = template.unwrap();
            assert_eq!(template.name, "agentic");
        }

        #[test]
        fn agentic_template_has_expected_kinds() {
            let template = get_template("agentic").unwrap();
            let slugs: Vec<&str> = template.kinds.iter().map(|k| k.slug).collect();

            assert!(slugs.contains(&"behavior"));
            assert!(slugs.contains(&"boundary"));
            assert!(slugs.contains(&"threat"));
            assert!(slugs.contains(&"eval"));
            assert!(slugs.contains(&"governance"));
            assert_eq!(slugs.len(), 5);
        }

        #[test]
        fn intent_template_has_expected_kinds() {
            let template = get_template("intent").unwrap();
            let slugs: Vec<&str> = template.kinds.iter().map(|k| k.slug).collect();

            assert!(slugs.contains(&"intent"));
            assert!(slugs.contains(&"contract"));
            assert!(slugs.contains(&"algorithm"));
            assert!(slugs.contains(&"evaluation"));
            assert!(slugs.contains(&"pace"));
            assert!(slugs.contains(&"monitor"));
            assert!(slugs.contains(&"glossary"));
            assert!(slugs.contains(&"integration"));
            assert!(slugs.contains(&"reporting"));
            assert!(slugs.contains(&"compliance"));
            assert!(slugs.contains(&"constraint"));
            assert_eq!(slugs.len(), 11);
        }

        #[test]
        fn all_template_kinds_have_descriptions_with_examples() {
            for name in list_templates() {
                let template = get_template(name).unwrap();
                for kind in template.kinds {
                    assert!(
                        !kind.description.is_empty(),
                        "{}:{} has empty description",
                        name,
                        kind.slug
                    );
                    assert!(
                        kind.description.contains("Ex:"),
                        "{}:{} description should contain an example",
                        name,
                        kind.slug
                    );
                }
            }
        }

        #[test]
        fn all_template_kind_slugs_are_valid() {
            for name in list_templates() {
                let template = get_template(name).unwrap();
                for kind in template.kinds {
                    assert!(
                        validate_kind_slug(kind.slug).is_ok(),
                        "{}:{} has invalid slug",
                        name,
                        kind.slug
                    );
                }
            }
        }
    }
}
