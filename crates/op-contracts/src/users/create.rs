//! Create contract for users
//!
//! Mirrors: app/contracts/users/create_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::base::{Contract, UserContext, ValidationResult};
use super::base::{UserBaseContract, UserData};

/// Contract for creating a new user
pub struct CreateUserContract<'a, U: UserContext> {
    base: UserBaseContract<'a, U>,
    user: &'a U,
}

impl<'a, U: UserContext> CreateUserContract<'a, U> {
    pub fn new(user: &'a U) -> Self {
        Self {
            base: UserBaseContract::new(user),
            user,
        }
    }

    /// Validate user has permission to create users
    fn validate_user_allowed_to_create(&self, errors: &mut ValidationErrors) {
        // Only admins can create users
        if !self.user.is_admin() {
            errors.add("base", "Only administrators can create users");
        }
    }

    /// Validate password requirements for new users
    fn validate_password(&self, password: Option<&str>, errors: &mut ValidationErrors) {
        match password {
            None => {
                // Password can be None if using external auth or invitation
            }
            Some(pwd) => {
                if pwd.len() < 10 {
                    errors.add("password", "is too short (minimum is 10 characters)");
                }
                if pwd.len() > 128 {
                    errors.add("password", "is too long (maximum is 128 characters)");
                }
                // Could add more complexity requirements here
            }
        }
    }
}

/// Extended user data for creation
pub trait CreateUserData: UserData {
    fn password(&self) -> Option<&str>;
}

impl<'a, U: UserContext, T: CreateUserData> Contract<T> for CreateUserContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Check create permission
        self.validate_user_allowed_to_create(&mut errors);

        // Run base validations
        if let Err(base_errors) = self.base.validate(entity) {
            errors.merge(base_errors);
        }

        // Validate password
        self.validate_password(entity.password(), &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_writable(&self, attribute: &str) -> bool {
        matches!(
            attribute,
            "login" | "firstname" | "lastname" | "mail" | "password" |
            "admin" | "status" | "language" | "force_password_change" |
            "auth_source_id"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserContext {
        admin: bool,
    }

    impl UserContext for MockUserContext {
        fn id(&self) -> Id { 1 }
        fn is_admin(&self) -> bool { self.admin }
        fn is_anonymous(&self) -> bool { false }
        fn allowed_in_project(&self, _: &str, _: Id) -> bool { false }
        fn allowed_globally(&self, _: &str) -> bool { self.admin }
    }

    struct MockNewUser {
        login: String,
        firstname: String,
        lastname: String,
        mail: String,
        password: Option<String>,
    }

    impl UserData for MockNewUser {
        fn id(&self) -> Option<Id> { None }
        fn login(&self) -> &str { &self.login }
        fn firstname(&self) -> &str { &self.firstname }
        fn lastname(&self) -> &str { &self.lastname }
        fn mail(&self) -> &str { &self.mail }
        fn admin(&self) -> bool { false }
    }

    impl CreateUserData for MockNewUser {
        fn password(&self) -> Option<&str> { self.password.as_deref() }
    }

    #[test]
    fn test_admin_can_create() {
        let ctx = MockUserContext { admin: true };
        let contract = CreateUserContract::new(&ctx);

        let user = MockNewUser {
            login: "newuser".to_string(),
            firstname: "New".to_string(),
            lastname: "User".to_string(),
            mail: "new@example.com".to_string(),
            password: Some("securepassword123".to_string()),
        };

        assert!(contract.validate(&user).is_ok());
    }

    #[test]
    fn test_non_admin_cannot_create() {
        let ctx = MockUserContext { admin: false };
        let contract = CreateUserContract::new(&ctx);

        let user = MockNewUser {
            login: "newuser".to_string(),
            firstname: "New".to_string(),
            lastname: "User".to_string(),
            mail: "new@example.com".to_string(),
            password: Some("securepassword123".to_string()),
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
    }

    #[test]
    fn test_short_password() {
        let ctx = MockUserContext { admin: true };
        let contract = CreateUserContract::new(&ctx);

        let user = MockNewUser {
            login: "newuser".to_string(),
            firstname: "New".to_string(),
            lastname: "User".to_string(),
            mail: "new@example.com".to_string(),
            password: Some("short".to_string()),
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("password"));
    }
}
