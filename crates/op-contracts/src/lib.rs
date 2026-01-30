//! # op-contracts
//!
//! Contract validation for OpenProject RS.
//!
//! This crate implements the Contract pattern from OpenProject Ruby.
//! Contracts validate entities before create/update operations and
//! check user permissions.

pub mod base;
pub mod work_packages;
pub mod users;
pub mod projects;
pub mod members;
pub mod queries;
pub mod time_entries;
pub mod attachments;
pub mod notifications;
pub mod versions;
pub mod oauth;
pub mod webhooks;

pub use base::*;
