//! Update contract for users
//!
//! Mirrors: app/contracts/users/update_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{ChangeTracker, Contract, UserContext, ValidationResult};
use super::base::{UserBaseContract, UserData};

/// Contract for updating an existing user
pub struct UpdateUserContract<'a, U: UserContext> {
    base: UserBaseContract<'a, U>,
    user: &'a U,
    target_user_id: Id,
    changes: ChangeTracker,
}

impl<'a, U: UserContext> UpdateUserContract<'a, U> {
    pub fn new(user: &'a U, target_user_id: Id) -> Self {
        Self {
            base: UserBaseContract::new(user),
            user,
            target_user_id,
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

    /// Validate user has permission to edit this user
    fn validate_user_allowed_to_edit(&self, errors: &mut ValidationErrors) {
        let is_self_edit = self.user.id() == self.target_user_id;
        let is_admin = self.user.is_admin();

        if !is_admin && !is_self_edit {
            errors.add("base", "You can only edit your own account or need administrator privileges");
        }
    }

    /// Validate admin flag changes
    fn validate_admin_change(&self, new_admin: bool, errors: &mut ValidationErrors) {
        if !self.changes.is_changed("admin") {
            return;
        }

        // Only admins can change admin status
        if !self.user.is_admin() {
            errors.add("admin", "can only be modified by administrators");
            return;
        }

        // Can't remove own admin status
        if self.user.id() == self.target_user_id && !new_admin {
            errors.add("admin", "you cannot remove your own administrator status");
        }
    }

    /// Validate login change
    fn validate_login_change(&self, errors: &mut ValidationErrors) {
        if !self.changes.is_changed("login") {
            return;
        }

        // Only admins can change login
        if !self.user.is_admin() {
            errors.add("login", "can only be changed by administrators");
        }
    }

    /// Get the target user ID
    pub fn target_user_id(&self) -> Id {
        self.target_user_id
    }
}

impl<'a, U: UserContext, T: UserData> Contract<T> for UpdateUserContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check edit permission
        self.validate_user_allowed_to_edit(&mut errors);

        // Check specific field permissions
        self.validate_admin_change(entity.admin(), &mut errors);
        self.validate_login_change(&mut errors);

        // Run base validations
        if let Err(base_errors) = self.base.validate(entity) {
            errors.merge(base_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, attribute: &str) -> bool {
        let is_admin = self.user.is_admin();
        let is_self = self.user.id() == self.target_user_id;

        match attribute {
            // Admin-only fields
            "login" | "admin" | "status" | "auth_source_id" => is_admin,
            // User can edit their own profile
            "firstname" | "lastname" | "mail" | "language" => is_admin || is_self,
            // Password change (special handling)
            "password" => is_admin || is_self,
            _ => false,
        }
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

    struct MockUser {
        id: Option<Id>,
        login: String,
        firstname: String,
        lastname: String,
        mail: String,
        admin: bool,
    }

    impl UserData for MockUser {
        fn id(&self) -> Option<Id> { self.id }
        fn login(&self) -> &str { &self.login }
        fn firstname(&self) -> &str { &self.firstname }
        fn lastname(&self) -> &str { &self.lastname }
        fn mail(&self) -> &str { &self.mail }
        fn admin(&self) -> bool { self.admin }
    }

    #[test]
    fn test_admin_can_update_any_user() {
        let ctx = MockUserContext { id: 1, admin: true };
        let contract = UpdateUserContract::new(&ctx, 2);

        let user = MockUser {
            id: Some(2),
            login: "otheruser".to_string(),
            firstname: "Other".to_string(),
            lastname: "User".to_string(),
            mail: "other@example.com".to_string(),
            admin: false,
        };

        assert!(contract.validate(&user).is_ok());
    }

    #[test]
    fn test_user_can_update_self() {
        let ctx = MockUserContext { id: 1, admin: false };
        let contract = UpdateUserContract::new(&ctx, 1);

        let user = MockUser {
            id: Some(1),
            login: "myself".to_string(),
            firstname: "Updated".to_string(),
            lastname: "Name".to_string(),
            mail: "me@example.com".to_string(),
            admin: false,
        };

        assert!(contract.validate(&user).is_ok());
    }

    #[test]
    fn test_user_cannot_update_other() {
        let ctx = MockUserContext { id: 1, admin: false };
        let contract = UpdateUserContract::new(&ctx, 2);

        let user = MockUser {
            id: Some(2),
            login: "otheruser".to_string(),
            firstname: "Other".to_string(),
            lastname: "User".to_string(),
            mail: "other@example.com".to_string(),
            admin: false,
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_cannot_remove_own_admin() {
        let ctx = MockUserContext { id: 1, admin: true };
        let mut contract = UpdateUserContract::new(&ctx, 1);
        contract.mark_changed("admin");

        let user = MockUser {
            id: Some(1),
            login: "admin".to_string(),
            firstname: "Admin".to_string(),
            lastname: "User".to_string(),
            mail: "admin@example.com".to_string(),
            admin: false, // Trying to remove admin
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("admin"));
    }

    #[test]
    fn test_writable_attributes() {
        let ctx = MockUserContext { id: 1, admin: true };
        let contract = UpdateUserContract::new(&ctx, 2);

        // Use fully qualified syntax for trait method
        type AdminContract<'a> = UpdateUserContract<'a, MockUserContext>;
        assert!(<AdminContract<'_> as Contract<MockUser>>::is_writable(&contract, "login"));
        assert!(<AdminContract<'_> as Contract<MockUser>>::is_writable(&contract, "admin"));
        assert!(<AdminContract<'_> as Contract<MockUser>>::is_writable(&contract, "firstname"));

        let non_admin_ctx = MockUserContext { id: 1, admin: false };
        let non_admin_contract = UpdateUserContract::new(&non_admin_ctx, 1);

        assert!(!<AdminContract<'_> as Contract<MockUser>>::is_writable(&non_admin_contract, "login"));
        assert!(!<AdminContract<'_> as Contract<MockUser>>::is_writable(&non_admin_contract, "admin"));
        assert!(<AdminContract<'_> as Contract<MockUser>>::is_writable(&non_admin_contract, "firstname"));
    }
}
