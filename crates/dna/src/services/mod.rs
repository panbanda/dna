pub mod artifact;
pub mod config;
pub mod kind;
pub mod search;
pub mod types;

pub use artifact::ArtifactService;
pub use config::ConfigService;
pub use kind::KindService;
pub use search::SearchService;
pub use types::{
    slugify_kind, validate_kind_slug, Artifact, ContentFormat, KindDefinition, KindValidationError,
    KindsConfig, ModelConfig, ProjectConfig, SearchFilters, SearchResult, StorageConfig,
    KIND_SLUG_MAX_LENGTH, KIND_SLUG_MIN_LENGTH, RESERVED_KIND_SLUGS,
};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}
