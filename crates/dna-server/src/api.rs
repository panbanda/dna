use axum::{
    extract::{Path, Query, State},
    http::{header, Method},
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use dna::services::{Artifact, ContentFormat, SearchFilters, SearchResult, ServiceError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::{auth_middleware, require_write, ApiKeyAuth};
use crate::state::AppState;

/// Query parameters for listing artifacts
#[derive(Deserialize, ToSchema)]
pub struct ListQuery {
    /// Filter by artifact kind
    kind: Option<String>,
    /// Maximum number of results to return
    limit: Option<usize>,
    /// Only return artifacts created after this ISO 8601 timestamp
    after: Option<String>,
    /// Only return artifacts created before this ISO 8601 timestamp
    before: Option<String>,
}

/// Request body for searching artifacts
#[derive(Deserialize, ToSchema)]
pub struct SearchBody {
    /// Search query text
    query: String,
    /// Filter by artifact kind
    kind: Option<String>,
    /// Maximum number of results to return
    limit: Option<usize>,
}

/// Request body for creating an artifact
#[derive(Deserialize, ToSchema)]
pub struct CreateBody {
    /// Artifact kind (e.g., "intent", "contract")
    kind: String,
    /// Artifact content
    content: String,
    /// Content format: markdown, yaml, json, openapi, text
    format: Option<String>,
    /// Optional human-readable name
    name: Option<String>,
    /// Optional key-value metadata
    metadata: Option<HashMap<String, String>>,
}

/// Request body for updating an artifact
#[derive(Deserialize, ToSchema)]
pub struct UpdateBody {
    /// New content (optional)
    content: Option<String>,
    /// New name (optional)
    name: Option<String>,
    /// New kind (optional)
    kind: Option<String>,
    /// New metadata (replaces existing)
    metadata: Option<HashMap<String, String>>,
}

/// Error response wrapper
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error details
    error: ErrorDetail,
}

/// Error detail
#[derive(Serialize, ToSchema)]
pub struct ErrorDetail {
    /// Error code (e.g., "not_found", "bad_request")
    code: String,
    /// Human-readable error message
    message: String,
}

/// Response containing a list of artifacts
#[derive(Serialize, ToSchema)]
pub struct ArtifactListResponse {
    /// List of artifacts
    artifacts: Vec<Artifact>,
}

/// Response containing search results
#[derive(Serialize, ToSchema)]
pub struct SearchResultsResponse {
    /// Search results with scores
    results: Vec<SearchResult>,
}

/// Response containing changes (same as artifact list)
#[derive(Serialize, ToSchema)]
pub struct ChangesResponse {
    /// List of changed artifacts
    changes: Vec<Artifact>,
}

/// Health check response
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    status: String,
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

fn parse_content_format(s: &str) -> Result<ContentFormat, String> {
    s.parse::<ContentFormat>()
        .map_err(|e| format!("Invalid content format '{}': {}", s, e))
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(
    get,
    path = "/api/v1/artifacts",
    tag = "Artifacts",
    params(ListQuery),
    responses(
        (status = 200, description = "List of artifacts", body = ArtifactListResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
async fn list_artifacts(
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
        kind: query.kind,
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

#[utoipa::path(
    post,
    path = "/api/v1/artifacts",
    tag = "Artifacts",
    request_body = CreateBody,
    responses(
        (status = 201, description = "Artifact created", body = Artifact),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Write access required"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = ["write"]))
)]
async fn create_artifact(
    State(state): State<AppState>,
    Json(body): Json<CreateBody>,
) -> axum::response::Response {
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
        .add(body.kind, body.content, format, body.name, metadata, None)
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

#[utoipa::path(
    get,
    path = "/api/v1/artifacts/{id}",
    tag = "Artifacts",
    params(
        ("id" = String, Path, description = "Artifact ID")
    ),
    responses(
        (status = 200, description = "Artifact found", body = Artifact),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Artifact not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
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

#[utoipa::path(
    put,
    path = "/api/v1/artifacts/{id}",
    tag = "Artifacts",
    params(
        ("id" = String, Path, description = "Artifact ID")
    ),
    request_body = UpdateBody,
    responses(
        (status = 200, description = "Artifact updated", body = Artifact),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Write access required"),
        (status = 404, description = "Artifact not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = ["write"]))
)]
async fn update_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> axum::response::Response {
    match state
        .artifact_service
        .update(&id, body.content, body.name, body.kind, body.metadata, None)
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

#[utoipa::path(
    delete,
    path = "/api/v1/artifacts/{id}",
    tag = "Artifacts",
    params(
        ("id" = String, Path, description = "Artifact ID")
    ),
    responses(
        (status = 204, description = "Artifact deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Write access required"),
        (status = 404, description = "Artifact not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = ["write"]))
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/search",
    tag = "Search",
    request_body = SearchBody,
    responses(
        (status = 200, description = "Search results", body = SearchResultsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
async fn search_artifacts(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> axum::response::Response {
    let filters = SearchFilters {
        kind: body.kind,
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

// Kind-scoped request bodies (no kind field needed -- it comes from the URL)

/// Request body for creating an artifact within a kind scope
#[derive(Deserialize, ToSchema)]
pub struct KindCreateBody {
    /// Artifact content
    content: String,
    /// Content format: markdown, yaml, json, openapi, text
    format: Option<String>,
    /// Optional human-readable name
    name: Option<String>,
    /// Optional key-value metadata
    metadata: Option<HashMap<String, String>>,
}

/// Request body for searching within a kind scope
#[derive(Deserialize, ToSchema)]
pub struct KindSearchBody {
    /// Search query text
    query: String,
    /// Maximum number of results to return
    limit: Option<usize>,
}

/// Query parameters for listing artifacts within a kind scope
#[derive(Deserialize, ToSchema)]
pub struct KindListQuery {
    /// Maximum number of results to return
    limit: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/api/v1/kinds/{kind}/artifacts",
    tag = "Kinds",
    params(
        ("kind" = String, Path, description = "Artifact kind slug"),
        KindListQuery
    ),
    responses(
        (status = 200, description = "List of artifacts for this kind", body = ArtifactListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
async fn kind_list_artifacts(
    State(state): State<AppState>,
    Path(kind): Path<String>,
    Query(query): Query<KindListQuery>,
) -> axum::response::Response {
    let filters = SearchFilters {
        kind: Some(kind),
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

#[utoipa::path(
    post,
    path = "/api/v1/kinds/{kind}/artifacts",
    tag = "Kinds",
    params(
        ("kind" = String, Path, description = "Artifact kind slug")
    ),
    request_body = KindCreateBody,
    responses(
        (status = 201, description = "Artifact created", body = Artifact),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Write access required"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = ["write"]))
)]
async fn kind_create_artifact(
    State(state): State<AppState>,
    Path(kind): Path<String>,
    Json(body): Json<KindCreateBody>,
) -> axum::response::Response {
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
        .add(kind, body.content, format, body.name, metadata, None)
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

#[utoipa::path(
    post,
    path = "/api/v1/kinds/{kind}/search",
    tag = "Kinds",
    params(
        ("kind" = String, Path, description = "Artifact kind slug")
    ),
    request_body = KindSearchBody,
    responses(
        (status = 200, description = "Search results for this kind", body = SearchResultsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
async fn kind_search_artifacts(
    State(state): State<AppState>,
    Path(kind): Path<String>,
    Json(body): Json<KindSearchBody>,
) -> axum::response::Response {
    let filters = SearchFilters {
        kind: Some(kind),
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

#[utoipa::path(
    get,
    path = "/api/v1/changes",
    tag = "Changes",
    params(ListQuery),
    responses(
        (status = 200, description = "List of recent changes", body = ChangesResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
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

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "DNA API",
        description = "Truth artifact management API with vector search",
        version = "0.3.2",
        license(name = "MIT")
    ),
    paths(
        health,
        list_artifacts,
        create_artifact,
        get_artifact,
        update_artifact,
        delete_artifact,
        search_artifacts,
        list_changes,
        kind_list_artifacts,
        kind_create_artifact,
        kind_search_artifacts,
    ),
    components(schemas(
        Artifact,
        ContentFormat,
        SearchResult,
        ListQuery,
        SearchBody,
        CreateBody,
        UpdateBody,
        ErrorResponse,
        ErrorDetail,
        ArtifactListResponse,
        SearchResultsResponse,
        ChangesResponse,
        HealthResponse,
        KindCreateBody,
        KindSearchBody,
        KindListQuery,
    )),
    tags(
        (name = "System", description = "System health and status"),
        (name = "Artifacts", description = "CRUD operations for artifacts"),
        (name = "Search", description = "Semantic search across artifacts"),
        (name = "Changes", description = "Track artifact changes over time"),
        (name = "Kinds", description = "Kind-scoped artifact operations")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

pub fn build_router(state: AppState, enable_api_docs: bool) -> Router {
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

    // Kind-scoped routes
    let kind_write_routes = Router::new()
        .route("/api/v1/kinds/{kind}/artifacts", post(kind_create_artifact))
        .route_layer(middleware::from_fn(require_write));

    let kind_read_routes = Router::new()
        .route("/api/v1/kinds/{kind}/artifacts", get(kind_list_artifacts))
        .route("/api/v1/kinds/{kind}/search", post(kind_search_artifacts));

    // Combine API routes with auth middleware
    let api_routes = Router::new()
        .merge(write_routes)
        .merge(read_routes)
        .merge(kind_write_routes)
        .merge(kind_read_routes)
        .route_layer(middleware::from_fn(auth_middleware))
        .layer(axum::Extension(api_key_auth));

    // MCP routes (with dynamic kind-specific tools)
    let mcp_routes = crate::mcp::mcp_router(
        state.db.clone(),
        state.embedding.clone(),
        state.registered_kinds.clone(),
    );

    let mut router = Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        .merge(mcp_routes);

    // Conditionally add API documentation
    if enable_api_docs {
        router =
            router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    router.layer(cors).with_state(state)
}
