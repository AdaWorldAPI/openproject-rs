//! Metrics and Observability
//!
//! Provides Prometheus-compatible metrics for monitoring.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use tracing::{debug, info_span, Instrument};

/// Metrics collector
pub struct Metrics {
    /// Total HTTP requests
    pub http_requests_total: AtomicU64,
    /// HTTP requests by status code (2xx, 4xx, 5xx)
    pub http_requests_2xx: AtomicU64,
    pub http_requests_4xx: AtomicU64,
    pub http_requests_5xx: AtomicU64,
    /// Total request duration in milliseconds
    pub http_request_duration_ms_total: AtomicU64,
    /// Active connections
    pub active_connections: AtomicU64,
    /// Database queries
    pub db_queries_total: AtomicU64,
    pub db_query_duration_ms_total: AtomicU64,
    /// Cache hits/misses
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    /// Background jobs
    pub jobs_processed: AtomicU64,
    pub jobs_failed: AtomicU64,
    /// Start time for uptime calculation
    start_time: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            http_requests_total: AtomicU64::new(0),
            http_requests_2xx: AtomicU64::new(0),
            http_requests_4xx: AtomicU64::new(0),
            http_requests_5xx: AtomicU64::new(0),
            http_request_duration_ms_total: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            db_queries_total: AtomicU64::new(0),
            db_query_duration_ms_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            jobs_processed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record an HTTP request
    pub fn record_request(&self, status: StatusCode, duration_ms: u64) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
        self.http_request_duration_ms_total
            .fetch_add(duration_ms, Ordering::Relaxed);

