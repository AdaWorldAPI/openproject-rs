//! Create Service for Work Packages
//!
//! Mirrors: app/services/work_packages/create_service.rb

use op_contracts::base::UserContext;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::set_attributes::{SetAttributesService, WorkPackageEntity};
use super::WorkPackageParams;

/// Service for creating work packages
///
/// # Example
/// ```ignore
/// let service = CreateWorkPackageService::new(&user);
/// let params = WorkPackageParams::new()
///     .with_subject("New Feature")
///     .with_project_id(1)
///     .with_type_id(1);
/// let result = service.call(params);
/// ```
pub struct CreateWorkPackageService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> CreateWorkPackageService<'a, U> {
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
    pub fn call(self, params: WorkPackageParams) -> ServiceResult<WorkPackageEntity> {
        // Create new work package with defaults
        let project_id = params.project_id.unwrap_or(1);
        let type_id = params.type_id.unwrap_or(1);
        let work_package = WorkPackageEntity::new(project_id, type_id, self.user.id());

        // Set attributes and validate
        let set_attrs_service = SetAttributesService::new(self.user, work_package);
        let result = set_attrs_service.call(&params);

        if result.is_failure() {
            return result;
        }

        let mut work_package = result.unwrap();

        // In a real implementation, this would persist to database
        // For now, we simulate assigning an ID
        work_package.id = Some(self.next_id());

        // Create result
        let mut service_result = ServiceResult::success(work_package.clone());

        // Handle notifications (would be implemented with actual notification service)
        if self.send_notifications && params.send_notifications {
            // Would send notifications here
        }

        // Handle ancestor updates (would be implemented with UpdateAncestors service)
        // self.update_ancestors(&work_package, &mut service_result);

        // Handle scheduling (would be implemented with SetScheduleService)
        // self.reschedule_related(&work_package, &mut service_result);

        // Add user as watcher (would be implemented with Watchers service)
        // self.set_user_as_watcher(&work_package);

        service_result
    }

    fn next_id(&self) -> Id {
        // In real implementation, this would come from database
        // For now, use a simple counter simulation
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

    fn create_user_with_permission() -> MockUser {
        let mut permissions = std::collections::HashMap::new();
        let mut project_perms = HashSet::new();
        project_perms.insert("add_work_packages".to_string());
        permissions.insert(1, project_perms);

        MockUser {
            id: 2,
            admin: false,
            project_permissions: permissions,
        }
    }

    #[test]
    fn test_create_work_package_as_admin() {
        let user = create_admin_user();
        let service = CreateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("New Feature")
            .with_project_id(1)
            .with_type_id(1);

        let result = service.call(params);
        assert!(result.is_success());

        let wp = result.result().unwrap();
        assert_eq!(wp.subject, "New Feature");
        assert!(wp.id.is_some());
        assert_eq!(wp.author_id, user.id);
    }

    #[test]
    fn test_create_work_package_with_permission() {
        let user = create_user_with_permission();
        let service = CreateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("Bug Fix")
            .with_project_id(1)
            .with_type_id(1);

        let result = service.call(params);
        assert!(result.is_success());
    }

    #[test]
    fn test_create_work_package_without_permission() {
        let user = MockUser {
            id: 3,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let service = CreateWorkPackageService::new(&user);

        let params = WorkPackageParams::new()
            .with_subject("Unauthorized")
            .with_project_id(1)
            .with_type_id(1);

        let result = service.call(params);
        assert!(result.is_failure());
    }

    #[test]
    fn test_create_work_package_validation_failure() {
        let user = create_admin_user();
        let service = CreateWorkPackageService::new(&user);

        // Missing subject
        let params = WorkPackageParams::new()
            .with_project_id(1)
            .with_type_id(1);

        let result = service.call(params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("subject"));
    }

    #[test]
    fn test_create_without_notifications() {
        let user = create_admin_user();
        let service = CreateWorkPackageService::without_notifications(&user);

        let params = WorkPackageParams::new()
            .with_subject("Silent Feature")
            .with_project_id(1);

        let result = service.call(params);
        assert!(result.is_success());
    }
}
