//! Base contract for users
//!
//! Mirrors: app/contracts/users/base_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;
use regex::Regex;
use std::sync::LazyLock;

use crate::base::{Contract, UserContext, ValidationResult};

/// Valid email pattern
static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// Valid login pattern
static LOGIN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9_@.\-]+$").unwrap()
});

/// Reserved login names
const RESERVED_LOGINS: &[&str] = &[
    "admin", "administrator", "anonymous", "system", "root",
    "me", "current", "self",
];

/// User data for validation
pub trait UserData: Send + Sync {
    fn id(&self) -> Option<Id>;
    fn login(&self) -> &str;
    fn firstname(&self) -> &str;
    fn lastname(&self) -> &str;
    fn mail(&self) -> &str;
    fn admin(&self) -> bool;
}

/// Base contract for users with common validations
pub struct UserBaseContract<'a, U: UserContext> {
    user: &'a U,
}

impl<'a, U: UserContext> UserBaseContract<'a, U> {
    pub fn new(user: &'a U) -> Self {
        Self { user }
    }

    /// Validate login format
    pub fn validate_login(&self, login: &str, errors: &mut ValidationErrors) {
        if login.is_empty() {
            errors.add("login", "can't be blank");
            return;
        }

        if login.len() < 2 {
            errors.add("login", "is too short (minimum is 2 characters)");
            return;
        }

        if login.len() > 255 {
            errors.add("login", "is too long (maximum is 255 characters)");
            return;
        }

        if !LOGIN_PATTERN.is_match(login) {
            errors.add("login", "is invalid. Only letters, numbers, underscores, @, periods and dashes allowed");
        }

        // Check reserved logins (only for new users)
        if RESERVED_LOGINS.contains(&login.to_lowercase().as_str()) {
            errors.add("login", "is reserved");
        }
    }

    /// Validate firstname
    pub fn validate_firstname(&self, firstname: &str, errors: &mut ValidationErrors) {
        if firstname.is_empty() {
            errors.add("firstname", "can't be blank");
        } else if firstname.len() > 255 {
            errors.add("firstname", "is too long (maximum is 255 characters)");
        }
    }

    /// Validate lastname
    pub fn validate_lastname(&self, lastname: &str, errors: &mut ValidationErrors) {
        if lastname.is_empty() {
            errors.add("lastname", "can't be blank");
        } else if lastname.len() > 255 {
            errors.add("lastname", "is too long (maximum is 255 characters)");
        }
    }

    /// Validate email format
    pub fn validate_email(&self, email: &str, errors: &mut ValidationErrors) {
        if email.is_empty() {
            errors.add("mail", "can't be blank");
            return;
        }

        if !EMAIL_PATTERN.is_match(email) {
            errors.add("mail", "is not a valid email address");
        }
    }

    /// Validate admin flag changes
    pub fn validate_admin_change(&self, _new_admin: bool, errors: &mut ValidationErrors) {
        // Only admins can set admin flag
        if !self.user.is_admin() {
            errors.add("admin", "can only be modified by administrators");
        }
    }

    /// Get the user context
    pub fn user(&self) -> &'a U {
        self.user
    }
}

impl<'a, U: UserContext, T: UserData> Contract<T> for UserBaseContract<'a, U> {
    fn validate(&self, entity: &T) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        self.validate_login(entity.login(), &mut errors);
        self.validate_firstname(entity.firstname(), &mut errors);
        self.validate_lastname(entity.lastname(), &mut errors);
        self.validate_email(entity.mail(), &mut errors);

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

    struct MockUser {
        login: String,
        firstname: String,
        lastname: String,
        mail: String,
    }

    impl UserData for MockUser {
        fn id(&self) -> Option<Id> { None }
        fn login(&self) -> &str { &self.login }
        fn firstname(&self) -> &str { &self.firstname }
        fn lastname(&self) -> &str { &self.lastname }
        fn mail(&self) -> &str { &self.mail }
        fn admin(&self) -> bool { false }
    }

    #[test]
    fn test_valid_user() {
        let ctx = MockUserContext { admin: true };
        let contract = UserBaseContract::new(&ctx);

        let user = MockUser {
            login: "john.doe".to_string(),
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            mail: "john@example.com".to_string(),
        };

        assert!(contract.validate(&user).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let ctx = MockUserContext { admin: true };
        let contract = UserBaseContract::new(&ctx);

        let user = MockUser {
            login: "john.doe".to_string(),
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            mail: "not-an-email".to_string(),
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("mail"));
    }

    #[test]
    fn test_reserved_login() {
        let ctx = MockUserContext { admin: true };
        let contract = UserBaseContract::new(&ctx);

        let user = MockUser {
            login: "admin".to_string(),
            firstname: "Admin".to_string(),
            lastname: "User".to_string(),
            mail: "admin@example.com".to_string(),
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("login"));
    }

    #[test]
    fn test_blank_firstname() {
        let ctx = MockUserContext { admin: true };
        let contract = UserBaseContract::new(&ctx);

        let user = MockUser {
            login: "john.doe".to_string(),
            firstname: "".to_string(),
            lastname: "Doe".to_string(),
            mail: "john@example.com".to_string(),
        };

        let result = contract.validate(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().has_error("firstname"));
    }
}
