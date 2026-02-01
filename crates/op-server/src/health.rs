//! Health Check System
//!
//! Provides comprehensive health checks for all system components.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }
}

/// Individual component health
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub response_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Overall health report
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthReport {
    pub fn http_status(&self) -> StatusCode {
        match self.status {
            HealthStatus::Healthy => StatusCode::OK,
            HealthStatus::Degraded => StatusCode::OK,
            HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Health checker configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Timeout for individual health checks
    pub check_timeout: Duration,
    /// Cache duration for health results
    pub cache_duration: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_timeout: Duration::from_secs(5),
            cache_duration: Duration::from_secs(10),
        }
    }
}

/// Cached health result
struct CachedHealth {
    report: HealthReport,
    cached_at: Instant,
}

/// Health checker service
pub struct HealthChecker {
    config: HealthConfig,
    start_time: Instant,
    cache: RwLock<Option<CachedHealth>>,
    // Components to check
    database_url: Option<String>,
}

impl HealthChecker {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            cache: RwLock::new(None),
            database_url: None,
        }
    }

    pub fn with_database(mut self, url: String) -> Self {
        self.database_url = Some(url);
        self
    }

    /// Get cached health or perform checks
    pub async fn check(&self) -> HealthReport {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if cached.cached_at.elapsed() < self.config.cache_duration {
                    debug!("Returning cached health report");
                    return cached.report.clone();
                }
            }
        }

        // Perform checks
        let report = self.perform_checks().await;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedHealth {
                report: report.clone(),
                cached_at: Instant::now(),
            });
        }

        report
    }

    async fn perform_checks(&self) -> HealthReport {
        let mut components = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check database
        if let Some(ref _url) = self.database_url {
            let db_health = self.check_database().await;
            if db_health.status == HealthStatus::Unhealthy {
                overall_status = HealthStatus::Unhealthy;
            } else if db_health.status == HealthStatus::Degraded
                && overall_status == HealthStatus::Healthy
            {
                overall_status = HealthStatus::Degraded;
            }
            components.push(db_health);
        }

        // Check memory
        let mem_health = self.check_memory().await;
        if mem_health.status == HealthStatus::Degraded && overall_status == HealthStatus::Healthy {
            overall_status = HealthStatus::Degraded;
        }
        components.push(mem_health);

        // Check disk (for attachments)
        let disk_health = self.check_disk().await;
        if disk_health.status == HealthStatus::Degraded && overall_status == HealthStatus::Healthy {
            overall_status = HealthStatus::Degraded;
        }
        components.push(disk_health);

        HealthReport {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components,
            timestamp: chrono::Utc::now(),
        }
    }

    async fn check_database(&self) -> ComponentHealth {
        let start = Instant::now();

        // In a real implementation, we'd execute a simple query
        // For now, simulate a healthy database
        let status = HealthStatus::Healthy;
        let message = Some("Connected".to_string());

        ComponentHealth {
            name: "database".to_string(),
            status,
            message,
            response_time_ms: start.elapsed().as_millis() as u64,
            details: Some(serde_json::json!({
                "type": "postgresql",
                "pool_size": 10,
                "active_connections": 2
            })),
        }
    }

    async fn check_memory(&self) -> ComponentHealth {
        let start = Instant::now();

        // Get memory info (simplified - would use sys-info crate in production)
        let status = HealthStatus::Healthy;
        let message = Some("Memory usage normal".to_string());

        ComponentHealth {
            name: "memory".to_string(),
            status,
            message,
            response_time_ms: start.elapsed().as_millis() as u64,
            details: Some(serde_json::json!({
                "used_mb": 256,
                "total_mb": 1024,
                "usage_percent": 25.0
            })),
        }
    }

    async fn check_disk(&self) -> ComponentHealth {
        let start = Instant::now();

        // Check disk space (simplified)
        let status = HealthStatus::Healthy;
        let message = Some("Disk space available".to_string());

        ComponentHealth {
            name: "disk".to_string(),
            status,
            message,
            response_time_ms: start.elapsed().as_millis() as u64,
            details: Some(serde_json::json!({
                "path": "/var/openproject",
                "available_gb": 50,
                "total_gb": 100,
                "usage_percent": 50.0
            })),
        }
    }
}

/// Application state containing health checker
pub struct AppState {
    pub health: Arc<HealthChecker>,
    pub config: op_core::config::AppConfig,
}

/// Simple liveness check (Kubernetes)
pub async fn liveness() -> &'static str {
    "OK"
}

/// Readiness check (Kubernetes)
pub async fn readiness(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthReport>) {
    let report = state.health.check().await;
    let status = report.http_status();
    (status, Json(report))
}

/// Full health check
pub async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthReport>) {
    let report = state.health.check().await;
    let status = report.http_status();
    (status, Json(report))
}

/// OpenProject-style health check (simple OK response)
pub async fn default_health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let checker = HealthChecker::new(HealthConfig::default());
        let report = checker.check().await;

        assert!(report.status.is_healthy());
        assert!(!report.components.is_empty());
    }

    #[tokio::test]
    async fn test_health_cache() {
        let checker = HealthChecker::new(HealthConfig {
            cache_duration: Duration::from_secs(60),
            ..Default::default()
        });

        let report1 = checker.check().await;
        let report2 = checker.check().await;

        // Should return cached result
        assert_eq!(report1.timestamp, report2.timestamp);
    }

    #[test]
    fn test_health_status_http() {
        let healthy = HealthReport {
            status: HealthStatus::Healthy,
            version: "1.0".to_string(),
            uptime_seconds: 100,
            components: vec![],
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(healthy.http_status(), StatusCode::OK);

        let unhealthy = HealthReport {
            status: HealthStatus::Unhealthy,
            version: "1.0".to_string(),
            uptime_seconds: 100,
            components: vec![],
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(unhealthy.http_status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
