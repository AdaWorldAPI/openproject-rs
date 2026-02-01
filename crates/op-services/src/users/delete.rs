//! Delete Service for Users
//!
//! Mirrors: app/services/users/delete_service.rb

use op_contracts::base::{Contract, UserContext};
use op_contracts::users::{DeleteUserContract, DeleteUserData};

use crate::result::ServiceResult;
use super::set_attributes::UserEntity;

/// Service for deleting users
///
/// # Example
/// ```ignore
/// let service = DeleteUserService::new(&user);
/// let result = service.call(user_entity);
/// ```
pub struct DeleteUserService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> DeleteUserService<'a, U> {
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

    /// Execute the delete operation
    pub fn call(self, user_entity: UserEntity) -> ServiceResult<UserEntity> {
        // Ensure we're deleting an existing user
        let user_id = match user_entity.id {
            Some(id) => id,
            None => {
                return ServiceResult::failure_with_base_error("Cannot delete a new user");
            }
        };

        // Validate permissions using contract
        if let Err(errors) = self.validate(&user_entity) {
            return ServiceResult::failure(errors);
        }

        // In a real implementation, this would:
        // 1. Anonymize user data (GDPR compliance)
        // 2. Reassign or delete work packages
        // 3. Remove project memberships
        // 4. Delete user preferences
        // 5. Clean up notifications
        // 6. Remove API tokens
        // 7. Delete OAuth grants
        // 8. Archive or delete journals
        // 9. Handle time entries

        let service_result = ServiceResult::success(user_entity.clone());

        // Handle notifications
        if self.send_notifications {
            // Would send deletion notifications here
        }

        service_result
    }

    /// Lock the user instead of deleting
    pub fn lock(self, mut user_entity: UserEntity) -> ServiceResult<UserEntity> {
        // Ensure we're locking an existing user
        if user_entity.id.is_none() {
            return ServiceResult::failure_with_base_error("Cannot lock a new user");
        }

        // Only admins can lock users
        if !self.user.is_admin() {
            return ServiceResult::failure_with_base_error("Only administrators can lock users");
        }

        // Set user as locked
        user_entity.status = super::set_attributes::status::LOCKED;

        // In a real implementation, this would:
        // 1. Update user in database
        // 2. Invalidate all sessions
        // 3. Revoke API tokens

        let service_result = ServiceResult::success(user_entity);

        // Handle notifications
        if self.send_notifications {
            // Would send lock notifications here
        }

        service_result
    }

    fn validate(&self, user_entity: &UserEntity) -> Result<(), op_core::error::ValidationErrors> {
        let user_id = user_entity.id.unwrap_or(0);
        let contract = DeleteUserContract::new(self.user, user_id);

        // Create delete data - in real implementation, these would be fetched from database
        let delete_data = DeleteUserData {
            id: user_id,
            is_builtin: false, // Would be looked up from database
            work_package_count: 0, // Would be counted from database
        };

        contract.validate(&delete_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use op_core::traits::Id;
    use std::collections::HashSet;

    struct MockUser {
        id: Id,
        admin: bool,
        project_permissions: std::collections::HashMap<Id, HashSet<String>>,
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

        fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool {
            self.project_permissions
                .get(&project_id)
                .map(|perms| perms.contains(permission))
                .unwrap_or(false)
        }

        fn allowed_globally(&self, _permission: &str) -> bool {
            false
        }
    }

    fn create_admin_user() -> MockUser {
        MockUser {
            id: 1,
            admin: true,
            project_permissions: std::collections::HashMap::new(),
        }
    }

    fn create_existing_user() -> UserEntity {
        let mut user = UserEntity::new();
        user.id = Some(100);
        user.login = "todelete".to_string();
        user.firstname = "To".to_string();
        user.lastname = "Delete".to_string();
        user.mail = "delete@example.com".to_string();
        user
    }

    #[test]
    fn test_delete_user_as_admin() {
        let user = create_admin_user();
        let user_entity = create_existing_user();
        let service = DeleteUserService::new(&user);

        let result = service.call(user_entity);
        assert!(result.is_success());
    }

    #[test]
    fn test_cannot_delete_new_user() {
        let user = create_admin_user();
        let user_entity = UserEntity::new(); // No ID = new
        let service = DeleteUserService::new(&user);

        let result = service.call(user_entity);
        assert!(result.is_failure());
    }

    #[test]
    fn test_delete_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let user_entity = create_existing_user();
        let service = DeleteUserService::new(&user);

        let result = service.call(user_entity);
        assert!(result.is_failure());
    }

    #[test]
    fn test_cannot_delete_self() {
        let user = MockUser {
            id: 100, // Same as target user
            admin: true,
            project_permissions: std::collections::HashMap::new(),
        };
        let user_entity = create_existing_user(); // ID is 100
        let service = DeleteUserService::new(&user);

        let result = service.call(user_entity);
        assert!(result.is_failure());
    }

    #[test]
    fn test_lock_user_as_admin() {
        let user = create_admin_user();
        let user_entity = create_existing_user();
        let service = DeleteUserService::new(&user);

        let result = service.lock(user_entity);
        assert!(result.is_success());

        let locked = result.result().unwrap();
        assert!(locked.is_locked());
    }

    #[test]
    fn test_cannot_lock_new_user() {
        let user = create_admin_user();
        let user_entity = UserEntity::new();
        let service = DeleteUserService::new(&user);

        let result = service.lock(user_entity);
        assert!(result.is_failure());
    }

    #[test]
    fn test_lock_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let user_entity = create_existing_user();
        let service = DeleteUserService::new(&user);

        let result = service.lock(user_entity);
        assert!(result.is_failure());
    }

    #[test]
    fn test_delete_without_notifications() {
        let user = create_admin_user();
        let user_entity = create_existing_user();
        let service = DeleteUserService::without_notifications(&user);

        let result = service.call(user_entity);
        assert!(result.is_success());
    }
}
