//! Delete contract for work packages
//!
//! Mirrors: app/contracts/work_packages/delete_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};
use super::permissions;

/// Contract for deleting a work package
pub struct DeleteWorkPackageContract<'a, U: UserContext> {
    user: &'a U,
    project_id: Id,
    work_package_id: Id,
}

impl<'a, U: UserContext> DeleteWorkPackageContract<'a, U> {
    pub fn new(user: &'a U, project_id: Id, work_package_id: Id) -> Self {
        Self {
            user,
            project_id,
            work_package_id,
        }
    }

    /// Validate user has permission to delete work packages
    fn validate_user_allowed_to_delete(&self, errors: &mut ValidationErrors) {
        let can_delete = self.user.is_admin()
            || self.user.allowed_in_project(permissions::DELETE_WORK_PACKAGES, self.project_id);

        if !can_delete {
            errors.add("base", "You are not authorized to delete this work package");
        }
    }

    /// Get the work package ID
    pub fn work_package_id(&self) -> Id {
        self.work_package_id
    }

    /// Get the project ID
    pub fn project_id(&self) -> Id {
        self.project_id
    }
}

/// Minimal data needed for delete validation
pub struct DeleteWorkPackageData {
    pub id: Id,
    pub project_id: Id,
}

impl<'a, U: UserContext> Contract<DeleteWorkPackageData> for DeleteWorkPackageContract<'a, U> {
    fn validate(&self, _entity: &DeleteWorkPackageData) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check permissions
        self.validate_user_allowed_to_delete(&mut errors);

        // Additional validations could include:
        // - Check if work package has children (may need to delete children first)
        // - Check if work package is referenced by other entities

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
    use std::collections::HashSet;

    struct MockUser {
        id: Id,
        admin: bool,
        permissions: HashSet<(String, Id)>,
    }

    impl UserContext for MockUser {
        fn id(&self) -> Id { self.id }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool {
            self.admin || self.permissions.contains(&(permission.to_string(), project_id))
        }
        fn allowed_globally(&self, _permission: &str) -> bool { false }
    }

    #[test]
    fn test_admin_can_delete() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let contract = DeleteWorkPackageContract::new(&user, 1, 1);

        let data = DeleteWorkPackageData { id: 1, project_id: 1 };
        assert!(contract.validate(&data).is_ok());
    }

    #[test]
    fn test_user_without_permission_cannot_delete() {
        let user = MockUser { id: 1, admin: false, permissions: HashSet::new() };
        let contract = DeleteWorkPackageContract::new(&user, 1, 1);

        let data = DeleteWorkPackageData { id: 1, project_id: 1 };
        let result = contract.validate(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_with_permission_can_delete() {
        let mut permissions = HashSet::new();
        permissions.insert((permissions::DELETE_WORK_PACKAGES.to_string(), 1));

        let user = MockUser { id: 1, admin: false, permissions };
        let contract = DeleteWorkPackageContract::new(&user, 1, 1);

        let data = DeleteWorkPackageData { id: 1, project_id: 1 };
        assert!(contract.validate(&data).is_ok());
    }
}
