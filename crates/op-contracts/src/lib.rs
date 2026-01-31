//! # op-contracts
//!
//! Contract validation for OpenProject RS.
//!
//! This crate implements the contract pattern from Ruby OpenProject,
//! providing validation and permission checking for domain operations.

pub mod base;
pub mod work_packages;
pub mod projects;
pub mod users;

pub use base::*;
pub use work_packages::{
    CreateWorkPackageContract,
    UpdateWorkPackageContract,
    DeleteWorkPackageContract,
    WorkPackageBaseContract,
};
pub use projects::{
    CreateProjectContract,
    UpdateProjectContract,
    DeleteProjectContract,
    ProjectBaseContract,
};
pub use users::{
    CreateUserContract,
    UpdateUserContract,
    DeleteUserContract,
    UserBaseContract,
};
