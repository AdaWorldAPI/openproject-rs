//! Update contract for work packages
//!
//! Mirrors: app/contracts/work_packages/update_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult, ChangeTracker};
use super::base::{WorkPackageBaseContract, WorkPackageData};
use super::permissions;

/// Contract for updating an existing work package
pub struct UpdateWorkPackageContract<'a, U: UserContext> {
    base: WorkPackageBaseContract<'a, U>,
    work_package_id: Id,
    changes: ChangeTracker,
}

impl<'a, U: UserContext> UpdateWorkPackageContract<'a, U> {
    pub fn new(user: &'a U, project_id: Id, work_package_id: Id) -> Self {
        Self {
            base: WorkPackageBaseContract::new(user, project_id),
            work_package_id,
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

    /// Validate user has permission to edit work packages
    fn validate_user_allowed_to_edit(&self, errors: &mut ValidationErrors) {
        if !self.base.user_allowed_to_edit() {
            errors.add("base", "You are not authorized to edit this work package");
        }
    }

    /// Validate status transitions (simplified - full implementation would check workflows)
    fn validate_status_transition(&self, _old_status_id: Id, _new_status_id: Id, errors: &mut ValidationErrors) {
        // In the full implementation, this would:
        // 1. Check if the transition is allowed by the workflow
        // 2. Check if the user's role allows the transition
        // For now, we allow all transitions
        let _ = errors;
    }

    /// Get the work package ID
    pub fn work_package_id(&self) -> Id {
        self.work_package_id
    }

    /// Get the base contract
    pub fn base(&self) -> &WorkPackageBaseContract<'a, U> {
        &self.base
    }
}

impl<'a, U: UserContext, T: WorkPackageData> Contract<T> for UpdateWorkPackageContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check permissions first
        self.validate_user_allowed_to_edit(&mut errors);

        // Then run base validations
        if let Err(base_errors) = self.base.validate(entity) {
            errors.merge(base_errors);
        }

        // Update-specific validations would go here
        // e.g., validate_status_transition if status changed

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, attribute: &str) -> bool {
        // Most attributes are writable on update
        // Some may be restricted based on status or permissions
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
                | "lock_version"
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
        id: Id,
        subject: String,
        project_id: Id,
        type_id: Id,
        status_id: Id,
        author_id: Id,
        done_ratio: i32,
    }

    impl WorkPackageData for MockWorkPackage {
        fn id(&self) -> Option<Id> { Some(self.id) }
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
    fn test_admin_can_update() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let contract = UpdateWorkPackageContract::new(&user, 1, 1);

        let wp = MockWorkPackage {
            id: 1,
            subject: "Updated".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            author_id: 1,
            done_ratio: 50,
        };

        assert!(contract.validate(&wp).is_ok());
    }

    #[test]
    fn test_change_tracking() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let mut contract = UpdateWorkPackageContract::new(&user, 1, 1);

        assert!(!contract.is_changed("subject"));

        contract.mark_changed("subject");
        assert!(contract.is_changed("subject"));
        assert!(!contract.is_changed("status_id"));
    }

    #[test]
    fn test_user_with_permission_can_update() {
        let mut permissions = HashSet::new();
        permissions.insert((permissions::EDIT_WORK_PACKAGES.to_string(), 1));

        let user = MockUser { id: 1, admin: false, permissions };
        let contract = UpdateWorkPackageContract::new(&user, 1, 1);

        let wp = MockWorkPackage {
            id: 1,
            subject: "Updated".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            author_id: 1,
            done_ratio: 50,
        };

        assert!(contract.validate(&wp).is_ok());
    }
}
