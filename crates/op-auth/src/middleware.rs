//! Authentication Middleware
//!
//! Provides axum middleware for authenticating requests using various strategies.

use crate::jwt::{extract_bearer_token, JwtService};
use crate::permissions::CurrentUser;
use crate::session::{extract_session_id, CookieConfig, SessionStore};

use std::sync::Arc;
use thiserror::Error;

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication required")]
    Required,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Authentication strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStrategy {
    /// JWT Bearer token
    Jwt,
    /// API key
    ApiKey,
    /// Session cookie
    Session,
    /// Basic auth
    Basic,
    /// OAuth2
    OAuth2,
}

/// Authentication result
#[derive(Debug)]
pub enum AuthResult {
    /// Successfully authenticated
    Authenticated(CurrentUser),
    /// Anonymous user (allowed for public endpoints)
    Anonymous,
    /// Authentication failed
    Failed(AuthError),
}

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    /// JWT service for token validation
    pub jwt_service: Option<Arc<JwtService>>,
    /// Session store for session-based auth
    pub session_store: Option<Arc<dyn SessionStore>>,
    /// Cookie configuration
    pub cookie_config: CookieConfig,
    /// Whether to allow anonymous access
    pub allow_anonymous: bool,
    /// Enabled authentication strategies (in order of preference)
    pub strategies: Vec<AuthStrategy>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_service: None,
            session_store: None,
            cookie_config: CookieConfig::default(),
            allow_anonymous: false,
            strategies: vec![
                AuthStrategy::Jwt,
                AuthStrategy::ApiKey,
                AuthStrategy::Session,
            ],
        }
    }
}

impl AuthConfig {
    /// Create config with JWT authentication
    pub fn jwt(secret: &[u8]) -> Self {
        Self {
            jwt_service: Some(Arc::new(JwtService::new(secret))),
            strategies: vec![AuthStrategy::Jwt],
            ..Default::default()
        }
    }

    /// Create config with session authentication
    pub fn session(store: Arc<dyn SessionStore>) -> Self {
        Self {
            session_store: Some(store),
            strategies: vec![AuthStrategy::Session],
            ..Default::default()
        }
    }

    /// Add JWT support
    pub fn with_jwt(mut self, service: JwtService) -> Self {
        self.jwt_service = Some(Arc::new(service));
        if !self.strategies.contains(&AuthStrategy::Jwt) {
            self.strategies.insert(0, AuthStrategy::Jwt);
        }
        self
    }

    /// Add session support
    pub fn with_session(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.session_store = Some(store);
        if !self.strategies.contains(&AuthStrategy::Session) {
            self.strategies.push(AuthStrategy::Session);
        }
        self
    }

    /// Allow anonymous access
    pub fn with_anonymous(mut self) -> Self {
        self.allow_anonymous = true;
        self
    }

    /// Set cookie configuration
    pub fn with_cookie_config(mut self, config: CookieConfig) -> Self {
        self.cookie_config = config;
        self
    }
}

/// Authenticator for validating requests
pub struct Authenticator {
    config: AuthConfig,
}

impl Authenticator {
    /// Create a new authenticator
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Authenticate a request using available headers
    pub async fn authenticate(&self, headers: &RequestHeaders) -> AuthResult {
        for strategy in &self.config.strategies {
            match strategy {
                AuthStrategy::Jwt => {
                    if let Some(result) = self.try_jwt_auth(headers).await {
                        return result;
                    }
                }
                AuthStrategy::ApiKey => {
                    if let Some(result) = self.try_api_key_auth(headers).await {
                        return result;
                    }
                }
                AuthStrategy::Session => {
                    if let Some(result) = self.try_session_auth(headers).await {
                        return result;
                    }
                }
                AuthStrategy::Basic => {
                    if let Some(result) = self.try_basic_auth(headers).await {
                        return result;
                    }
                }
                AuthStrategy::OAuth2 => {
                    // OAuth2 requires more complex flow
                    continue;
                }
            }
        }

        // No authentication found
        if self.config.allow_anonymous {
            AuthResult::Anonymous
        } else {
            AuthResult::Failed(AuthError::Required)
        }
    }

    /// Try JWT authentication
    async fn try_jwt_auth(&self, headers: &RequestHeaders) -> Option<AuthResult> {
        let jwt_service = self.config.jwt_service.as_ref()?;

        let auth_header = headers.authorization.as_ref()?;
        let token = extract_bearer_token(auth_header)?;

        match jwt_service.validate_token(token) {
            Ok(claims) => {
                let user_id: i64 = claims.sub.parse().ok()?;
                let user = CurrentUser::new(
                    user_id,
                    claims.login.unwrap_or_else(|| format!("user_{}", user_id)),
                    claims.email.unwrap_or_default(),
                );
                Some(AuthResult::Authenticated(user))
            }
            Err(crate::jwt::JwtError::Expired) => {
                Some(AuthResult::Failed(AuthError::TokenExpired))
            }
            Err(_) => Some(AuthResult::Failed(AuthError::InvalidCredentials)),
        }
    }

