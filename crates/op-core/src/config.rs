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
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmailDeliveryMethod {
    #[default]
    Smtp,
    Sendmail,
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FeatureFlags {
    pub bim_enabled: bool,
    pub git_enabled: bool,
    pub ldap_enabled: bool,
    pub oauth_enabled: bool,
    pub webhooks_enabled: bool,
    pub api_v3_enabled: bool,
    pub collaborative_editing: bool,
    pub two_factor_auth: bool,
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
