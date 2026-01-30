//! # op-api
//!
//! REST API v3 handlers for OpenProject RS.
//!
//! This crate implements the HAL+JSON API matching OpenProject's API v3.

pub mod error;
pub mod extractors;
pub mod handlers;
pub mod routes;

pub use routes::router;