    /// Try API key authentication
    async fn try_api_key_auth(&self, headers: &RequestHeaders) -> Option<AuthResult> {
        // Check X-OpenProject-API-Key header
        let api_key = headers.api_key.as_ref()?;

        // In a real implementation, we would look up the API key in the database
        // For now, we just validate the format
        if api_key.len() >= 20 {
            // Mock user for API key auth
            let user = CurrentUser::new(1, "api_user", "api@example.com");
            Some(AuthResult::Authenticated(user))
        } else {
            Some(AuthResult::Failed(AuthError::InvalidCredentials))
        }
    }

    /// Try session authentication
    async fn try_session_auth(&self, headers: &RequestHeaders) -> Option<AuthResult> {
        let session_store = self.config.session_store.as_ref()?;
        let cookie = headers.cookie.as_ref()?;

        let session_id = extract_session_id(cookie, &self.config.cookie_config.name)?;
        let session = session_store.get(&session_id)?;

        if let Some(user_id) = session.user_id {
            let user = CurrentUser::new(user_id, "session_user", "session@example.com");
            Some(AuthResult::Authenticated(user))
        } else {
            None // Anonymous session, continue to next strategy
        }
    }

    /// Try basic authentication
    async fn try_basic_auth(&self, headers: &RequestHeaders) -> Option<AuthResult> {
        let auth_header = headers.authorization.as_ref()?;

        if !auth_header.to_lowercase().starts_with("basic ") {
            return None;
        }

        let encoded = &auth_header[6..];
        let decoded = base64_decode(encoded)?;
        let credentials = String::from_utf8(decoded).ok()?;
        let (username, password) = credentials.split_once(':')?;

        // In a real implementation, we would verify against the database
        // For now, just check if username/password are non-empty
        if !username.is_empty() && !password.is_empty() {
            let user = CurrentUser::new(1, username, format!("{}@example.com", username));
            Some(AuthResult::Authenticated(user))
        } else {
            Some(AuthResult::Failed(AuthError::InvalidCredentials))
        }
    }
}

/// Request headers relevant for authentication
#[derive(Debug, Default)]
pub struct RequestHeaders {
    pub authorization: Option<String>,
    pub api_key: Option<String>,
    pub cookie: Option<String>,
    pub x_forwarded_for: Option<String>,
    pub user_agent: Option<String>,
}

impl RequestHeaders {
    /// Create from a list of header key-value pairs
    pub fn from_pairs(pairs: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        let mut headers = Self::default();

        for (name, value) in pairs {
            let name_lower = name.as_ref().to_lowercase();
            let value = value.as_ref().to_string();

            match name_lower.as_str() {
                "authorization" => headers.authorization = Some(value),
                "x-openproject-api-key" => headers.api_key = Some(value),
                "cookie" => headers.cookie = Some(value),
                "x-forwarded-for" => headers.x_forwarded_for = Some(value),
                "user-agent" => headers.user_agent = Some(value),
                _ => {}
            }
        }

        headers
    }
}

/// Simple base64 decode
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_authentication() {
        let jwt_service = JwtService::new(b"test-secret-key-at-least-32-bytes");
        let token = jwt_service
            .create_token(1, Some("test@example.com".into()), Some("testuser".into()), 3600)
            .unwrap();

        let config = AuthConfig::default().with_jwt(JwtService::new(b"test-secret-key-at-least-32-bytes"));
        let authenticator = Authenticator::new(config);

        let headers = RequestHeaders {
            authorization: Some(format!("Bearer {}", token)),
            ..Default::default()
        };

        match authenticator.authenticate(&headers).await {
            AuthResult::Authenticated(user) => {
                assert_eq!(user.id(), 1);
            }
            _ => panic!("Expected authenticated result"),
        }
    }

    #[tokio::test]
    async fn test_anonymous_access() {
        let config = AuthConfig::default().with_anonymous();
        let authenticator = Authenticator::new(config);

        let headers = RequestHeaders::default();

        match authenticator.authenticate(&headers).await {
            AuthResult::Anonymous => {}
            _ => panic!("Expected anonymous result"),
        }
    }

    #[tokio::test]
    async fn test_authentication_required() {
        let config = AuthConfig::default(); // No anonymous access
        let authenticator = Authenticator::new(config);

        let headers = RequestHeaders::default();

        match authenticator.authenticate(&headers).await {
            AuthResult::Failed(AuthError::Required) => {}
            _ => panic!("Expected authentication required error"),
        }
    }

    #[test]
    fn test_request_headers_from_pairs() {
        let pairs = vec![
            ("Authorization", "Bearer token123"),
            ("X-OpenProject-API-Key", "api-key-123"),
            ("Cookie", "_session=abc"),
        ];

        let headers = RequestHeaders::from_pairs(&pairs);

        assert_eq!(headers.authorization, Some("Bearer token123".to_string()));
        assert_eq!(headers.api_key, Some("api-key-123".to_string()));
        assert_eq!(headers.cookie, Some("_session=abc".to_string()));
    }
}
