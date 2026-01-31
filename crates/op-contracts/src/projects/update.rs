//! Update contract for projects
//!
//! Mirrors: app/contracts/projects/update_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{ChangeTracker, Contract, UserContext, ValidationResult};
use super::base::{ProjectBaseContract, ProjectData};
use super::permissions;

/// Contract for updating an existing project
pub struct UpdateProjectContract<'a, U: UserContext> {
    base: ProjectBaseContract<'a, U>,
    user: &'a U,
    project_id: Id,
    changes: ChangeTracker,
}

impl<'a, U: UserContext> UpdateProjectContract<'a, U> {
    pub fn new(user: &'a U, project_id: Id) -> Self {
        Self {
            base: ProjectBaseContract::new(user),
            user,
            project_id,
            changes: ChangeTracker::new(),
        }
    }

    /// Mark an attribute as changed
    pub fn mark_changed(&mut self, attribute: impl Into<String>) {
        self.changes.mark_changed(attribute);
    }

    /// Check if an attribute was changed
    pub fn is_changed(&self, attribute: &str) -> bool {
        self.changes.is_changed(attribute)
    }

    /// Validate user has permission to edit the project
    fn validate_user_allowed_to_edit(&self, errors: &mut ValidationErrors) {
        let can_edit = self.user.is_admin()
            || self.user.allowed_in_project(permissions::EDIT_PROJECT, self.project_id);

        if !can_edit {
            errors.add("base", "You are not authorized to edit this project");
        }
    }

    /// Validate that identifier is not changed (it's immutable after creation)
    fn validate_identifier_not_changed(&self, errors: &mut ValidationErrors) {
        if self.changes.is_changed("identifier") {
            errors.add("identifier", "cannot be changed after project creation");
        }
    }

    /// Validate parent change is allowed
    fn validate_parent_change(&self, new_parent_id: Option<Id>, errors: &mut ValidationErrors) {
        if !self.changes.is_changed("parent_id") {
            return;
        }

        // Can't set self as parent
        if let Some(pid) = new_parent_id {
            if pid == self.project_id {
                errors.add("parent", "cannot be set to the project itself");
                return;
            }

            // Check permission on new parent
            if !self.user.is_admin() && !self.user.allowed_in_project(permissions::ADD_PROJECT, pid) {
                errors.add("parent", "you don't have permission to move project under this parent");
            }
        }
    }

    /// Get the project ID
    pub fn project_id(&self) -> Id {
        self.project_id
    }
}

impl<'a, U: UserContext, T: ProjectData> Contract<T> for UpdateProjectContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check edit permission
        self.validate_user_allowed_to_edit(&mut errors);

        // Check immutable fields
        self.validate_identifier_not_changed(&mut errors);

        // Check parent change
        self.validate_parent_change(entity.parent_id(), &mut errors);

        // Run base validations (but skip identifier if not changed)
        if let Err(base_errors) = self.base.validate(entity) {
            // Filter out identifier errors if identifier wasn't changed
            for (field, messages) in base_errors.errors {
                if field != "identifier" || self.changes.is_changed("identifier") {
                    for msg in messages {
                        errors.add(&field, msg);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, attribute: &str) -> bool {
        // Identifier is not writable on update
        if attribute == "identifier" {
            return false;
        }

        matches!(
            attribute,
            "name" | "description" | "public" | "parent_id" |
            "status_code" | "status_explanation" | "active"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct MockUser {
        id: Id,
        admin: bool,
        project_permissions: HashSet<(String, Id)>,
    }

    impl UserContext for MockUser {
        fn id(&self) -> Id { self.id }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool {
            self.admin || self.project_permissions.contains(&(permission.to_string(), project_id))
        }
        fn allowed_globally(&self, _permission: &str) -> bool { false }
    }

    struct MockProject {
        id: Option<Id>,
        identifier: String,
        name: String,
        parent_id: Option<Id>,
    }

    impl ProjectData for MockProject {
        fn id(&self) -> Option<Id> { self.id }
        fn identifier(&self) -> &str { &self.identifier }
        fn name(&self) -> &str { &self.name }
        fn description(&self) -> Option<&str> { None }
        fn public(&self) -> bool { false }
        fn parent_id(&self) -> Option<Id> { self.parent_id }
        fn active(&self) -> bool { true }
    }

    #[test]
    fn test_admin_can_update() {
        let user = MockUser { id: 1, admin: true, project_permissions: HashSet::new() };
        let contract = UpdateProjectContract::new(&user, 1);

        let project = MockProject {
            id: Some(1),
            identifier: "test-project".to_string(),
            name: "Updated Name".to_string(),
            parent_id: None,
        };

        assert!(contract.validate(&project).is_ok());
    }

    #[test]
    fn test_user_with_permission_can_update() {
        let mut permissions = HashSet::new();
        permissions.insert((permissions::EDIT_PROJECT.to_string(), 1));

        let user = MockUser { id: 1, admin: false, project_permissions: permissions };
        let contract = UpdateProjectContract::new(&user, 1);

        let project = MockProject {
            id: Some(1),
            identifier: "test-project".to_string(),
            name: "Updated Name".to_string(),
            parent_id: None,
        };

        assert!(contract.validate(&project).is_ok());
    }

    #[test]
    fn test_change_tracking() {
        let user = MockUser { id: 1, admin: true, project_permissions: HashSet::new() };
        let mut contract = UpdateProjectContract::new(&user, 1);

        assert!(!contract.is_changed("name"));
        contract.mark_changed("name");
        assert!(contract.is_changed("name"));
    }

    #[test]
    fn test_identifier_not_writable() {
        let user = MockUser { id: 1, admin: true, project_permissions: HashSet::new() };
        let contract = UpdateProjectContract::new(&user, 1);

        // Use fully qualified syntax for trait method
        assert!(!<UpdateProjectContract<'_, MockUser> as Contract<MockProject>>::is_writable(&contract, "identifier"));
        assert!(<UpdateProjectContract<'_, MockUser> as Contract<MockProject>>::is_writable(&contract, "name"));
    }

    #[test]
    fn test_cannot_set_self_as_parent() {
        let user = MockUser { id: 1, admin: true, project_permissions: HashSet::new() };
        let mut contract = UpdateProjectContract::new(&user, 1);
        contract.mark_changed("parent_id");

        let project = MockProject {
            id: Some(1),
            identifier: "test-project".to_string(),
            name: "Test".to_string(),
            parent_id: Some(1), // Self-reference
        };

        let result = contract.validate(&project);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("parent"));
    }
}
