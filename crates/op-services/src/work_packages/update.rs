//! Update Service for Work Packages
//!
//! Mirrors: app/services/work_packages/update_service.rb

use op_contracts::base::UserContext;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::set_attributes::{SetAttributesService, WorkPackageEntity};
use super::WorkPackageParams;

/// Service for updating work packages
///
/// # Example
/// ```ignore
/// let service = UpdateWorkPackageService::new(&user);
/// let params = WorkPackageParams::new()
///     .with_subject("Updated Subject");
/// let result = service.call(work_package, params);
/// ```
pub struct UpdateWorkPackageService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> UpdateWorkPackageService<'a, U> {
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

    /// Execute the update operation
    pub fn call(
        self,
        work_package: WorkPackageEntity,
        params: WorkPackageParams,
    ) -> ServiceResult<WorkPackageEntity> {
        // Ensure we're updating an existing work package
        if work_package.id.is_none() {
            return ServiceResult::failure_with_base_error("Cannot update a new work package");
        }

        // Store original values for change detection
        let original_project_id = work_package.project_id;
        let original_status_id = work_package.status_id;
        let original_parent_id = work_package.parent_id;

        // Set attributes and validate
        let set_attrs_service = SetAttributesService::new(self.user, work_package);
        let result = set_attrs_service.call(&params);

        if result.is_failure() {
            return result;
        }

        let work_package = result.unwrap();

        // In a real implementation, this would:
        // 1. Persist changes to database
        // 2. Create journal entry
        // 3. Handle project change
        // 4. Update ancestors
        // 5. Reschedule related work packages

        let mut service_result = ServiceResult::success(work_package.clone());

        // Handle project change
        if work_package.project_id != original_project_id {
            // Would handle project change here (move to different project)
        }

        // Handle status change
        if work_package.status_id != original_status_id {
            // Would handle status transition validations here
        }

        // Handle parent change
        if work_package.parent_id != original_parent_id {
            // Would update hierarchy here
        }

        // Handle notifications
        if self.send_notifications && params.send_notifications {
            // Would send update notifications here
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

    fn create_existing_work_package() -> WorkPackageEntity {
        let mut wp = WorkPackageEntity::new(1, 1, 1);
        wp.id = Some(100);
        wp.subject = "Original Subject".to_string();
        wp
    }

    #[test]
    fn test_update_work_package_subject() {
        let user = create_admin_user();
        let work_package = create_existing_work_package();
        let service = UpdateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("Updated Subject");

        let result = service.call(work_package, params);
        assert!(result.is_success());

        let wp = result.result().unwrap();
        assert_eq!(wp.subject, "Updated Subject");
    }

    #[test]
    fn test_update_work_package_multiple_fields() {
        let user = create_admin_user();
        let work_package = create_existing_work_package();
        let service = UpdateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("New Subject")
            .with_description("New description")
            .with_done_ratio(50);

        let result = service.call(work_package, params);
        assert!(result.is_success());

        let wp = result.result().unwrap();
        assert_eq!(wp.subject, "New Subject");
        assert_eq!(wp.description, Some("New description".to_string()));
        assert_eq!(wp.done_ratio, 50);
    }

    #[test]
    fn test_cannot_update_new_work_package() {
        let user = create_admin_user();
        let work_package = WorkPackageEntity::new(1, 1, 1); // No ID = new
        let service = UpdateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("Should Fail");

        let result = service.call(work_package, params);
        assert!(result.is_failure());
    }

    #[test]
    fn test_update_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let work_package = create_existing_work_package();
        let service = UpdateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("Unauthorized Update");

        let result = service.call(work_package, params);
        assert!(result.is_failure());
    }
}
