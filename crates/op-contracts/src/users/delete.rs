//! Delete contract for users
//!
//! Mirrors: app/contracts/users/delete_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};

/// Contract for deleting a user
pub struct DeleteUserContract<'a, U: UserContext> {
    user: &'a U,
    target_user_id: Id,
}

impl<'a, U: UserContext> DeleteUserContract<'a, U> {
    pub fn new(user: &'a U, target_user_id: Id) -> Self {
        Self { user, target_user_id }
    }

    /// Validate user has permission to delete users
    fn validate_user_allowed_to_delete(&self, errors: &mut ValidationErrors) {
        if !self.user.is_admin() {
            errors.add("base", "Only administrators can delete users");
        }
    }

    /// Validate not deleting self
    fn validate_not_self_delete(&self, errors: &mut ValidationErrors) {
        if self.user.id() == self.target_user_id {
            errors.add("base", "You cannot delete your own account");
        }
    }

    /// Get the target user ID
    pub fn target_user_id(&self) -> Id {
        self.target_user_id
    }
}

/// Data needed for delete validation
pub struct DeleteUserData {
    pub id: Id,
    pub is_builtin: bool,
    pub work_package_count: i64,
}

impl<'a, U: UserContext> Contract<DeleteUserData> for DeleteUserContract<'a, U> {
    fn validate(&self, entity: &DeleteUserData) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check permissions
        self.validate_user_allowed_to_delete(&mut errors);
        self.validate_not_self_delete(&mut errors);

        // Can't delete builtin users (anonymous, system)
        if entity.is_builtin {
            errors.add("base", "Cannot delete built-in users");
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, _attribute: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserContext {
        id: Id,
        admin: bool,
    }

    impl UserContext for MockUserContext {
        fn id(&self) -> Id { self.id }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, _: &str, _: Id) -> bool { false }
        fn allowed_globally(&self, _: &str) -> bool { self.admin }
    }

    #[test]
    fn test_admin_can_delete() {
        let ctx = MockUserContext { id: 1, admin: true };
        let contract = DeleteUserContract::new(&ctx, 2);

        let data = DeleteUserData {
            id: 2,
            is_builtin: false,
            work_package_count: 0,
        };

        assert!(contract.validate(&data).is_ok());
    }

    #[test]
    fn test_non_admin_cannot_delete() {
        let ctx = MockUserContext { id: 1, admin: false };
        let contract = DeleteUserContract::new(&ctx, 2);

        let data = DeleteUserData {
            id: 2,
            is_builtin: false,
            work_package_count: 0,
        };

        let result = contract.validate(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_delete_self() {
        let ctx = MockUserContext { id: 1, admin: true };
        let contract = DeleteUserContract::new(&ctx, 1);

        let data = DeleteUserData {
            id: 1,
            is_builtin: false,
            work_package_count: 0,
        };

        let result = contract.validate(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_delete_builtin() {
        let ctx = MockUserContext { id: 1, admin: true };
        let contract = DeleteUserContract::new(&ctx, 2);

        let data = DeleteUserData {
            id: 2,
            is_builtin: true, // Anonymous user
            work_package_count: 0,
        };

        let result = contract.validate(&data);
        assert!(result.is_err());
    }
}
