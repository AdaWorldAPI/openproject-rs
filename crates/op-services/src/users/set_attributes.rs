//! Set Attributes Service for Users
//!
//! Mirrors: app/services/users/set_attributes_service.rb

use op_contracts::base::UserContext;
use op_contracts::users::{CreateUserContract, CreateUserData, UserData};
use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::UserParams;

/// User status constants
pub mod status {
    pub const REGISTERED: i32 = 2;
    pub const ACTIVE: i32 = 1;
    pub const LOCKED: i32 = 3;
    pub const INVITED: i32 = 4;
}

/// User entity for service operations
#[derive(Debug, Clone, Default)]
pub struct UserEntity {
    pub id: Option<Id>,
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub password: Option<String>,
    pub admin: bool,
    pub status: i32,
    pub language: Option<String>,
    pub force_password_change: bool,
}

impl UserEntity {
    pub fn new() -> Self {
        Self {
            id: None,
            login: String::new(),
            firstname: String::new(),
            lastname: String::new(),
            mail: String::new(),
            password: None,
            admin: false,
            status: status::ACTIVE,
            language: None,
            force_password_change: false,
        }
    }

    pub fn is_new(&self) -> bool {
        self.id.is_none()
    }

    pub fn is_active(&self) -> bool {
        self.status == status::ACTIVE
    }

    pub fn is_locked(&self) -> bool {
        self.status == status::LOCKED
    }
}

/// Implement UserData trait for UserEntity
impl UserData for UserEntity {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn login(&self) -> &str {
        &self.login
    }

    fn firstname(&self) -> &str {
        &self.firstname
    }

    fn lastname(&self) -> &str {
        &self.lastname
    }

    fn mail(&self) -> &str {
        &self.mail
    }

    fn admin(&self) -> bool {
        self.admin
    }
}

/// Implement CreateUserData trait for UserEntity
impl CreateUserData for UserEntity {
    fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }
}

/// Service for setting attributes on a user
pub struct SetAttributesService<'a, U: UserContext> {
    user: &'a U,
    model: UserEntity,
}

impl<'a, U: UserContext> SetAttributesService<'a, U> {
    pub fn new(user: &'a U, model: UserEntity) -> Self {
        Self { user, model }
    }

    /// Set attributes from params and validate
    pub fn call(mut self, params: &UserParams) -> ServiceResult<UserEntity> {
        // Set attributes from params
        self.set_attributes(params);

        // Run contract validation
        let validation_result = self.validate();

        if let Err(errors) = validation_result {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(self.model)
    }

    fn set_attributes(&mut self, params: &UserParams) {
        if let Some(ref login) = params.login {
            self.model.login = login.clone();
        }
        if let Some(ref firstname) = params.firstname {
            self.model.firstname = firstname.clone();
        }
        if let Some(ref lastname) = params.lastname {
            self.model.lastname = lastname.clone();
        }
        if let Some(ref mail) = params.mail {
            self.model.mail = mail.clone();
        }
        if let Some(ref password) = params.password {
            self.model.password = Some(password.clone());
        }
        if let Some(admin) = params.admin {
            // Only admins can set admin flag
            if self.user.is_admin() {
                self.model.admin = admin;
            }
        }
        if let Some(status) = params.status {
            self.model.status = status;
        }
        if let Some(ref language) = params.language {
            self.model.language = Some(language.clone());
        }
        if let Some(force) = params.force_password_change {
            self.model.force_password_change = force;
        }
    }

    fn validate(&self) -> Result<(), ValidationErrors> {
        use op_contracts::base::Contract;

        let contract = CreateUserContract::new(self.user);
        contract.validate(&self.model)
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
        fn id(&self) -> Id {
            self.id
        }

        fn is_admin(&self) -> bool {
            self.admin
        }

        fn is_anonymous(&self) -> bool {
            false
        }

        fn allowed_in_project(&self, _permission: &str, _project_id: Id) -> bool {
            false
        }

        fn allowed_globally(&self, permission: &str) -> bool {
            self.global_permissions.contains(permission)
        }
    }

    fn create_admin_user() -> MockUser {
        MockUser {
            id: 1,
            admin: true,
            global_permissions: HashSet::new(),
        }
    }

    #[test]
    fn test_set_attributes_basic() {
        let user = create_admin_user();
        let entity = UserEntity::new();
        let service = SetAttributesService::new(&user, entity);

        let params = UserParams::new()
            .with_login("johndoe")
            .with_firstname("John")
            .with_lastname("Doe")
            .with_mail("john@example.com")
            .with_password("securepassword123");

        let result = service.call(&params);
        assert!(result.is_success());
        let user_entity = result.result().unwrap();
        assert_eq!(user_entity.login, "johndoe");
        assert_eq!(user_entity.firstname, "John");
        assert_eq!(user_entity.lastname, "Doe");
        assert_eq!(user_entity.mail, "john@example.com");
    }

    #[test]
    fn test_set_attributes_validation_fails() {
        let user = create_admin_user();
        let entity = UserEntity::new();
        let service = SetAttributesService::new(&user, entity);

        // Empty login should fail validation
        let params = UserParams::new()
            .with_firstname("John")
            .with_lastname("Doe")
            .with_mail("john@example.com");

        let result = service.call(&params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("login"));
    }
}
