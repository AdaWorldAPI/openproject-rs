//! API Key Authentication
//!
//! Mirrors: lib/open_project/authentication/strategies/api_key_strategy.rb

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// API key errors
#[derive(Debug, Error)]
pub enum ApiKeyError {
    #[error("API key not found")]
    NotFound,
    #[error("API key is revoked")]
    Revoked,
    #[error("API key is expired")]
    Expired,
    #[error("Invalid API key format")]
    InvalidFormat,
}

/// API key data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique identifier
    pub id: i64,
    /// User who owns this API key
    pub user_id: i64,
    /// Hashed key value (stored)
    pub hashed_value: String,
    /// Last characters for display (e.g., "****abc123")
    pub last_chars: String,
    /// Optional name/description
    pub name: Option<String>,
    /// Whether the key is active
    pub active: bool,
    /// Expiration date (optional)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Creation date
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used date
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ApiKey {
    /// Check if the API key is valid (not revoked or expired)
    pub fn is_valid(&self) -> bool {
        if !self.active {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            if expires_at < chrono::Utc::now() {
                return false;
            }
        }

        true
    }
}

/// API key service for validating keys
pub struct ApiKeyService {
    /// Hash function for comparing keys
    hash_algorithm: HashAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum HashAlgorithm {
    Sha256,
    Argon2,
}

impl Default for ApiKeyService {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeyService {
    /// Create a new API key service
    pub fn new() -> Self {
        Self {
            hash_algorithm: HashAlgorithm::Sha256,
        }
    }

    /// Use Argon2 for hashing (more secure, slower)
    pub fn with_argon2(mut self) -> Self {
        self.hash_algorithm = HashAlgorithm::Argon2;
        self
    }

    /// Hash a plaintext API key
    pub fn hash_key(&self, plaintext: &str) -> String {
        match self.hash_algorithm {
            HashAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(plaintext.as_bytes());
                hex::encode(hasher.finalize())
            }
            HashAlgorithm::Argon2 => {
                // For Argon2, use the argon2 crate
                use argon2::{
                    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
                    Argon2,
                };
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                argon2
                    .hash_password(plaintext.as_bytes(), &salt)
                    .expect("Failed to hash password")
                    .to_string()
            }
        }
    }

    /// Verify a plaintext API key against a stored hash
    pub fn verify_key(&self, plaintext: &str, stored_hash: &str) -> bool {
        match self.hash_algorithm {
            HashAlgorithm::Sha256 => {
                let computed_hash = self.hash_key(plaintext);
                constant_time_compare(&computed_hash, stored_hash)
            }
            HashAlgorithm::Argon2 => {
                use argon2::{
                    password_hash::{PasswordHash, PasswordVerifier},
                    Argon2,
                };
                match PasswordHash::new(stored_hash) {
                    Ok(parsed_hash) => Argon2::default()
                        .verify_password(plaintext.as_bytes(), &parsed_hash)
                        .is_ok(),
                    Err(_) => false,
                }
            }
        }
    }

    /// Generate a new random API key
    pub fn generate_key() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        const KEY_LENGTH: usize = 40;

        let mut rng = rand::rng();
        (0..KEY_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Get the last N characters of a key for display
    pub fn get_display_suffix(key: &str, n: usize) -> String {
        if key.len() <= n {
            key.to_string()
        } else {
            format!("****{}", &key[key.len() - n..])
        }
    }
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

/// Extract API key from various header formats
pub fn extract_api_key(headers: &[(String, String)]) -> Option<String> {
    // Check X-OpenProject-API-Key header first
    for (name, value) in headers {
        if name.to_lowercase() == "x-openproject-api-key" {
            return Some(value.clone());
        }
    }

    // Check Authorization header with apikey scheme
    for (name, value) in headers {
        if name.to_lowercase() == "authorization" {
            if let Some(key) = extract_api_key_from_auth(value) {
                return Some(key);
            }
        }
    }

    None
}

/// Extract API key from Authorization header
fn extract_api_key_from_auth(auth_header: &str) -> Option<String> {
    let lower = auth_header.to_lowercase();

    // Basic auth with apikey as username
    if lower.starts_with("basic ") {
        if let Ok(decoded) = base64::decode(&auth_header[6..]) {
            if let Ok(credentials) = String::from_utf8(decoded) {
                if let Some((username, _)) = credentials.split_once(':') {
                    if username.to_lowercase() == "apikey" {
                        return None; // Use password as key
                    }
                    // Return the username as the API key if no password
                    return Some(username.to_string());
                }
            }
        }
    }

    None
}

// Base64 decode helper (simplified)
mod base64 {
    pub fn decode(input: &str) -> Result<Vec<u8>, ()> {
        use ::base64::Engine;
        ::base64::engine::general_purpose::STANDARD
            .decode(input)
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = ApiKeyService::generate_key();
        assert_eq!(key.len(), 40);
        assert!(key.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_hash_and_verify_sha256() {
        let service = ApiKeyService::new();
        let plaintext = "my-secret-api-key";
        let hash = service.hash_key(plaintext);

        assert!(service.verify_key(plaintext, &hash));
        assert!(!service.verify_key("wrong-key", &hash));
    }

    #[test]
    fn test_display_suffix() {
        assert_eq!(
            ApiKeyService::get_display_suffix("abcdefghij", 4),
            "****ghij"
        );
        assert_eq!(ApiKeyService::get_display_suffix("abc", 4), "abc");
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hell"));
    }

    #[test]
    fn test_api_key_validity() {
        let mut key = ApiKey {
            id: 1,
            user_id: 1,
            hashed_value: "hash".to_string(),
            last_chars: "****abc".to_string(),
            name: None,
            active: true,
            expires_at: None,
            created_at: chrono::Utc::now(),
            last_used_at: None,
        };

        assert!(key.is_valid());

        key.active = false;
        assert!(!key.is_valid());

        key.active = true;
        key.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        assert!(!key.is_valid());
    }
}
