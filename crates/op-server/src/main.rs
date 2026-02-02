//! OpenProject RS Server
//!
//! Production-ready HTTP server for OpenProject Rust implementation.

use std::sync::Arc;

use axum::{
    middleware,
    routing::get,
    Json, Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use op_core::config::AppConfig;
use op_db::{Database, DatabaseConfig};

mod health;
mod metrics;

use health::{AppState, HealthChecker, HealthConfig};
use metrics::Metrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    init_tracing();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config from env: {}, using defaults", e);
        AppConfig::default()
    });

    info!(
        version = env!("CARGO_PKG_VERSION"),
        host = %config.server.host,
        port = config.server.port,
        "Starting OpenProject RS"
    );

    // Connect to database
    let db_config = DatabaseConfig::with_url(&config.database.url);
    let db = match Database::connect(&db_config).await {
        Ok(db) => {
            info!("Connected to database");
            Some(db)
        }
        Err(e) => {
            tracing::warn!("Failed to connect to database: {}. Running without database.", e);
            None
        }
    };

    // Initialize components
    let metrics = Arc::new(Metrics::new());
    let mut health_checker = HealthChecker::new(HealthConfig::default());
    if let Some(ref db) = db {
        health_checker = health_checker.with_pool(db.pool().clone());
    }

    let app_state = Arc::new(AppState {
        health: Arc::new(health_checker),
        config: config.clone(),
        db: db.map(|d| d.pool().clone()),
    });

    // Build router
    let app = build_router(app_state.clone(), metrics.clone());

    // Start server
    let addr = config.server_addr();
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Initialize tracing/logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,op_server=debug,op_api=debug,tower_http=debug".into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .init();
}

/// Build the application router
fn build_router(state: Arc<AppState>, metrics: Arc<Metrics>) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(health::default_health_check))
        .route("/health_checks/default", get(health::default_health_check))
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .route("/health/full", get(health::health))
        .with_state(state.clone());

    // Metrics routes
    let metrics_routes = Router::new()
        .route("/metrics", get(metrics::prometheus_metrics))
        .route("/metrics.json", get(metrics::json_metrics))
        .with_state(metrics.clone());

    // API v3 routes
    let api_routes = Router::new()
        .route("/", get(api_root))
        .route("/configuration", get(api_configuration))
        .route("/users/me", get(api_current_user));

    // Main router
    Router::new()
        .merge(health_routes)
        .merge(metrics_routes)
        .nest("/api/v3", api_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .layer(middleware::from_fn_with_state(
            metrics,
            metrics::metrics_middleware,
        ))
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}

/// API v3 root endpoint
async fn api_root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "_type": "Root",
        "instanceName": "OpenProject RS",
        "coreVersion": env!("CARGO_PKG_VERSION"),
        "_links": {
            "self": { "href": "/api/v3" },
            "configuration": { "href": "/api/v3/configuration" },
            "user": { "href": "/api/v3/users/me" },
            "users": { "href": "/api/v3/users" },
            "projects": { "href": "/api/v3/projects" },
            "workPackages": { "href": "/api/v3/work_packages" },
            "statuses": { "href": "/api/v3/statuses" },
            "types": { "href": "/api/v3/types" },
            "priorities": { "href": "/api/v3/priorities" },
            "queries": { "href": "/api/v3/queries" }
        }
    }))
}

/// API configuration endpoint
async fn api_configuration() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "_type": "Configuration",
        "maximumAttachmentFileSize": 256 * 1024 * 1024,
        "perPageOptions": [20, 100],
        "dateFormat": "%Y-%m-%d",
        "timeFormat": "%H:%M",
        "startOfWeek": 1,
        "activeFeatureFlags": [
            "bim",
            "boards",
            "budgets",
            "costs",
            "documents",
            "meeting",
            "openid_connect",
            "reporting",
            "team_planner",
            "webhooks",
            "wiki"
        ],
        "_links": {
            "self": { "href": "/api/v3/configuration" }
        }
    }))
}

/// Current user endpoint (returns anonymous for now)
async fn api_current_user() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "_type": "User",
        "id": 0,
        "login": "anonymous",
        "firstName": "Anonymous",
        "lastName": "User",
        "admin": false,
        "status": "active",
        "_links": {
            "self": { "href": "/api/v3/users/me" }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let metrics = Arc::new(Metrics::new());
        let health_checker = Arc::new(HealthChecker::new(HealthConfig::default()));
        let config = AppConfig::default();

        let state = Arc::new(AppState {
            health: health_checker,
            config,
            db: None,
        });

        build_router(state, metrics)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_root() {
        let app = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
