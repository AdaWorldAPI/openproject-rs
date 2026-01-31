//! Session Authentication
//!
//! Mirrors: lib/open_project/authentication/strategies/session_strategy.rb

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Session errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Session invalid")]
    Invalid,
    #[error("User not authenticated")]
    NotAuthenticated,
}

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User ID (None for anonymous sessions)
    pub user_id: Option<i64>,
    /// Session data
    pub data: HashMap<String, String>,
    /// Creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed time
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    /// Expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: Option<i64>, lifetime_seconds: i64) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: generate_session_id(),
            user_id,
            data: HashMap::new(),
            created_at: now,
            accessed_at: now,
            expires_at: now + chrono::Duration::seconds(lifetime_seconds),
            ip_address: None,
            user_agent: None,
        }
    }

    /// Create an authenticated session
    pub fn authenticated(user_id: i64, lifetime_seconds: i64) -> Self {
        Self::new(Some(user_id), lifetime_seconds)
    }

    /// Create an anonymous session
    pub fn anonymous(lifetime_seconds: i64) -> Self {
        Self::new(None, lifetime_seconds)
    }

    /// Check if the session is valid
    pub fn is_valid(&self) -> bool {
        chrono::Utc::now() < self.expires_at
    }

    /// Check if this is an authenticated session
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Touch the session (update accessed_at)
    pub fn touch(&mut self) {
        self.accessed_at = chrono::Utc::now();
    }

    /// Extend the session
    pub fn extend(&mut self, additional_seconds: i64) {
        self.expires_at = chrono::Utc::now() + chrono::Duration::seconds(additional_seconds);
        self.touch();
    }

    /// Set session data
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get session data
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Remove session data
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }
}

/// Generate a secure random session ID
fn generate_session_id() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const SESSION_ID_LENGTH: usize = 64;

    let mut rng = rand::rng();
    (0..SESSION_ID_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Session store trait for different backends
pub trait SessionStore: Send + Sync {
    /// Get a session by ID
    fn get(&self, session_id: &str) -> Option<Session>;

    /// Store a session
    fn set(&self, session: Session) -> Result<(), SessionError>;

    /// Delete a session
    fn delete(&self, session_id: &str) -> Result<(), SessionError>;

    /// Delete all sessions for a user
    fn delete_user_sessions(&self, user_id: i64) -> Result<usize, SessionError>;

    /// Clean up expired sessions
    fn cleanup_expired(&self) -> Result<usize, SessionError>;
}

/// In-memory session store (for development/testing)
pub struct MemorySessionStore {
    sessions: std::sync::RwLock<HashMap<String, Session>>,
}

impl Default for MemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl SessionStore for MemorySessionStore {
    fn get(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().ok()?;
        sessions.get(session_id).cloned().filter(|s| s.is_valid())
    }

    fn set(&self, session: Session) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::Invalid)?;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    fn delete(&self, session_id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::Invalid)?;
        sessions.remove(session_id);
        Ok(())
    }

    fn delete_user_sessions(&self, user_id: i64) -> Result<usize, SessionError> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::Invalid)?;
        let to_remove: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.user_id == Some(user_id))
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_remove.len();
        for key in to_remove {
            sessions.remove(&key);
        }

        Ok(count)
    }

    fn cleanup_expired(&self) -> Result<usize, SessionError> {
        let mut sessions = self.sessions.write().map_err(|_| SessionError::Invalid)?;
        let now = chrono::Utc::now();
        let to_remove: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.expires_at < now)
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_remove.len();
        for key in to_remove {
            sessions.remove(&key);
        }

        Ok(count)
    }
}

/// Cookie configuration for sessions
#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub name: String,
    pub path: String,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub max_age: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            name: "_openproject_session".to_string(),
            path: "/".to_string(),
            domain: None,
            secure: true,
            http_only: true,
            same_site: SameSite::Lax,
            max_age: None,
        }
    }
}

impl CookieConfig {
    /// Create a development configuration (non-secure)
    pub fn development() -> Self {
        Self {
            secure: false,
            ..Default::default()
        }
    }

    /// Build cookie header value
    pub fn build_cookie(&self, session_id: &str) -> String {
        let mut parts = vec![format!("{}={}", self.name, session_id)];

        parts.push(format!("Path={}", self.path));

        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }

        if self.secure {
            parts.push("Secure".to_string());
        }

        if self.http_only {
            parts.push("HttpOnly".to_string());
        }

        match self.same_site {
            SameSite::Strict => parts.push("SameSite=Strict".to_string()),
            SameSite::Lax => parts.push("SameSite=Lax".to_string()),
            SameSite::None => parts.push("SameSite=None".to_string()),
        }

        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }

        parts.join("; ")
    }

    /// Build cookie header to clear the session
    pub fn build_clear_cookie(&self) -> String {
        format!(
            "{}=; Path={}; Max-Age=0; HttpOnly",
            self.name, self.path
        )
    }
}

/// Extract session ID from cookie header
pub fn extract_session_id(cookie_header: &str, cookie_name: &str) -> Option<String> {
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some((name, value)) = part.split_once('=') {
            if name.trim() == cookie_name {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::authenticated(1, 3600);
        assert!(session.is_valid());
        assert!(session.is_authenticated());
        assert_eq!(session.user_id, Some(1));
    }

    #[test]
    fn test_anonymous_session() {
        let session = Session::anonymous(3600);
        assert!(session.is_valid());
        assert!(!session.is_authenticated());
        assert_eq!(session.user_id, None);
    }

    #[test]
    fn test_session_data() {
        let mut session = Session::authenticated(1, 3600);
        session.set("key", "value");
        assert_eq!(session.get("key"), Some("value"));

        session.remove("key");
        assert_eq!(session.get("key"), None);
    }

    #[test]
    fn test_memory_session_store() {
        let store = MemorySessionStore::new();
        let session = Session::authenticated(1, 3600);
        let session_id = session.id.clone();

        store.set(session).unwrap();

        let retrieved = store.get(&session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, Some(1));

        store.delete(&session_id).unwrap();
        assert!(store.get(&session_id).is_none());
    }

    #[test]
    fn test_cookie_config() {
        let config = CookieConfig::default();
        let cookie = config.build_cookie("abc123");

        assert!(cookie.contains("_openproject_session=abc123"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn test_extract_session_id() {
        let cookie = "_openproject_session=abc123; other=value";
        assert_eq!(
            extract_session_id(cookie, "_openproject_session"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_session_id(cookie, "missing"), None);
    }
}
