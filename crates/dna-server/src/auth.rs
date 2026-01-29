use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyScope {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub scope: KeyScope,
}

#[derive(Clone)]
pub struct ApiKeyAuth {
    read_only_keys: Vec<String>,
    read_write_keys: Vec<String>,
    trust_proxy_auth: bool,
}

impl ApiKeyAuth {
    pub fn from_env() -> Self {
        let read_only_keys = std::env::var("DNA_SERVER__API_KEYS_RO")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let read_write_keys = std::env::var("DNA_SERVER__API_KEYS_RW")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let trust_proxy_auth = std::env::var("DNA_SERVER__TRUST_PROXY_AUTH")
            .unwrap_or_default()
            .eq_ignore_ascii_case("true");

        Self {
            read_only_keys,
            read_write_keys,
            trust_proxy_auth,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.read_only_keys.is_empty() || !self.read_write_keys.is_empty()
    }

    pub fn authenticate(&self, token: &str) -> Option<AuthContext> {
        let token_bytes = token.as_bytes();

        // Check read-write keys first (higher privilege)
        let mut is_read_write = false;
        for key in &self.read_write_keys {
            if constant_time_eq(token_bytes, key.as_bytes()) {
                is_read_write = true;
            }
        }
        if is_read_write {
            return Some(AuthContext {
                scope: KeyScope::ReadWrite,
            });
        }

        // Check read-only keys
        let mut is_read_only = false;
        for key in &self.read_only_keys {
            if constant_time_eq(token_bytes, key.as_bytes()) {
                is_read_only = true;
            }
        }
        if is_read_only {
            return Some(AuthContext {
                scope: KeyScope::ReadOnly,
            });
        }

        None
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let auth = request.extensions().get::<ApiKeyAuth>().cloned();

    let Some(auth) = auth else {
        return next.run(request).await;
    };

    if !auth.is_enabled() {
        return next.run(request).await;
    }

    // Check for API Gateway authorizer header (Lambda behind a proxy)
    // Only trusted when explicitly opted in via DNA_SERVER__TRUST_PROXY_AUTH=true
    if auth.trust_proxy_auth {
        if let Some(scope_header) = request.headers().get("x-auth-scope") {
            if let Ok(scope_str) = scope_header.to_str() {
                let scope = match scope_str {
                    "read-write" | "rw" => KeyScope::ReadWrite,
                    _ => KeyScope::ReadOnly,
                };
                let mut request = request;
                request.extensions_mut().insert(AuthContext { scope });
                return next.run(request).await;
            }
        }
    }

    // Fall back to Bearer token auth
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(token) => match auth.authenticate(token) {
            Some(context) => {
                let mut request = request;
                request.extensions_mut().insert(context);
                next.run(request).await
            },
            None => (StatusCode::UNAUTHORIZED, "Invalid API key").into_response(),
        },
        None => (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response(),
    }
}

/// Middleware to enforce write access
pub async fn require_write(request: Request, next: Next) -> Response {
    if let Some(context) = request.extensions().get::<AuthContext>() {
        if context.scope != KeyScope::ReadWrite {
            return (StatusCode::FORBIDDEN, "Write access required").into_response();
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, middleware, routing::get, Router};
    use tower::ServiceExt;

    fn test_auth(trust_proxy: bool) -> ApiKeyAuth {
        ApiKeyAuth {
            read_only_keys: vec!["ro-key".to_string()],
            read_write_keys: vec!["rw-key".to_string()],
            trust_proxy_auth: trust_proxy,
        }
    }

    fn test_router(auth: ApiKeyAuth) -> Router {
        Router::new()
            .route("/test", get(|| async { "ok" }))
            .route_layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(auth))
    }

    #[tokio::test]
    async fn x_auth_scope_rejected_when_trust_proxy_auth_disabled() {
        let app = test_router(test_auth(false));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-auth-scope", "read-write")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn x_auth_scope_accepted_when_trust_proxy_auth_enabled() {
        let app = test_router(test_auth(true));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-auth-scope", "read-write")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_bearer_token_authenticates() {
        let app = test_router(test_auth(false));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("authorization", "Bearer rw-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_bearer_token_rejected() {
        let app = test_router(test_auth(false));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("authorization", "Bearer wrong-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn authenticate_returns_correct_scope() {
        let auth = test_auth(false);

        let rw = auth.authenticate("rw-key").unwrap();
        assert_eq!(rw.scope, KeyScope::ReadWrite);

        let ro = auth.authenticate("ro-key").unwrap();
        assert_eq!(ro.scope, KeyScope::ReadOnly);

        assert!(auth.authenticate("bad-key").is_none());
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }
}
