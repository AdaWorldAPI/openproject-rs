//! # op-auth
//!
//! Authentication and authorization for OpenProject RS.
//!
//! This crate handles:
//! - Password hashing (Argon2)
//! - JWT token generation/validation
//! - OAuth 2.0 flows
//! - Session management
//! - Permission checking

pub mod password;
pub mod jwt;
pub mod oauth;
pub mod session;
pub mod middleware;
pub mod permissions;

pub use middleware::*;
