//! Role model
//!
//! Mirrors: app/models/role.rb
//! Table: roles

use chrono::{DateTime, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, HalRepresentable};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use validator::Validate;

/// Role entity
///
/// Roles define sets of permissions that can be assigned to members.
///
/// # Ruby equivalent
/// ```ruby
/// class Role < ApplicationRecord
///   has_many :member_roles
///   has_many :members, through: :member_roles
///   has_many :role_permissions
///   has_and_belongs_to_many :workflows
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: Option<Id>,

    /// Role name (unique)
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Sort position
    #[serde(default)]
    pub position: i32,

    /// Whether this role can be assigned to members
    #[serde(default = "default_true")]
    pub assignable: bool,

    /// Whether role is builtin (system-managed)
    #[serde(default)]
    pub builtin: i32,

    /// Permissions granted by this role
    #[serde(default)]
    pub permissions: HashSet<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

impl Default for Role {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            position: 0,
            assignable: true,
            builtin: 0,
            permissions: HashSet::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Role {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Role {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl Entity for Role {
    const TABLE_NAME: &'static str = "roles";
    const TYPE_NAME: &'static str = "Role";
}

impl HalRepresentable for Role {
    fn hal_type(&self) -> &'static str {
        "Role"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/roles/{}", self.id.unwrap_or(0))
    }
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builtin role types
    pub const BUILTIN_NON_MEMBER: i32 = 1;
    pub const BUILTIN_ANONYMOUS: i32 = 2;
    pub const BUILTIN_WORK_PACKAGE_EDITOR: i32 = 3;
    pub const BUILTIN_WORK_PACKAGE_COMMENTER: i32 = 4;
    pub const BUILTIN_WORK_PACKAGE_VIEWER: i32 = 5;

    /// Check if role is builtin
    pub fn is_builtin(&self) -> bool {
        self.builtin > 0
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: impl Into<String>) {
        self.permissions.insert(permission.into());
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.remove(permission);
    }

    /// Standard role names
    pub const PROJECT_ADMIN: &'static str = "Project admin";
    pub const MEMBER: &'static str = "Member";
    pub const READER: &'static str = "Reader";
}

/// Common permissions in OpenProject
pub mod permissions {
    // Work package permissions
    pub const VIEW_WORK_PACKAGES: &str = "view_work_packages";
    pub const ADD_WORK_PACKAGES: &str = "add_work_packages";
    pub const EDIT_WORK_PACKAGES: &str = "edit_work_packages";
    pub const DELETE_WORK_PACKAGES: &str = "delete_work_packages";
    pub const MOVE_WORK_PACKAGES: &str = "move_work_packages";
    pub const COPY_WORK_PACKAGES: &str = "copy_work_packages";
    pub const MANAGE_WORK_PACKAGE_RELATIONS: &str = "manage_work_package_relations";
    pub const ASSIGN_VERSIONS: &str = "assign_versions";
    pub const COMMENT_ON_WORK_PACKAGES: &str = "comment_on_work_packages";
    pub const LOG_TIME: &str = "log_time";
    pub const VIEW_TIME_ENTRIES: &str = "view_time_entries";

    // Project permissions
    pub const VIEW_PROJECT: &str = "view_project";
    pub const EDIT_PROJECT: &str = "edit_project";
    pub const SELECT_PROJECT_MODULES: &str = "select_project_modules";
    pub const MANAGE_MEMBERS: &str = "manage_members";
    pub const MANAGE_VERSIONS: &str = "manage_versions";
    pub const MANAGE_CATEGORIES: &str = "manage_categories";
    pub const MANAGE_PROJECT_ACTIVITIES: &str = "manage_project_activities";

    // Wiki permissions
    pub const VIEW_WIKI_PAGES: &str = "view_wiki_pages";
    pub const EDIT_WIKI_PAGES: &str = "edit_wiki_pages";
    pub const DELETE_WIKI_PAGES: &str = "delete_wiki_pages";
    pub const PROTECT_WIKI_PAGES: &str = "protect_wiki_pages";

    // Forum permissions
    pub const VIEW_MESSAGES: &str = "view_messages";
    pub const ADD_MESSAGES: &str = "add_messages";
    pub const EDIT_MESSAGES: &str = "edit_messages";
    pub const DELETE_MESSAGES: &str = "delete_messages";

    // File permissions
    pub const VIEW_FILES: &str = "view_files";
    pub const MANAGE_FILES: &str = "manage_files";

    // Meeting permissions
    pub const VIEW_MEETINGS: &str = "view_meetings";
    pub const CREATE_MEETINGS: &str = "create_meetings";
    pub const EDIT_MEETINGS: &str = "edit_meetings";
    pub const DELETE_MEETINGS: &str = "delete_meetings";

    // Budget permissions
    pub const VIEW_BUDGETS: &str = "view_budgets";
    pub const EDIT_BUDGETS: &str = "edit_budgets";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_new() {
        let r = Role::new("Developer");
        assert_eq!(r.name, "Developer");
        assert!(r.assignable);
        assert!(!r.is_builtin());
    }

    #[test]
    fn test_role_permissions() {
        let mut r = Role::new("Tester");
        r.add_permission(permissions::VIEW_WORK_PACKAGES);
        r.add_permission(permissions::ADD_WORK_PACKAGES);

        assert!(r.has_permission(permissions::VIEW_WORK_PACKAGES));
        assert!(r.has_permission(permissions::ADD_WORK_PACKAGES));
        assert!(!r.has_permission(permissions::DELETE_WORK_PACKAGES));

        r.remove_permission(permissions::ADD_WORK_PACKAGES);
        assert!(!r.has_permission(permissions::ADD_WORK_PACKAGES));
    }

    #[test]
    fn test_builtin_role() {
        let mut r = Role::new("Non member");
        r.builtin = Role::BUILTIN_NON_MEMBER;
        assert!(r.is_builtin());
    }
}
