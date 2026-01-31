//! Base contract for projects
//!
//! Mirrors: app/contracts/projects/base_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;
use regex::Regex;
use std::sync::LazyLock;

use crate::base::{Contract, UserContext, ValidationResult};

/// Valid identifier pattern (alphanumeric, hyphens, underscores)
static IDENTIFIER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9][a-z0-9_-]*[a-z0-9]$|^[a-z0-9]$").unwrap()
});

/// Reserved identifiers that cannot be used
const RESERVED_IDENTIFIERS: &[&str] = &[
    "new", "edit", "delete", "destroy", "create", "update",
    "admin", "api", "settings", "logout", "login",
];

/// Project data for validation
pub trait ProjectData: Send + Sync {
    fn id(&self) -> Option<Id>;
    fn identifier(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn public(&self) -> bool;
    fn parent_id(&self) -> Option<Id>;
    fn active(&self) -> bool;
}

/// Base contract for projects with common validations
pub struct ProjectBaseContract<'a, U: UserContext> {
    user: &'a U,
}

impl<'a, U: UserContext> ProjectBaseContract<'a, U> {
    pub fn new(user: &'a U) -> Self {
        Self { user }
    }

    /// Validate identifier format and uniqueness
    pub fn validate_identifier(&self, identifier: &str, errors: &mut ValidationErrors) {
        if identifier.is_empty() {
            errors.add("identifier", "can't be blank");
            return;
        }

        if identifier.len() < 1 || identifier.len() > 100 {
            errors.add("identifier", "is too long (maximum is 100 characters)");
            return;
        }

        if !IDENTIFIER_PATTERN.is_match(identifier) {
            errors.add(
                "identifier",
                "is invalid. Only lowercase letters, numbers, dashes and underscores allowed. \
                 It must start with a letter or number.",
            );
        }

        if RESERVED_IDENTIFIERS.contains(&identifier) {
            errors.add("identifier", "is reserved and cannot be used");
        }
    }

    /// Validate name is present and within length
    pub fn validate_name(&self, name: &str, errors: &mut ValidationErrors) {
        if name.trim().is_empty() {
            errors.add("name", "can't be blank");
        } else if name.len() > 255 {
            errors.add("name", "is too long (maximum is 255 characters)");
        }
    }

    /// Validate parent project exists and user has access (if set)
    pub fn validate_parent(&self, parent_id: Option<Id>, errors: &mut ValidationErrors) {
        if let Some(pid) = parent_id {
            // Check user has permission to add subproject to parent
            if !self.user.is_admin() && !self.user.allowed_in_project(super::permissions::ADD_PROJECT, pid) {
                errors.add("parent", "is not accessible or you don't have permission to add subprojects");
            }
        }
    }

    /// Get the user context
    pub fn user(&self) -> &'a U {
        self.user
    }
}

impl<'a, U: UserContext, T: ProjectData> Contract<T> for ProjectBaseContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        self.validate_identifier(entity.identifier(), &mut errors);
        self.validate_name(entity.name(), &mut errors);
        self.validate_parent(entity.parent_id(), &mut errors);

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
    }

    impl UserContext for MockUser {
        fn id(&self) -> Id { self.id }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, _permission: &str, _project_id: Id) -> bool { false }
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
    fn test_valid_project() {
        let user = MockUser { id: 1, admin: true };
        let contract = ProjectBaseContract::new(&user);

        let project = MockProject {
            id: None,
            identifier: "my-project".to_string(),
            name: "My Project".to_string(),
            parent_id: None,
        };

        assert!(contract.validate(&project).is_ok());
    }

    #[test]
    fn test_blank_name() {
        let user = MockUser { id: 1, admin: true };
        let contract = ProjectBaseContract::new(&user);

        let project = MockProject {
            id: None,
            identifier: "my-project".to_string(),
            name: "".to_string(),
            parent_id: None,
        };

        let result = contract.validate(&project);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("name"));
    }

    #[test]
    fn test_invalid_identifier() {
        let user = MockUser { id: 1, admin: true };
        let contract = ProjectBaseContract::new(&user);

        let project = MockProject {
            id: None,
            identifier: "My Project!".to_string(), // Invalid: uppercase and special chars
            name: "My Project".to_string(),
            parent_id: None,
        };

        let result = contract.validate(&project);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("identifier"));
    }

    #[test]
    fn test_reserved_identifier() {
        let user = MockUser { id: 1, admin: true };
        let contract = ProjectBaseContract::new(&user);

        let project = MockProject {
            id: None,
            identifier: "admin".to_string(),
            name: "Admin Project".to_string(),
            parent_id: None,
        };

        let result = contract.validate(&project);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("identifier"));
    }
}
