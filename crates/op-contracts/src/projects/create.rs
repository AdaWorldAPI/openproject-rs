//! Create contract for projects
//!
//! Mirrors: app/contracts/projects/create_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};
use super::base::{ProjectBaseContract, ProjectData};
use super::permissions;

/// Contract for creating a new project
pub struct CreateProjectContract<'a, U: UserContext> {
    base: ProjectBaseContract<'a, U>,
    user: &'a U,
}

impl<'a, U: UserContext> CreateProjectContract<'a, U> {
    pub fn new(user: &'a U) -> Self {
        Self {
            base: ProjectBaseContract::new(user),
            user,
        }
    }

    /// Validate user has permission to create projects
    fn validate_user_allowed_to_create(&self, errors: &mut ValidationErrors) {
        let can_create = self.user.is_admin()
            || self.user.allowed_globally(permissions::ADD_PROJECT);

        if !can_create {
            errors.add("base", "You are not authorized to create projects");
        }
    }

    /// Validate identifier uniqueness (would check database in real impl)
    fn validate_identifier_unique(&self, _identifier: &str, _errors: &mut ValidationErrors) {
        // In a real implementation, this would check the database
        // For now, we assume it's unique
    }
}

impl<'a, U: UserContext, T: ProjectData> Contract<T> for CreateProjectContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check create permission
        self.validate_user_allowed_to_create(&mut errors);

        // Run base validations
        if let Err(base_errors) = self.base.validate(entity) {
            errors.merge(base_errors);
        }

        // Check identifier uniqueness
        self.validate_identifier_unique(entity.identifier(), &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, attribute: &str) -> bool {
        matches!(
            attribute,
            "identifier" | "name" | "description" | "public" | "parent_id" |
            "status_code" | "status_explanation" | "templated"
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
        global_permissions: HashSet<String>,
    }

    impl UserContext for MockUser {
        fn id(&self) -> Id { self.id }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, _permission: &str, _project_id: Id) -> bool { false }
        fn allowed_globally(&self, permission: &str) -> bool {
            self.global_permissions.contains(permission)
        }
    }

    struct MockProject {
        identifier: String,
        name: String,
    }

    impl ProjectData for MockProject {
        fn id(&self) -> Option<Id> { None }
        fn identifier(&self) -> &str { &self.identifier }
        fn name(&self) -> &str { &self.name }
        fn description(&self) -> Option<&str> { None }
        fn public(&self) -> bool { false }
        fn parent_id(&self) -> Option<Id> { None }
        fn active(&self) -> bool { true }
    }

    #[test]
    fn test_admin_can_create() {
        let user = MockUser { id: 1, admin: true, global_permissions: HashSet::new() };
        let contract = CreateProjectContract::new(&user);

        let project = MockProject {
            identifier: "test-project".to_string(),
            name: "Test Project".to_string(),
        };

        assert!(contract.validate(&project).is_ok());
    }

    #[test]
    fn test_user_with_permission_can_create() {
        let mut permissions = HashSet::new();
        permissions.insert(permissions::ADD_PROJECT.to_string());

        let user = MockUser { id: 1, admin: false, global_permissions: permissions };
        let contract = CreateProjectContract::new(&user);

        let project = MockProject {
            identifier: "test-project".to_string(),
            name: "Test Project".to_string(),
        };

        assert!(contract.validate(&project).is_ok());
    }

    #[test]
    fn test_user_without_permission_cannot_create() {
        let user = MockUser { id: 1, admin: false, global_permissions: HashSet::new() };
        let contract = CreateProjectContract::new(&user);

        let project = MockProject {
            identifier: "test-project".to_string(),
            name: "Test Project".to_string(),
        };

        let result = contract.validate(&project);
        assert!(result.is_err());
    }

    #[test]
    fn test_writable_attributes() {
        let user = MockUser { id: 1, admin: true, global_permissions: HashSet::new() };
        let contract = CreateProjectContract::new(&user);

        // Use fully qualified syntax for trait method
        type CreateContract<'a> = CreateProjectContract<'a, MockUser>;
        assert!(<CreateContract<'_> as Contract<MockProject>>::is_writable(&contract, "identifier"));
        assert!(<CreateContract<'_> as Contract<MockProject>>::is_writable(&contract, "name"));
        assert!(<CreateContract<'_> as Contract<MockProject>>::is_writable(&contract, "public"));
        assert!(!<CreateContract<'_> as Contract<MockProject>>::is_writable(&contract, "created_at"));
    }
}
