//! Database connection pool management
//!
//! Provides PostgreSQL connection pooling using SQLx.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Idle timeout for connections in seconds
    pub idle_timeout_secs: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/openproject".to_string()),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
        }
    }
}

impl DatabaseConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/openproject".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            connect_timeout_secs: std::env::var("DB_CONNECT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            idle_timeout_secs: std::env::var("DB_IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600),
            max_lifetime_secs: std::env::var("DB_MAX_LIFETIME")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1800),
        }
    }

    /// Create config with a specific URL
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
}

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
            .connect(&config.url)
            .await?;

        tracing::info!(
            "Database pool created with {} max connections",
            config.max_connections
        );

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Check if the database is reachable
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Close the connection pool
    pub async fn close(&self) {
        self.pool.close().await;
        tracing::info!("Database pool closed");
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }

    #[test]
    fn test_config_with_url() {
        let config = DatabaseConfig::with_url("postgres://test:test@localhost/test");
        assert_eq!(config.url, "postgres://test:test@localhost/test");
        assert_eq!(config.max_connections, 10);
    }
}
