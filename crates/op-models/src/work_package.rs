//! Work Package model and related types
//!
//! Mirrors: app/models/work_package.rb and related files
//!
//! Work packages are the central entity in OpenProject - they represent
//! tasks, issues, features, bugs, etc.

pub mod model;
pub mod hierarchy;
pub mod scheduling;
pub mod relations;

pub use model::*;
