//! # op-core
//!
//! Core types, traits, and utilities for OpenProject RS.
//!
//! This crate provides the foundational building blocks used across all other crates:
//! - Common error types
//! - Result type aliases
//! - Core traits (Entity, Identifiable, Timestamped)
//! - Pagination types
//! - Service result types (ServiceResult)
//! - Configuration types

pub mod error;
pub mod result;
pub mod traits;
pub mod types;
pub mod pagination;
pub mod config;

pub use error::*;
pub use result::*;
pub use traits::*;
pub use types::*;
pub use pagination::*;