        let code = status.as_u16();
        if (200..300).contains(&code) {
            self.http_requests_2xx.fetch_add(1, Ordering::Relaxed);
        } else if (400..500).contains(&code) {
            self.http_requests_4xx.fetch_add(1, Ordering::Relaxed);
        } else if code >= 500 {
            self.http_requests_5xx.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record a database query
    pub fn record_db_query(&self, duration_ms: u64) {
        self.db_queries_total.fetch_add(1, Ordering::Relaxed);
        self.db_query_duration_ms_total
            .fetch_add(duration_ms, Ordering::Relaxed);
    }

    /// Record cache access
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record job processing
    pub fn record_job_success(&self) {
        self.jobs_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_job_failure(&self) {
        self.jobs_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // HTTP metrics
        output.push_str("# HELP http_requests_total Total number of HTTP requests\n");
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!(
            "http_requests_total {}\n",
            self.http_requests_total.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP http_requests_by_status HTTP requests by status code range\n");
        output.push_str("# TYPE http_requests_by_status counter\n");
        output.push_str(&format!(
            "http_requests_by_status{{status=\"2xx\"}} {}\n",
            self.http_requests_2xx.load(Ordering::Relaxed)
        ));
        output.push_str(&format!(
            "http_requests_by_status{{status=\"4xx\"}} {}\n",
            self.http_requests_4xx.load(Ordering::Relaxed)
        ));
        output.push_str(&format!(
            "http_requests_by_status{{status=\"5xx\"}} {}\n",
            self.http_requests_5xx.load(Ordering::Relaxed)
        ));

        output.push_str(
            "# HELP http_request_duration_ms_total Total HTTP request duration in milliseconds\n",
        );
        output.push_str("# TYPE http_request_duration_ms_total counter\n");
        output.push_str(&format!(
            "http_request_duration_ms_total {}\n",
            self.http_request_duration_ms_total.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP active_connections Current number of active connections\n");
        output.push_str("# TYPE active_connections gauge\n");
        output.push_str(&format!(
            "active_connections {}\n",
            self.active_connections.load(Ordering::Relaxed)
        ));

        // Database metrics
        output.push_str("# HELP db_queries_total Total number of database queries\n");
        output.push_str("# TYPE db_queries_total counter\n");
        output.push_str(&format!(
            "db_queries_total {}\n",
            self.db_queries_total.load(Ordering::Relaxed)
        ));

        output.push_str(
            "# HELP db_query_duration_ms_total Total database query duration in milliseconds\n",
        );
        output.push_str("# TYPE db_query_duration_ms_total counter\n");
        output.push_str(&format!(
            "db_query_duration_ms_total {}\n",
            self.db_query_duration_ms_total.load(Ordering::Relaxed)
        ));

        // Cache metrics
        output.push_str("# HELP cache_hits_total Total cache hits\n");
        output.push_str("# TYPE cache_hits_total counter\n");
        output.push_str(&format!(
            "cache_hits_total {}\n",
            self.cache_hits.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP cache_misses_total Total cache misses\n");
        output.push_str("# TYPE cache_misses_total counter\n");
        output.push_str(&format!(
            "cache_misses_total {}\n",
            self.cache_misses.load(Ordering::Relaxed)
        ));

        // Job metrics
        output.push_str("# HELP jobs_processed_total Total jobs processed successfully\n");
        output.push_str("# TYPE jobs_processed_total counter\n");
        output.push_str(&format!(
            "jobs_processed_total {}\n",
            self.jobs_processed.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP jobs_failed_total Total jobs that failed\n");
        output.push_str("# TYPE jobs_failed_total counter\n");
        output.push_str(&format!(
            "jobs_failed_total {}\n",
            self.jobs_failed.load(Ordering::Relaxed)
        ));

        // Uptime
        output.push_str("# HELP uptime_seconds Server uptime in seconds\n");
        output.push_str("# TYPE uptime_seconds gauge\n");
        output.push_str(&format!("uptime_seconds {}\n", self.uptime_seconds()));

        output
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "http": {
                "requests_total": self.http_requests_total.load(Ordering::Relaxed),
                "requests_2xx": self.http_requests_2xx.load(Ordering::Relaxed),
                "requests_4xx": self.http_requests_4xx.load(Ordering::Relaxed),
                "requests_5xx": self.http_requests_5xx.load(Ordering::Relaxed),
                "request_duration_ms_total": self.http_request_duration_ms_total.load(Ordering::Relaxed),
                "active_connections": self.active_connections.load(Ordering::Relaxed),
            },
            "database": {
                "queries_total": self.db_queries_total.load(Ordering::Relaxed),
                "query_duration_ms_total": self.db_query_duration_ms_total.load(Ordering::Relaxed),
            },
            "cache": {
                "hits": self.cache_hits.load(Ordering::Relaxed),
                "misses": self.cache_misses.load(Ordering::Relaxed),
            },
            "jobs": {
                "processed": self.jobs_processed.load(Ordering::Relaxed),
                "failed": self.jobs_failed.load(Ordering::Relaxed),
            },
            "uptime_seconds": self.uptime_seconds(),
        })
    }
}

/// Metrics middleware
pub async fn metrics_middleware(
    State(metrics): State<Arc<Metrics>>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().path().to_string();

    metrics.connection_opened();

    let response = next
        .run(request)
        .instrument(info_span!("http_request", %method, %uri))
        .await;

    let duration = start.elapsed();
    let status = response.status();

    debug!(
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    metrics.record_request(status, duration.as_millis() as u64);
    metrics.connection_closed();

    response
}

/// Handler for /metrics endpoint (Prometheus format)
pub async fn prometheus_metrics(State(metrics): State<Arc<Metrics>>) -> String {
    metrics.export_prometheus()
}

/// Handler for /metrics.json endpoint
pub async fn json_metrics(State(metrics): State<Arc<Metrics>>) -> axum::Json<serde_json::Value> {
    axum::Json(metrics.export_json())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_request() {
        let metrics = Metrics::new();

        metrics.record_request(StatusCode::OK, 50);
        metrics.record_request(StatusCode::NOT_FOUND, 10);
        metrics.record_request(StatusCode::INTERNAL_SERVER_ERROR, 100);

        assert_eq!(metrics.http_requests_total.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.http_requests_2xx.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.http_requests_4xx.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.http_requests_5xx.load(Ordering::Relaxed), 1);
        assert_eq!(
            metrics.http_request_duration_ms_total.load(Ordering::Relaxed),
            160
        );
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = Metrics::new();
        metrics.record_request(StatusCode::OK, 50);

        let output = metrics.export_prometheus();
        assert!(output.contains("http_requests_total 1"));
        assert!(output.contains("uptime_seconds"));
    }

    #[test]
    fn test_json_export() {
        let metrics = Metrics::new();
        metrics.record_request(StatusCode::OK, 50);
        metrics.record_db_query(10);

        let json = metrics.export_json();
        assert_eq!(json["http"]["requests_total"], 1);
        assert_eq!(json["database"]["queries_total"], 1);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = Metrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        assert_eq!(metrics.cache_hits.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.cache_misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_job_metrics() {
        let metrics = Metrics::new();

        metrics.record_job_success();
        metrics.record_job_success();
        metrics.record_job_failure();

        assert_eq!(metrics.jobs_processed.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.jobs_failed.load(Ordering::Relaxed), 1);
    }
}
