//! # op-services
//!
//! Business logic services for OpenProject RS.
//!
//! This crate implements the Service Object pattern from OpenProject Ruby.
//! Services handle business operations like create, update, delete.

pub mod base;
pub mod work_packages;
pub mod users;
pub mod projects;
pub mod members;
pub mod notifications;
pub mod journals;
pub mod attachments;
pub mod oauth;
pub mod webhooks;
pub mod queries;
pub mod custom_fields;
pub mod mail;

pub use base::*;
