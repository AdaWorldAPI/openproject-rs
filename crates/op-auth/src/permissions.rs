//! Permission system for OpenProject RS
//!
//! Implements role-based access control matching OpenProject's permission model.

use op_core::traits::Id;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Permission Definition
// ============================================================================

/// Permission scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionScope {
    Global,
    Project,
    WorkPackage,
}

/// Built-in permission definitions
pub struct Permission {
    pub name: &'static str,
    pub scope: PermissionScope,
    pub description: &'static str,
}

/// All built-in permissions matching OpenProject
pub mod builtin {
    use super::*;

    // Global permissions
    pub const ADD_PROJECT: Permission = Permission {
        name: "add_project",
        scope: PermissionScope::Global,
        description: "Create new projects",
    };

    pub const COPY_PROJECTS: Permission = Permission {
        name: "copy_projects",
        scope: PermissionScope::Global,
        description: "Copy projects",
    };

    // Project permissions
    pub const VIEW_PROJECT: Permission = Permission {
        name: "view_project",
        scope: PermissionScope::Project,
        description: "View project",
    };

    pub const EDIT_PROJECT: Permission = Permission {
        name: "edit_project",
        scope: PermissionScope::Project,
        description: "Edit project settings",
    };

    pub const DELETE_PROJECT: Permission = Permission {
        name: "delete_project",
        scope: PermissionScope::Project,
        description: "Delete project",
    };

    pub const MANAGE_MEMBERS: Permission = Permission {
        name: "manage_members",
        scope: PermissionScope::Project,
        description: "Manage project members",
    };

    // Work package permissions
    pub const VIEW_WORK_PACKAGES: Permission = Permission {
        name: "view_work_packages",
        scope: PermissionScope::Project,
        description: "View work packages",
    };

    pub const ADD_WORK_PACKAGES: Permission = Permission {
        name: "add_work_packages",
        scope: PermissionScope::Project,
        description: "Create work packages",
    };

    pub const EDIT_WORK_PACKAGES: Permission = Permission {
        name: "edit_work_packages",
        scope: PermissionScope::Project,
        description: "Edit work packages",
    };

    pub const DELETE_WORK_PACKAGES: Permission = Permission {
        name: "delete_work_packages",
        scope: PermissionScope::Project,
        description: "Delete work packages",
    };

    pub const MANAGE_WORK_PACKAGE_RELATIONS: Permission = Permission {
        name: "manage_work_package_relations",
        scope: PermissionScope::Project,
        description: "Manage work package relations",
    };

    pub const MOVE_WORK_PACKAGES: Permission = Permission {
        name: "move_work_packages",
        scope: PermissionScope::Project,
        description: "Move work packages between projects",
    };

    pub const ASSIGN_VERSIONS: Permission = Permission {
        name: "assign_versions",
        scope: PermissionScope::Project,
        description: "Assign versions to work packages",
    };
}

// ============================================================================
// User Context
// ============================================================================

/// Current user with permissions
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Id,
    pub login: String,
    pub email: String,
    pub is_admin: bool,
    pub is_anonymous: bool,
    global_permissions: HashSet<String>,
    project_permissions: HashMap<Id, HashSet<String>>,
    work_package_permissions: HashMap<Id, HashSet<String>>,
}

impl CurrentUser {
    /// Create a new current user
    pub fn new(id: Id, login: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            login: login.into(),
            email: email.into(),
            is_admin: false,
            is_anonymous: false,
            global_permissions: HashSet::new(),
            project_permissions: HashMap::new(),
            work_package_permissions: HashMap::new(),
        }
    }

    /// Create an anonymous user
    pub fn anonymous() -> Self {
        Self {
            id: 0,
            login: "anonymous".to_string(),
            email: String::new(),
            is_admin: false,
            is_anonymous: true,
            global_permissions: HashSet::new(),
            project_permissions: HashMap::new(),
            work_package_permissions: HashMap::new(),
        }
    }

    /// Create an admin user
    pub fn admin(id: Id, login: impl Into<String>, email: impl Into<String>) -> Self {
        let mut user = Self::new(id, login, email);
        user.is_admin = true;
        user
    }

    /// Add a global permission
    pub fn add_global_permission(&mut self, permission: impl Into<String>) {
        self.global_permissions.insert(permission.into());
    }

    /// Add a project permission
    pub fn add_project_permission(&mut self, project_id: Id, permission: impl Into<String>) {
        self.project_permissions
            .entry(project_id)
            .or_default()
            .insert(permission.into());
    }

    /// Check if user has permission in project
    pub fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool {
        if self.is_admin {
            return true;
        }
        if let Some(perms) = self.project_permissions.get(&project_id) {
            return perms.contains(permission);
        }
        false
    }

    /// Check if user has global permission
    pub fn allowed_globally(&self, permission: &str) -> bool {
        if self.is_admin {
            return true;
        }
        self.global_permissions.contains(permission)
    }

    /// Get user ID
    pub fn id(&self) -> Id {
        self.id
    }

    /// Check if admin
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    /// Check if anonymous
    pub fn is_anonymous(&self) -> bool {
        self.is_anonymous
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let user = CurrentUser::new(1, "admin", "admin@example.com");
        assert_eq!(user.id, 1);
        assert_eq!(user.login, "admin");
        assert!(!user.is_admin);
    }

    #[test]
    fn test_admin_user() {
        let user = CurrentUser::admin(1, "admin", "admin@example.com");
        assert!(user.is_admin);
        assert!(user.allowed_in_project("anything", 1));
        assert!(user.allowed_globally("anything"));
    }

    #[test]
    fn test_project_permissions() {
        let mut user = CurrentUser::new(1, "user", "user@example.com");
        user.add_project_permission(1, "view_work_packages");

        assert!(user.allowed_in_project("view_work_packages", 1));
        assert!(!user.allowed_in_project("edit_work_packages", 1));
        assert!(!user.allowed_in_project("view_work_packages", 2));
    }

    #[test]
    fn test_global_permissions() {
        let mut user = CurrentUser::new(1, "user", "user@example.com");
        user.add_global_permission("add_project");

        assert!(user.allowed_globally("add_project"));
        assert!(!user.allowed_globally("copy_projects"));
    }

    #[test]
    fn test_anonymous_user() {
        let user = CurrentUser::anonymous();
        assert!(user.is_anonymous);
        assert_eq!(user.id, 0);
    }
}
