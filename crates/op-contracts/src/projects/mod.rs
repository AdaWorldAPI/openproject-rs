//! Project contracts
//!
//! Mirrors:
//! - app/contracts/projects/base_contract.rb
//! - app/contracts/projects/create_contract.rb
//! - app/contracts/projects/update_contract.rb
//! - app/contracts/projects/delete_contract.rb

mod base;
mod create;
mod update;
mod delete;

pub use base::ProjectBaseContract;
pub use create::CreateProjectContract;
pub use update::UpdateProjectContract;
pub use delete::DeleteProjectContract;

/// Permissions required for project operations
pub mod permissions {
    // Project-level permissions
    pub const VIEW_PROJECT: &str = "view_project";
    pub const EDIT_PROJECT: &str = "edit_project";
    pub const SELECT_PROJECT_MODULES: &str = "select_project_modules";
    pub const MANAGE_MEMBERS: &str = "manage_members";
    pub const MANAGE_VERSIONS: &str = "manage_versions";
    pub const MANAGE_CATEGORIES: &str = "manage_categories";
    pub const MANAGE_PROJECT_ACTIVITIES: &str = "manage_project_activities";
    pub const COPY_PROJECTS: &str = "copy_projects";
    pub const ARCHIVE_PROJECT: &str = "archive_project";

    // Global permissions
    pub const ADD_PROJECT: &str = "add_project";
}
