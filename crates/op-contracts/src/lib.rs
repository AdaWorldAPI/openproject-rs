//! # op-contracts
//!
//! Contract validation for OpenProject RS.
//!
//! This crate implements the contract pattern from Ruby OpenProject,
//! providing validation and permission checking for domain operations.

pub mod base;
pub mod work_packages;

pub use base::*;
pub use work_packages::{
    CreateWorkPackageContract,
    UpdateWorkPackageContract,
    DeleteWorkPackageContract,
    WorkPackageBaseContract,
};
