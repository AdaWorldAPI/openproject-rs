//! User contracts
//!
//! Mirrors:
//! - app/contracts/users/base_contract.rb
//! - app/contracts/users/create_contract.rb
//! - app/contracts/users/update_contract.rb
//! - app/contracts/users/delete_contract.rb

mod base;
mod create;
mod update;
mod delete;

pub use base::{UserBaseContract, UserData};
pub use create::{CreateUserContract, CreateUserData};
pub use update::UpdateUserContract;
pub use delete::{DeleteUserContract, DeleteUserData};

/// Permissions required for user operations
pub mod permissions {
    pub const MANAGE_USER: &str = "manage_user";
    pub const CREATE_USER: &str = "create_user";
    pub const EDIT_USER: &str = "edit_user";
    pub const DELETE_USER: &str = "delete_user";
}
