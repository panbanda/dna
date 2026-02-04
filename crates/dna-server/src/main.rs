use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod auth;
mod mcp;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = state::AppState::from_env().await?;

    // API docs are enabled by default, can be disabled with DNA_SERVER__API_DOCS=false
    let enable_api_docs = std::env::var("DNA_SERVER__API_DOCS")
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    let app = api::build_router(state, enable_api_docs);

    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        tracing::info!("Starting in Lambda mode");
        if enable_api_docs {
            tracing::info!("API documentation available at /docs");
        }
        lambda_http::run(app)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    } else {
        let addr = std::env::var("DNA_SERVER__BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        tracing::info!("Starting server on {}", addr);
        if enable_api_docs {
            tracing::info!("API documentation available at http://{}/docs", addr);
        }
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
