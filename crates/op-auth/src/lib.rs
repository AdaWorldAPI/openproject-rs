//! # op-auth
//!
//! Authentication and authorization for OpenProject RS.
//!
//! ## Features
//!
//! - JWT authentication
//! - API key authentication
//! - Session-based authentication
//! - Permission system with role-based access control

pub mod api_key;
pub mod jwt;
pub mod middleware;
pub mod permissions;
pub mod session;

pub use api_key::{ApiKey, ApiKeyService};
pub use jwt::{Claims, JwtError, JwtService};
pub use middleware::{AuthConfig, AuthError, AuthResult, AuthStrategy, Authenticator, RequestHeaders};
pub use permissions::CurrentUser;
pub use session::{CookieConfig, MemorySessionStore, Session, SessionError, SessionStore};
