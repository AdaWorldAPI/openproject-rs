//! # op-db
//!
//! Database layer for OpenProject RS.
//!
//! This crate provides PostgreSQL database access via SQLx.

pub mod pool;
pub mod repositories;
pub mod queries;
pub mod migrations;

pub use pool::*;
