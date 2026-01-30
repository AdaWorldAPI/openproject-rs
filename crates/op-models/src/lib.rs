//! # op-models
//!
//! Domain models for OpenProject RS.

pub use op_core::traits::{Entity, Id, Identifiable, Timestamped};

pub mod user;
pub mod work_package;

pub use user::model::User;
