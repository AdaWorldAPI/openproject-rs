//! Base contract for work packages
//!
//! Mirrors: app/contracts/work_packages/base_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};

/// Work package data for validation
pub trait WorkPackageData: Send + Sync {
    fn id(&self) -> Option<Id>;
    fn subject(&self) -> &str;
    fn project_id(&self) -> Id;
    fn type_id(&self) -> Id;
    fn status_id(&self) -> Id;
    fn author_id(&self) -> Id;
    fn assigned_to_id(&self) -> Option<Id>;
    fn priority_id(&self) -> Option<Id>;
    fn version_id(&self) -> Option<Id>;
    fn parent_id(&self) -> Option<Id>;
    fn done_ratio(&self) -> i32;
    fn estimated_hours(&self) -> Option<f64>;
    fn lock_version(&self) -> i32;
}

/// Base contract for work packages with common validations
pub struct WorkPackageBaseContract<'a, U: UserContext> {
    user: &'a U,
    project_id: Id,
}

impl<'a, U: UserContext> WorkPackageBaseContract<'a, U> {
    pub fn new(user: &'a U, project_id: Id) -> Self {
        Self { user, project_id }
    }

    /// Validate subject is present and within length
    pub fn validate_subject(&self, subject: &str, errors: &mut ValidationErrors) {
        if subject.trim().is_empty() {
            errors.add("subject", "can't be blank");
        } else if subject.len() > 255 {
            errors.add("subject", "is too long (maximum is 255 characters)");
        }
    }

    /// Validate project is present
    pub fn validate_project(&self, project_id: Id, errors: &mut ValidationErrors) {
        if project_id == 0 {
            errors.add("project", "can't be blank");
        }
    }

    /// Validate type is present
    pub fn validate_type(&self, type_id: Id, errors: &mut ValidationErrors) {
        if type_id == 0 {
            errors.add("type", "can't be blank");
        }
    }

    /// Validate status is present
    pub fn validate_status(&self, status_id: Id, errors: &mut ValidationErrors) {
        if status_id == 0 {
            errors.add("status", "can't be blank");
        }
    }

    /// Validate done ratio is between 0 and 100
    pub fn validate_done_ratio(&self, done_ratio: i32, errors: &mut ValidationErrors) {
        if !(0..=100).contains(&done_ratio) {
            errors.add("done_ratio", "must be between 0 and 100");
        }
    }

    /// Validate estimated hours is positive if present
    pub fn validate_estimated_hours(&self, hours: Option<f64>, errors: &mut ValidationErrors) {
        if let Some(h) = hours {
            if h < 0.0 {
                errors.add("estimated_hours", "must be greater than or equal to 0");
            }
        }
    }

    /// Check if user can edit work packages in the project
    pub fn user_allowed_to_edit(&self) -> bool {
        self.user.is_admin()
            || self.user.allowed_in_project(super::permissions::EDIT_WORK_PACKAGES, self.project_id)
    }

    /// Check if user can add work packages to the project
    pub fn user_allowed_to_add(&self) -> bool {
        self.user.is_admin()
            || self.user.allowed_in_project(super::permissions::ADD_WORK_PACKAGES, self.project_id)
    }

    /// Get the user context
    pub fn user(&self) -> &'a U {
        self.user
    }

    /// Get the project ID
    pub fn project_id(&self) -> Id {
        self.project_id
    }
}

impl<'a, U: UserContext, T: WorkPackageData> Contract<T> for WorkPackageBaseContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        self.validate_subject(entity.subject(), &mut errors);
        self.validate_project(entity.project_id(), &mut errors);
        self.validate_type(entity.type_id(), &mut errors);
        self.validate_status(entity.status_id(), &mut errors);
        self.validate_done_ratio(entity.done_ratio(), &mut errors);
        self.validate_estimated_hours(entity.estimated_hours(), &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
            self.permissions.contains(&(permission.to_string(), project_id))
        }
        fn allowed_globally(&self, _permission: &str) -> bool { false }
    }

    struct MockWorkPackage {
        subject: String,
        project_id: Id,
        type_id: Id,
        status_id: Id,
        done_ratio: i32,
        estimated_hours: Option<f64>,
    }

    impl WorkPackageData for MockWorkPackage {
        fn id(&self) -> Option<Id> { None }
        fn subject(&self) -> &str { &self.subject }
        fn project_id(&self) -> Id { self.project_id }
        fn type_id(&self) -> Id { self.type_id }
        fn status_id(&self) -> Id { self.status_id }
        fn author_id(&self) -> Id { 1 }
        fn assigned_to_id(&self) -> Option<Id> { None }
        fn priority_id(&self) -> Option<Id> { None }
        fn version_id(&self) -> Option<Id> { None }
        fn parent_id(&self) -> Option<Id> { None }
        fn done_ratio(&self) -> i32 { self.done_ratio }
        fn estimated_hours(&self) -> Option<f64> { self.estimated_hours }
        fn lock_version(&self) -> i32 { 0 }
    }

    #[test]
    fn test_valid_work_package() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let contract = WorkPackageBaseContract::new(&user, 1);

        let wp = MockWorkPackage {
            subject: "Test work package".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            done_ratio: 50,
            estimated_hours: Some(8.0),
        };

        assert!(contract.validate(&wp).is_ok());
    }

    #[test]
    fn test_blank_subject() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let contract = WorkPackageBaseContract::new(&user, 1);

        let wp = MockWorkPackage {
            subject: "".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            done_ratio: 0,
            estimated_hours: None,
        };

        let result = contract.validate(&wp);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.has_error("subject"));
    }

    #[test]
    fn test_invalid_done_ratio() {
        let user = MockUser { id: 1, admin: true, permissions: HashSet::new() };
        let contract = WorkPackageBaseContract::new(&user, 1);

        let wp = MockWorkPackage {
            subject: "Test".to_string(),
            project_id: 1,
            type_id: 1,
            status_id: 1,
            done_ratio: 150,
            estimated_hours: None,
        };

        let result = contract.validate(&wp);
        assert!(result.is_err());
    }
}
