//! # op-api
//!
//! REST API v3 handlers for OpenProject RS.
//!
//! This crate implements the HAL+JSON API matching OpenProject's API v3.

pub mod routes;
pub mod handlers;
pub mod representers;
pub mod schemas;
pub mod extractors;
pub mod middleware;
pub mod error;

pub use routes::router;
