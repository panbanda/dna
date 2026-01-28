use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;

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
    read_only_keys: HashSet<String>,
    read_write_keys: HashSet<String>,
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

        Self {
            read_only_keys,
            read_write_keys,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.read_only_keys.is_empty() || !self.read_write_keys.is_empty()
    }

    pub fn authenticate(&self, token: &str) -> Option<AuthContext> {
        if self.read_write_keys.contains(token) {
            Some(AuthContext {
                scope: KeyScope::ReadWrite,
            })
        } else if self.read_only_keys.contains(token) {
            Some(AuthContext {
                scope: KeyScope::ReadOnly,
            })
        } else {
            None
        }
    }
}

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let auth = request.extensions().get::<ApiKeyAuth>().cloned();

    let Some(auth) = auth else {
        return next.run(request).await;
    };

    if !auth.is_enabled() {
        return next.run(request).await;
    }

    // Check for API Gateway authorizer header first (Lambda)
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
