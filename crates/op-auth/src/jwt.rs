//! JWT Authentication
//!
//! Mirrors: lib/open_project/authentication/strategies/jwt_strategy.rb

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    /// Issued at (Unix timestamp)
    pub iat: usize,
    /// JWT ID (for token revocation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// User email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// User login
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

/// JWT errors
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token is expired")]
    Expired,
    #[error("Invalid token: {0}")]
    Invalid(String),
    #[error("Missing token")]
    Missing,
    #[error("Token encoding failed: {0}")]
    EncodingFailed(String),
}

/// JWT service for creating and validating tokens
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: Option<String>,
    audience: Option<String>,
}

impl JwtService {
    /// Create a new JWT service with the given secret
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            issuer: None,
            audience: None,
        }
    }

    /// Create from an RSA private key (PEM format)
    pub fn from_rsa_pem(private_key: &[u8], public_key: &[u8]) -> Result<Self, JwtError> {
        Ok(Self {
            encoding_key: EncodingKey::from_rsa_pem(private_key)
                .map_err(|e| JwtError::Invalid(e.to_string()))?,
            decoding_key: DecodingKey::from_rsa_pem(public_key)
                .map_err(|e| JwtError::Invalid(e.to_string()))?,
            issuer: None,
            audience: None,
        })
    }

    /// Set the issuer claim for validation
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Set the audience claim for validation
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// Create a new JWT token
    pub fn create_token(
        &self,
        user_id: i64,
        email: Option<String>,
        login: Option<String>,
        expires_in_seconds: i64,
    ) -> Result<String, JwtError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + expires_in_seconds as usize,
            iat: now,
            jti: Some(uuid::Uuid::new_v4().to_string()),
            email,
            login,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::EncodingFailed(e.to_string()))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::default();

        if let Some(ref issuer) = self.issuer {
            validation.set_issuer(&[issuer.clone()]);
        }

        if let Some(ref audience) = self.audience {
            validation.set_audience(&[audience.clone()]);
        }

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
                _ => JwtError::Invalid(e.to_string()),
            })?;

        Ok(token_data.claims)
    }

    /// Extract user ID from a validated token
    pub fn get_user_id(&self, token: &str) -> Result<i64, JwtError> {
        let claims = self.validate_token(token)?;
        claims
            .sub
            .parse()
            .map_err(|_| JwtError::Invalid("Invalid user ID in token".to_string()))
    }
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(authorization: &str) -> Option<&str> {
    if authorization.to_lowercase().starts_with("bearer ") {
        Some(authorization[7..].trim())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let service = JwtService::new(b"test-secret-key-at-least-32-bytes");

        let token = service
            .create_token(1, Some("test@example.com".into()), Some("testuser".into()), 3600)
            .unwrap();

        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "1");
        assert_eq!(claims.email, Some("test@example.com".into()));
        assert_eq!(claims.login, Some("testuser".into()));
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            extract_bearer_token("bearer abc123"),
            Some("abc123")
        );
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn test_get_user_id() {
        let service = JwtService::new(b"test-secret-key-at-least-32-bytes");

        let token = service
            .create_token(42, None, None, 3600)
            .unwrap();

        let user_id = service.get_user_id(&token).unwrap();
        assert_eq!(user_id, 42);
    }
}
