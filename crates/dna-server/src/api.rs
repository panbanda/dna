use axum::{
    extract::{Path, Query, State},
    http::{header, Method},
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use dna::services::{ArtifactType, ContentFormat, SearchFilters, ServiceError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;

use crate::auth::{auth_middleware, require_write, ApiKeyAuth};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(rename = "type")]
    artifact_type: Option<String>,
    limit: Option<usize>,
    after: Option<String>,
    before: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchBody {
    query: String,
    #[serde(rename = "type")]
    artifact_type: Option<String>,
    limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateBody {
    #[serde(rename = "type")]
    artifact_type: String,
    content: String,
    format: Option<String>,
    name: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct UpdateBody {
    content: Option<String>,
    name: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    code: String,
    message: String,
}

fn error_response(
    status: axum::http::StatusCode,
    code: &str,
    message: &str,
) -> axum::response::Response {
    let body = ErrorResponse {
        error: ErrorDetail {
            code: code.to_string(),
            message: message.to_string(),
        },
    };
    (status, Json(body)).into_response()
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, String> {
    s.parse::<DateTime<Utc>>()
        .map_err(|e| format!("Invalid datetime '{}': {}", s, e))
}

fn parse_artifact_type(s: &str) -> Result<ArtifactType, String> {
    s.parse::<ArtifactType>()
        .map_err(|e| format!("Invalid artifact type '{}': {}", s, e))
}

fn parse_content_format(s: &str) -> Result<ContentFormat, String> {
    s.parse::<ContentFormat>()
        .map_err(|e| format!("Invalid content format '{}': {}", s, e))
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

async fn list_artifacts(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> axum::response::Response {
    let artifact_type = match query.artifact_type {
        Some(ref t) => match parse_artifact_type(t) {
            Ok(at) => Some(at),
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => None,
    };

    let after = match query.after {
        Some(ref s) => match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => None,
    };

    let before = match query.before {
        Some(ref s) => match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => None,
    };

    let filters = SearchFilters {
        artifact_type,
        after,
        before,
        limit: query.limit,
        ..Default::default()
    };

    match state.artifact_service.list(filters).await {
        Ok(artifacts) => Json(serde_json::json!({"artifacts": artifacts})).into_response(),
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

async fn create_artifact(
    State(state): State<AppState>,
    Json(body): Json<CreateBody>,
) -> axum::response::Response {
    let artifact_type = match parse_artifact_type(&body.artifact_type) {
        Ok(at) => at,
        Err(msg) => {
            return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
        },
    };

    let format = match body.format {
        Some(ref f) => match parse_content_format(f) {
            Ok(cf) => cf,
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => ContentFormat::Markdown,
    };

    let metadata = body.metadata.unwrap_or_default();

    match state
        .artifact_service
        .add(artifact_type, body.content, format, body.name, metadata)
        .await
    {
        Ok(artifact) => (axum::http::StatusCode::CREATED, Json(artifact)).into_response(),
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

async fn get_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    match state.artifact_service.get(&id).await {
        Ok(Some(artifact)) => Json(artifact).into_response(),
        Ok(None) => error_response(
            axum::http::StatusCode::NOT_FOUND,
            "not_found",
            &format!("Artifact '{}' not found", id),
        ),
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

async fn update_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> axum::response::Response {
    match state
        .artifact_service
        .update(&id, body.content, body.name, body.metadata)
        .await
    {
        Ok(artifact) => Json(artifact).into_response(),
        Err(ServiceError::NotFound(msg)) => {
            error_response(axum::http::StatusCode::NOT_FOUND, "not_found", &msg)
        },
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

async fn delete_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    match state.artifact_service.remove(&id).await {
        Ok(true) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Ok(false) => error_response(
            axum::http::StatusCode::NOT_FOUND,
            "not_found",
            &format!("Artifact '{}' not found", id),
        ),
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

async fn search_artifacts(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> axum::response::Response {
    let artifact_type = match body.artifact_type {
        Some(ref t) => match parse_artifact_type(t) {
            Ok(at) => Some(at),
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => None,
    };

    let filters = SearchFilters {
        artifact_type,
        limit: body.limit,
        ..Default::default()
    };

    match state.search_service.search(&body.query, filters).await {
        Ok(results) => Json(serde_json::json!({"results": results})).into_response(),
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

async fn list_changes(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> axum::response::Response {
    let after = match query.after {
        Some(ref s) => match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => None,
    };

    let before = match query.before {
        Some(ref s) => match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(msg) => {
                return error_response(axum::http::StatusCode::BAD_REQUEST, "bad_request", &msg)
            },
        },
        None => None,
    };

    let filters = SearchFilters {
        after,
        before,
        limit: query.limit,
        ..Default::default()
    };

    match state.artifact_service.list(filters).await {
        Ok(artifacts) => Json(serde_json::json!({"changes": artifacts})).into_response(),
        Err(e) => error_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        ),
    }
}

pub fn build_router(state: AppState) -> Router {
    let api_key_auth = ApiKeyAuth::from_env();

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // Routes that require write access
    let write_routes = Router::new()
        .route("/api/v1/artifacts", post(create_artifact))
        .route(
            "/api/v1/artifacts/{id}",
            put(update_artifact).delete(delete_artifact),
        )
        .route_layer(middleware::from_fn(require_write));

    // Read-only API routes
    let read_routes = Router::new()
        .route("/api/v1/artifacts", get(list_artifacts))
        .route("/api/v1/artifacts/{id}", get(get_artifact))
        .route("/api/v1/search", post(search_artifacts))
        .route("/api/v1/changes", get(list_changes));

    // Combine API routes with auth middleware
    let api_routes = Router::new()
        .merge(write_routes)
        .merge(read_routes)
        .route_layer(middleware::from_fn(auth_middleware))
        .layer(axum::Extension(api_key_auth));

    // MCP routes
    let mcp_routes = crate::mcp::mcp_router(state.db.clone(), state.embedding.clone());

    Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        .merge(mcp_routes)
        .layer(cors)
        .with_state(state)
}
