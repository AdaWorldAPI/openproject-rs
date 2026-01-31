//! Delete contract for projects
//!
//! Mirrors: app/contracts/projects/delete_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};

/// Contract for deleting a project
pub struct DeleteProjectContract<'a, U: UserContext> {
    user: &'a U,
    project_id: Id,
}

impl<'a, U: UserContext> DeleteProjectContract<'a, U> {
    pub fn new(user: &'a U, project_id: Id) -> Self {
        Self { user, project_id }
    }

    /// Validate user has permission to delete the project
    /// Note: Only admins can delete projects in OpenProject
    fn validate_user_allowed_to_delete(&self, errors: &mut ValidationErrors) {
        if !self.user.is_admin() {
            errors.add("base", "Only administrators can delete projects");
        }
    }

    /// Get the project ID
    pub fn project_id(&self) -> Id {
        self.project_id
    }
}

/// Minimal data needed for delete validation
pub struct DeleteProjectData {
    pub id: Id,
    pub has_children: bool,
    pub work_package_count: i64,
}

impl<'a, U: UserContext> Contract<DeleteProjectData> for DeleteProjectContract<'a, U> {
    fn validate(&self, entity: &DeleteProjectData) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check permissions (only admin)
        self.validate_user_allowed_to_delete(&mut errors);

        // Warn about children (deletion is cascading)
        if entity.has_children {
            // This is a warning, not an error - deletion will proceed
            // but subprojects will also be deleted
        }

        // Warn about work packages (they will be deleted too)
        if entity.work_package_count > 0 {
            // This is informational - the API should confirm with user
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, _attribute: &str) -> bool {
        // Delete doesn't write any attributes
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUser {
        id: Id,
        admin: bool,
    }

    impl UserContext for MockUser {
        fn id(&self) -> Id { self.id }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, _permission: &str, _project_id: Id) -> bool { false }
        fn allowed_globally(&self, _permission: &str) -> bool { false }
    }

    #[test]
    fn test_admin_can_delete() {
        let user = MockUser { id: 1, admin: true };
        let contract = DeleteProjectContract::new(&user, 1);

        let data = DeleteProjectData {
            id: 1,
            has_children: false,
            work_package_count: 0,
        };

        assert!(contract.validate(&data).is_ok());
    }

    #[test]
    fn test_non_admin_cannot_delete() {
        let user = MockUser { id: 1, admin: false };
        let contract = DeleteProjectContract::new(&user, 1);

        let data = DeleteProjectData {
            id: 1,
            has_children: false,
            work_package_count: 0,
        };

        let result = contract.validate(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_can_delete_with_children() {
        let user = MockUser { id: 1, admin: true };
        let contract = DeleteProjectContract::new(&user, 1);

        let data = DeleteProjectData {
            id: 1,
            has_children: true,
            work_package_count: 100,
        };

        // Still allowed - children and work packages will be cascade deleted
        assert!(contract.validate(&data).is_ok());
    }
}
