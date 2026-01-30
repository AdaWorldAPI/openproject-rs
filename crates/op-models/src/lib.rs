//! # op-models
//!
//! Domain models for OpenProject RS.
//!
//! This crate contains all the data models that represent the core domain entities.
//! Each model mirrors its Ruby counterpart from app/models/.

// Re-export core types
pub use op_core::traits::{Entity, Id, Identifiable, Timestamped};

// Domain modules
pub mod user;
pub mod project;
pub mod work_package;
pub mod status;
pub mod type_def;
pub mod priority;
pub mod version;
pub mod category;
pub mod role;
pub mod member;
pub mod attachment;
pub mod journal;
pub mod notification;
pub mod query;
pub mod custom_field;
pub mod custom_value;
pub mod comment;
pub mod activity;
pub mod time_entry;
pub mod cost;
pub mod meeting;
pub mod news;
pub mod wiki;
pub mod forum;
pub mod document;
pub mod relation;
pub mod watcher;
pub mod setting;
pub mod token;
pub mod oauth;
pub mod ldap;
pub mod webhook;
pub mod storage;
pub mod file_link;

// Re-exports for convenience
pub use user::*;
pub use project::*;
pub use work_package::*;
pub use status::*;
pub use type_def::*;
pub use priority::*;
