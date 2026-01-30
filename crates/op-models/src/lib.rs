//! # op-models
//!
//! Domain models for OpenProject RS.
//!
//! This crate contains all entity structs that map to OpenProject's database tables.
//! Each model implements the core traits from `op-core` (Entity, Identifiable, etc.)

pub use op_core::traits::{Entity, Id, Identifiable, Timestamped, ProjectScoped, HalRepresentable};

// Core domain modules
pub mod user;
pub mod project;
pub mod work_package;
pub mod status;
pub mod type_def;
pub mod priority;
pub mod version;
pub mod member;
pub mod role;

// Re-exports for convenience
pub use user::model::{User, NewUser, UpdateUser};
pub use project::{Project, CreateProjectDto, UpdateProjectDto, ProjectStatusCode};
pub use work_package::model::WorkPackage;
pub use status::Status;
pub use type_def::Type;
pub use priority::Priority;
pub use version::{Version, VersionStatus, VersionSharing, CreateVersionDto};
pub use member::{Member, CreateMemberDto, UpdateMemberDto};
pub use role::{Role, permissions};
