//! OpenProject RS Server
//!
//! Main entry point for the OpenProject Rust server.

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,op_server=debug,op_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();

    // TODO: Load config from environment/files
    // let config = op_core::config::AppConfig::load()?;

    // TODO: Initialize database pool
    // let pool = op_db::create_pool(&config.database).await?;

    // TODO: Build application router
    // let app = op_api::router(pool);

    let app = axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/api/v3", axum::routing::get(api_root));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Starting OpenProject RS on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn api_root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "_type": "Root",
        "instanceName": "OpenProject RS",
        "coreVersion": env!("CARGO_PKG_VERSION"),
        "_links": {
            "self": { "href": "/api/v3" },
            "configuration": { "href": "/api/v3/configuration" },
            "user": { "href": "/api/v3/users/me" }
        }
    }))
}
