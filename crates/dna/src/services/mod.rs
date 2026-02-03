pub mod artifact;
pub mod config;
pub mod kind;
pub mod search;
pub mod types;

pub use artifact::ArtifactService;
pub use config::ConfigService;
pub use kind::KindService;
pub use search::SearchService;
pub use types::*;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}
