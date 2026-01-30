//! Create contract for work packages
//!
//! Mirrors: app/contracts/work_packages/create_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};
use super::base::{WorkPackageBaseContract, WorkPackageData};
use super::permissions;

/// Contract for creating a new work package
pub struct CreateWorkPackageContract<'a, U: UserContext> {
    base: WorkPackageBaseContract<'a, U>,
}

impl<'a, U: UserContext> CreateWorkPackageContract<'a, U> {
    pub fn new(user: &'a U, project_id: Id) -> Self {
        Self {
            base: WorkPackageBaseContract::new(user, project_id),
        }
    }

    /// Validate user has permission to create work packages
    fn validate_user_allowed_to_create(&self, errors: &mut ValidationErrors) {
        if !self.base.user_allowed_to_add() {
            errors.add("base", "You are not authorized to create work packages in this project");
        }
    }

    /// Validate author is set
    fn validate_author(&self, author_id: Id, errors: &mut ValidationErrors) {
        if author_id == 0 {
            errors.add("author", "can't be blank");
        }
    }

    /// Get the base contract
    pub fn base(&self) -> &WorkPackageBaseContract<'a, U> {
        &self.base
    }
}

impl<'a, U: UserContext, T: WorkPackageData> Contract<T> for CreateWorkPackageContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check permissions first
        self.validate_user_allowed_to_create(&mut errors);

        // Then run base validations
        if let Err(base_errors) = self.base.validate(entity) {
            errors.merge(base_errors);
        }

        // Create-specific validations
        self.validate_author(entity.author_id(), &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, attribute: &str) -> bool {
        // All attributes are writable on create
        matches!(
            attribute,
            "subject"
                | "description"
                | "type_id"
                | "status_id"
                | "priority_id"
                | "assigned_to_id"
                | "responsible_id"
                | "version_id"
                | "parent_id"
                | "start_date"
                | "due_date"
                | "estimated_hours"
                | "done_ratio"
                | "category_id"
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

    struct MockWorkPackage {
        subject: String,
        project_id: Id,
        type_id: Id,
        status_id: Id,
        author_id: Id,
        done_ratio: i32,
    }

    impl WorkPackageData for MockWorkPackage {
        fn id(&self) -> Option<Id> { None }
        fn subject(&self) -> &str { &self.subject }
        fn project_id(&self) -> Id { self.project_id }
        fn type_id(&self) -> Id { self.type_id }
        fn status_id(&self) -> Id { self.status_id }
        fn author_id(&self) -> Id { self.author_id }
        fn assigned_to_id(&self) -> Option<Id> { None }
        fn priority_id(&self) -> Option<Id> { None }
        fn version_id(&self) -> Option<Id> { None }
        fn parent_id(&self) -> Option<Id> { None }
        fn done_ratio(&self) -> i32 { self.done_ratio }
        fn estimated_hours(&self) -> Option<f64> { None }
        fn lock_version(&self) -> i32 { 0 }
    }

    #[test]
    fn test_admin_can_create() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let contract = CreateWorkPackageContract::new(&user, 1);

        let wp = MockWorkPackage {
            subject: "Test".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            author_id: 1,
            done_ratio: 0,
        };

        assert!(contract.validate(&wp).is_ok());
    }

    #[test]
    fn test_user_without_permission_cannot_create() {
        let user = MockUser { id: 1, admin: false, permissions: HashSet::new() };
        let contract = CreateWorkPackageContract::new(&user, 1);

        let wp = MockWorkPackage {
            subject: "Test".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            author_id: 1,
            done_ratio: 0,
        };

        let result = contract.validate(&wp);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_with_permission_can_create() {
        let mut permissions = HashSet::new();
        permissions.insert((permissions::ADD_WORK_PACKAGES.to_string(), 1));

        let user = MockUser { id: 1, admin: false, permissions };
        let contract = CreateWorkPackageContract::new(&user, 1);

        let wp = MockWorkPackage {
            subject: "Test".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            author_id: 1,
            done_ratio: 0,
        };

        assert!(contract.validate(&wp).is_ok());
    }
}
