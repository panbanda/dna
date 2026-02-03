use axum::Router;
use dna::db::Database;
use dna::embedding::EmbeddingProvider;
use dna::mcp::{DnaToolHandler, RegisteredKind};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpService,
};
use std::sync::Arc;

use crate::state::AppState;

pub fn mcp_router(
    db: Arc<dyn Database>,
    embedding: Arc<dyn EmbeddingProvider>,
    kinds: Vec<RegisteredKind>,
) -> Router<AppState> {
    let service = StreamableHttpService::new(
        move || {
            Ok(DnaToolHandler::with_kinds(
                db.clone(),
                embedding.clone(),
                None,
                None,
                kinds.clone(),
            ))
        },
        LocalSessionManager::default().into(),
        Default::default(),
    );

    Router::new().nest_service("/mcp", service)
}
