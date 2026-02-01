//! Work Package contracts
//!
//! Mirrors:
//! - app/contracts/work_packages/base_contract.rb
//! - app/contracts/work_packages/create_contract.rb
//! - app/contracts/work_packages/update_contract.rb
//! - app/contracts/work_packages/delete_contract.rb

mod base;
mod create;
mod update;
mod delete;

pub use base::{WorkPackageBaseContract, WorkPackageData};
pub use create::CreateWorkPackageContract;
pub use update::UpdateWorkPackageContract;
pub use delete::{DeleteWorkPackageContract, DeleteWorkPackageData};

/// Permissions required for work package operations
pub mod permissions {
    pub const VIEW_WORK_PACKAGES: &str = "view_work_packages";
    pub const ADD_WORK_PACKAGES: &str = "add_work_packages";
    pub const EDIT_WORK_PACKAGES: &str = "edit_work_packages";
    pub const DELETE_WORK_PACKAGES: &str = "delete_work_packages";
    pub const MANAGE_SUBTASKS: &str = "manage_subtasks";
    pub const ADD_WORK_PACKAGE_NOTES: &str = "add_work_package_notes";
    pub const EDIT_OWN_WORK_PACKAGE_NOTES: &str = "edit_own_work_package_notes";
    pub const ASSIGN_VERSIONS: &str = "assign_versions";
    pub const LOG_TIME: &str = "log_time";
}
