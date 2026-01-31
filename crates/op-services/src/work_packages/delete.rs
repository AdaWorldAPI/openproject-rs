//! Delete Service for Work Packages
//!
//! Mirrors: app/services/work_packages/delete_service.rb

use op_contracts::base::{Contract, UserContext};
use op_contracts::work_packages::{DeleteWorkPackageContract, DeleteWorkPackageData};
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::set_attributes::WorkPackageEntity;

/// Service for deleting work packages
///
/// # Example
/// ```ignore
/// let service = DeleteWorkPackageService::new(&user);
/// let result = service.call(work_package);
/// ```
pub struct DeleteWorkPackageService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> DeleteWorkPackageService<'a, U> {
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
    pub fn call(self, work_package: &WorkPackageEntity) -> ServiceResult<()> {
        // Ensure work package exists (has an ID)
        let work_package_id = match work_package.id {
            Some(id) => id,
            None => {
                return ServiceResult::failure_with_base_error(
                    "Cannot delete a work package that doesn't exist",
                );
            }
        };

        // Validate delete permission through contract
        let contract =
            DeleteWorkPackageContract::new(self.user, work_package.project_id, work_package_id);
        let delete_data = DeleteWorkPackageData {
            id: work_package_id,
            project_id: work_package.project_id,
        };
        if let Err(errors) = contract.validate(&delete_data) {
            return ServiceResult::failure(errors);
        }

        // In a real implementation, this would:
        // 1. Handle children (delete or orphan them)
        // 2. Delete attachments
        // 3. Delete journals
        // 4. Delete time entries (or re-assign them)
        // 5. Update ancestors
        // 6. Delete the work package
        // 7. Send notifications

        let service_result = ServiceResult::success(());

        // Handle children (would be recursive delete or orphan)
        // self.handle_children(work_package, &mut service_result);

        // Delete the work package (would call database)
        // self.do_delete(work_package)?;

        // Send notifications
        if self.send_notifications {
            // Would send delete notifications here
        }

        service_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn create_user_with_delete_permission() -> MockUser {
        let mut permissions = std::collections::HashMap::new();
        let mut project_perms = HashSet::new();
        project_perms.insert("delete_work_packages".to_string());
        permissions.insert(1, project_perms);

        MockUser {
            id: 2,
            admin: false,
            project_permissions: permissions,
        }
    }

    fn create_existing_work_package() -> WorkPackageEntity {
        let mut wp = WorkPackageEntity::new(1, 1, 1);
        wp.id = Some(100);
        wp.subject = "Work Package to Delete".to_string();
        wp
    }

    #[test]
    fn test_delete_as_admin() {
        let user = create_admin_user();
        let work_package = create_existing_work_package();
        let service = DeleteWorkPackageService::new(&user);

        let result = service.call(&work_package);
        assert!(result.is_success());
    }

    #[test]
    fn test_delete_with_permission() {
        let user = create_user_with_delete_permission();
        let work_package = create_existing_work_package();
        let service = DeleteWorkPackageService::new(&user);

        let result = service.call(&work_package);
        assert!(result.is_success());
    }

    #[test]
    fn test_cannot_delete_non_existent() {
        let user = create_admin_user();
        let work_package = WorkPackageEntity::new(1, 1, 1); // No ID
        let service = DeleteWorkPackageService::new(&user);

        let result = service.call(&work_package);
        assert!(result.is_failure());
    }

    #[test]
    fn test_delete_without_permission() {
        let user = MockUser {
            id: 3,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let work_package = create_existing_work_package();
        let service = DeleteWorkPackageService::new(&user);

        let result = service.call(&work_package);
        assert!(result.is_failure());
    }
}
