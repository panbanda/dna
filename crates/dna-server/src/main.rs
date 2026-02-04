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
    let api_docs_enabled = state.server_config.api_docs.enabled;
    let bind_addr = state
        .server_config
        .bind
        .clone()
        .unwrap_or_else(|| "0.0.0.0:3000".to_string());

    let app = api::build_router(state);

    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        tracing::info!("Starting in Lambda mode");
        if api_docs_enabled {
            tracing::info!("API documentation available at /docs");
        }
        lambda_http::run(app)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    } else {
        tracing::info!("Starting server on {}", bind_addr);
        if api_docs_enabled {
            tracing::info!("API documentation available at http://{}/docs", bind_addr);
        }
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
