//! Configuration types and loading
//!
//! Mirrors OpenProject's Settings and configuration patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Server configuration
    pub server: ServerConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Email/SMTP configuration
    pub email: EmailConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Feature flags
    pub features: FeatureFlags,

    /// Instance-specific settings
    pub instance: InstanceConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub pool_timeout_seconds: u64,
    pub statement_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub request_timeout_seconds: u64,
    pub max_body_size_bytes: usize,
    pub rails_relative_url_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// JWT secret for token signing
    pub jwt_secret: String,
    /// Token expiration in seconds
    pub token_expiration_seconds: u64,
    /// Session timeout in minutes
    pub session_timeout_minutes: u64,
    /// Enable self-registration
    pub self_registration: SelfRegistration,
    /// Password minimum length
    pub password_min_length: usize,
    /// OAuth/OIDC providers
    pub oauth_providers: Vec<OAuthProviderConfig>,
    /// LDAP configurations
    pub ldap: Vec<LdapConfig>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SelfRegistration {
    #[default]
    Disabled,
    Activation,
    Manual,
    Automatic,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OAuthProviderConfig {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LdapConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub bind_password: Option<String>,
    pub filter: Option<String>,
    pub attribute_mapping: LdapAttributeMapping,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LdapAttributeMapping {
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub admin: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub delivery_method: EmailDeliveryMethod,
    pub smtp: Option<SmtpConfig>,
    /// Microsoft Graph API configuration for Office 365 email
    pub ms_graph: Option<MsGraphConfig>,
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmailDeliveryMethod {
    #[default]
    Smtp,
    Sendmail,
    MsGraph,
    Test,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub authentication: Option<String>,
    pub enable_starttls: bool,
    pub ssl: bool,
}

/// Microsoft Graph API configuration for Office 365 email
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MsGraphConfig {
    /// Azure AD tenant ID
    pub tenant_id: String,
    /// Azure AD application (client) ID
    pub client_id: String,
    /// Azure AD client secret
    pub client_secret: String,
    /// User principal name (email) or object ID of the sender
    /// This user must have Mail.Send permission granted
    pub sender: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Local file storage path
    pub local_path: String,
    /// S3/MinIO configuration
    pub s3: Option<S3Config>,
    /// Maximum attachment size in bytes
    pub max_attachment_size: usize,
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: Option<String>,
    pub path_style: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureFlags {
    pub bim_enabled: bool,
    pub git_enabled: bool,
    pub ldap_enabled: bool,
    pub oauth_enabled: bool,
    pub webhooks_enabled: bool,
    pub api_v3_enabled: bool,
    pub collaborative_editing: bool,
    pub two_factor_auth: bool,
    // Business features
    pub boards_enabled: bool,
    pub budgets_enabled: bool,
    pub costs_enabled: bool,
    pub documents_enabled: bool,
    pub meetings_enabled: bool,
    pub team_planner_enabled: bool,
    pub backlogs_enabled: bool,
    pub reporting_enabled: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            // Core features - enabled by default
            api_v3_enabled: true,
            webhooks_enabled: true,
            oauth_enabled: true,
            // Business features - enabled by default
            boards_enabled: true,
            budgets_enabled: true,
            costs_enabled: true,
            documents_enabled: true,
            meetings_enabled: true,
            team_planner_enabled: true,
            backlogs_enabled: true,
            reporting_enabled: true,
            collaborative_editing: true,
            // Optional features - disabled by default
            bim_enabled: false,
            git_enabled: false,
            ldap_enabled: false,
            two_factor_auth: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceConfig {
    /// Application title
    pub app_title: String,
    /// Welcome text
    pub welcome_text: Option<String>,
    /// Default locale
    pub default_locale: String,
    /// Available locales
    pub available_locales: Vec<String>,
    /// Time zone
    pub timezone: String,
    /// Date format
    pub date_format: String,
    /// First day of week (0 = Sunday, 1 = Monday)
    pub first_day_of_week: u8,
    /// First week of year calculation
    pub first_week_of_year: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://openproject:openproject@localhost/openproject".to_string(),
                pool_size: 10,
                pool_timeout_seconds: 5,
                statement_timeout_seconds: 30,
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                request_timeout_seconds: 60,
                max_body_size_bytes: 100 * 1024 * 1024, // 100MB
                rails_relative_url_root: None,
            },
            auth: AuthConfig {
                jwt_secret: "change-me-in-production".to_string(),
                token_expiration_seconds: 86400, // 24 hours
                session_timeout_minutes: 30,
                self_registration: SelfRegistration::Disabled,
                password_min_length: 10,
                oauth_providers: vec![],
                ldap: vec![],
            },
            email: EmailConfig {
                delivery_method: EmailDeliveryMethod::Smtp,
                smtp: None,
                ms_graph: None,
                from_address: "openproject@example.com".to_string(),
                from_name: "OpenProject".to_string(),
            },
            storage: StorageConfig {
                local_path: "/var/openproject/assets".to_string(),
                s3: None,
                max_attachment_size: 256 * 1024 * 1024, // 256MB
                allowed_extensions: vec![],
            },
            features: FeatureFlags::default(),
            instance: InstanceConfig {
                app_title: "OpenProject".to_string(),
                welcome_text: None,
                default_locale: "en".to_string(),
                available_locales: vec!["en".to_string()],
                timezone: "UTC".to_string(),
                date_format: "%Y-%m-%d".to_string(),
                first_day_of_week: 1,
                first_week_of_year: 1,
            },
        }
    }
}

/// Dynamic settings (stored in database)
/// Mirrors OpenProject's Setting model
#[derive(Debug, Clone, Default)]
pub struct Settings {
    values: HashMap<String, SettingValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<String>),
    Hash(HashMap<String, String>),
}

impl Settings {
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(SettingValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(SettingValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.values.get(key) {
            Some(SettingValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: SettingValue) {
        self.values.insert(key.into(), value);
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable not set: {0}")]
    MissingEnvVar(String),
    #[error("Invalid value for {key}: {message}")]
    InvalidValue { key: String, message: String },
    #[error("Config file error: {0}")]
    FileError(String),
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Database - check DATABASE_URL first, then Railway's individual vars
        if let Ok(url) = std::env::var("DATABASE_URL") {
            config.database.url = url;
        } else if let Some(url) = Self::database_url_from_railway() {
            // Railway provides PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE
            config.database.url = url;
        }
        if let Ok(size) = std::env::var("DATABASE_POOL_SIZE") {
            config.database.pool_size = size.parse().unwrap_or(10);
        }

        // Server
        if let Ok(host) = std::env::var("HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("PORT") {
            config.server.port = port.parse().unwrap_or(8080);
        }
        if let Ok(root) = std::env::var("RAILS_RELATIVE_URL_ROOT") {
            config.server.rails_relative_url_root = Some(root);
        }

        // Auth
        if let Ok(secret) = std::env::var("SECRET_KEY_BASE") {
            config.auth.jwt_secret = secret;
        } else if let Ok(secret) = std::env::var("JWT_SECRET") {
            config.auth.jwt_secret = secret;
        }

        // Storage
        if let Ok(path) = std::env::var("OPENPROJECT_ATTACHMENTS_STORAGE_PATH") {
            config.storage.local_path = path;
        }

        // S3 storage
        if let Ok(bucket) = std::env::var("S3_BUCKET") {
            config.storage.s3 = Some(S3Config {
                bucket,
                region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                access_key_id: std::env::var("S3_ACCESS_KEY_ID").unwrap_or_default(),
                secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY").unwrap_or_default(),
                endpoint: std::env::var("S3_ENDPOINT").ok(),
                path_style: std::env::var("S3_PATH_STYLE")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
            });
        }

        // Email
        if let Ok(host) = std::env::var("SMTP_HOST") {
            config.email.smtp = Some(SmtpConfig {
                host,
                port: std::env::var("SMTP_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(587),
                username: std::env::var("SMTP_USERNAME").ok(),
                password: std::env::var("SMTP_PASSWORD").ok(),
                authentication: std::env::var("SMTP_AUTH").ok(),
                enable_starttls: std::env::var("SMTP_STARTTLS")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(true),
                ssl: std::env::var("SMTP_SSL")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
            });
        }
        if let Ok(from) = std::env::var("SMTP_FROM") {
            config.email.from_address = from;
        }

        // Microsoft Graph (Office 365) email
        if let Ok(tenant_id) = std::env::var("MS_GRAPH_TENANT_ID") {
            if let (Ok(client_id), Ok(client_secret), Ok(sender)) = (
                std::env::var("MS_GRAPH_CLIENT_ID"),
                std::env::var("MS_GRAPH_CLIENT_SECRET"),
                std::env::var("MS_GRAPH_SENDER"),
            ) {
                config.email.ms_graph = Some(MsGraphConfig {
                    tenant_id,
                    client_id,
                    client_secret,
                    sender,
                });
                // Auto-set delivery method to MsGraph if configured
                config.email.delivery_method = EmailDeliveryMethod::MsGraph;
                // Use sender as from_address if not explicitly set
                if config.email.from_address == "openproject@example.com" {
                    config.email.from_address = config.email.ms_graph.as_ref().unwrap().sender.clone();
                }
            }
        }

        // Instance
        if let Ok(title) = std::env::var("OPENPROJECT_APP_TITLE") {
            config.instance.app_title = title;
        }
        if let Ok(locale) = std::env::var("OPENPROJECT_DEFAULT_LOCALE") {
            config.instance.default_locale = locale;
        }
        if let Ok(tz) = std::env::var("TZ") {
            config.instance.timezone = tz;
        }

        // Features - all business features enabled by default
        // Can be disabled via environment variables
        let parse_bool = |v: String| v == "true" || v == "1" || v == "yes";

        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_BIM_ENABLED") {
            config.features.bim_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_GIT_ENABLED") {
            config.features.git_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_LDAP_ENABLED") {
            config.features.ldap_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_2FA_ENABLED") {
            config.features.two_factor_auth = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_BOARDS_ENABLED") {
            config.features.boards_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_BUDGETS_ENABLED") {
            config.features.budgets_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_COSTS_ENABLED") {
            config.features.costs_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_MEETINGS_ENABLED") {
            config.features.meetings_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_TEAM_PLANNER_ENABLED") {
            config.features.team_planner_enabled = parse_bool(v);
        }
        if let Ok(v) = std::env::var("OPENPROJECT_FEATURE_BACKLOGS_ENABLED") {
            config.features.backlogs_enabled = parse_bool(v);
        }

        Ok(config)
    }

    /// Build DATABASE_URL from Railway's individual PostgreSQL variables
    /// Railway provides: PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE
    pub fn database_url_from_railway() -> Option<String> {
        let host = std::env::var("PGHOST").ok()?;
        let port = std::env::var("PGPORT").unwrap_or_else(|_| "5432".to_string());
        let user = std::env::var("PGUSER").ok()?;
        let password = std::env::var("PGPASSWORD").ok()?;
        let database = std::env::var("PGDATABASE").ok()?;

        Some(format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        ))
    }

    /// Get the server address
    pub fn server_addr(&self) -> std::net::SocketAddr {
        use std::net::SocketAddr;
        let ip: std::net::IpAddr = self.server.host.parse().unwrap_or([0, 0, 0, 0].into());
        SocketAddr::new(ip, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.pool_size, 10);
    }

    #[test]
    fn test_settings() {
        let mut settings = Settings::default();
        settings.set("key1", SettingValue::String("value1".to_string()));
        settings.set("key2", SettingValue::Boolean(true));
        settings.set("key3", SettingValue::Integer(42));

        assert_eq!(settings.get_string("key1"), Some("value1"));
        assert_eq!(settings.get_bool("key2"), Some(true));
        assert_eq!(settings.get_int("key3"), Some(42));
    }

    #[test]
    fn test_server_addr() {
        let config = AppConfig::default();
        let addr = config.server_addr();
        assert_eq!(addr.port(), 8080);
    }
}
