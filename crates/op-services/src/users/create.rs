//! Create Service for Users
//!
//! Mirrors: app/services/users/create_service.rb

use op_contracts::base::UserContext;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::set_attributes::{SetAttributesService, UserEntity, status};
use super::UserParams;

/// Service for creating users
///
/// # Example
/// ```ignore
/// let service = CreateUserService::new(&user);
/// let params = UserParams::new()
///     .with_login("johndoe")
///     .with_firstname("John")
///     .with_lastname("Doe")
///     .with_mail("john@example.com")
///     .with_password("securepassword123");
/// let result = service.call(params);
/// ```
pub struct CreateUserService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> CreateUserService<'a, U> {
    pub fn new(user: &'a U) -> Self {
        Self {
            user,
            send_notifications: true,
        }
    }

    pub fn without_notifications(user: &'a U) -> Self {
        Self {
            user,
            send_notifications: false,
        }
    }

    /// Execute the create operation
    pub fn call(self, params: UserParams) -> ServiceResult<UserEntity> {
        // Create new user with defaults
        let mut user_entity = UserEntity::new();
        user_entity.status = status::ACTIVE;

        // Set attributes and validate
        let set_attrs_service = SetAttributesService::new(self.user, user_entity);
        let result = set_attrs_service.call(&params);

        if result.is_failure() {
            return result;
        }

        let mut user_entity = result.unwrap();

        // In a real implementation, this would persist to database
        // For now, we simulate assigning an ID
        user_entity.id = Some(self.next_id());

        // Create result
        let service_result = ServiceResult::success(user_entity.clone());

        // In a real implementation, this would:
        // 1. Hash password
        // 2. Persist user to database
        // 3. Create default preferences
        // 4. Send activation email if invited

        // Handle notifications
        if self.send_notifications && params.send_notifications {
            // Would send user creation notifications here
        }

        service_result
    }

    fn next_id(&self) -> Id {
        // In real implementation, this would come from database
        1000
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
    fn test_create_user_as_admin() {
        let user = create_admin_user();
        let service = CreateUserService::new(&user);

        let params = UserParams::new()
            .with_login("newuser")
            .with_firstname("New")
            .with_lastname("User")
            .with_mail("new@example.com")
            .with_password("securepassword123");

        let result = service.call(params);
        assert!(result.is_success());

        let created = result.result().unwrap();
        assert_eq!(created.login, "newuser");
        assert_eq!(created.firstname, "New");
        assert!(created.id.is_some());
    }

    #[test]
    fn test_create_user_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            global_permissions: HashSet::new(),
        };
        let service = CreateUserService::new(&user);

        let params = UserParams::new()
            .with_login("unauthorized")
            .with_firstname("Unauthorized")
            .with_lastname("User")
            .with_mail("unauth@example.com")
            .with_password("securepassword123");

        let result = service.call(params);
        assert!(result.is_failure());
    }

    #[test]
    fn test_create_user_validation_failure() {
        let user = create_admin_user();
        let service = CreateUserService::new(&user);

        // Missing login should fail validation
        let params = UserParams::new()
            .with_firstname("No")
            .with_lastname("Login")
            .with_mail("nologin@example.com");

        let result = service.call(params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("login"));
    }

    #[test]
    fn test_create_user_with_admin_flag() {
        let user = create_admin_user();
        let service = CreateUserService::new(&user);

        let params = UserParams::new()
            .with_login("adminuser")
            .with_firstname("Admin")
            .with_lastname("User")
            .with_mail("admin@example.com")
            .with_password("securepassword123")
            .with_admin(true);

        let result = service.call(params);
        assert!(result.is_success());

        let created = result.result().unwrap();
        assert!(created.admin);
    }

    #[test]
    fn test_create_without_notifications() {
        let user = create_admin_user();
        let service = CreateUserService::without_notifications(&user);

        let params = UserParams::new()
            .with_login("silentuser")
            .with_firstname("Silent")
            .with_lastname("User")
            .with_mail("silent@example.com")
            .with_password("securepassword123");

        let result = service.call(params);
        assert!(result.is_success());
    }
}
