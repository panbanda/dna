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
    let app = api::build_router(state);

    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        tracing::info!("Starting in Lambda mode");
        lambda_http::run(app)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    } else {
        let addr = std::env::var("DNA_SERVER__BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        tracing::info!("Starting server on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
