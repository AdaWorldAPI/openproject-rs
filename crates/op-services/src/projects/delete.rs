//! Delete Service for Projects
//!
//! Mirrors: app/services/projects/delete_service.rb

use op_contracts::base::{Contract, UserContext};
use op_contracts::projects::{DeleteProjectContract, DeleteProjectData};

use crate::result::ServiceResult;
use super::set_attributes::ProjectEntity;

/// Service for deleting (archiving or destroying) projects
///
/// # Example
/// ```ignore
/// let service = DeleteProjectService::new(&user);
/// let result = service.call(project);
/// ```
pub struct DeleteProjectService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> DeleteProjectService<'a, U> {
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
    ///
    /// In OpenProject, project deletion is typically a soft delete (archive)
    /// or a scheduled background job for hard delete due to the amount
    /// of associated data.
    pub fn call(self, project: ProjectEntity) -> ServiceResult<ProjectEntity> {
        // Ensure we're deleting an existing project
        if project.id.is_none() {
            return ServiceResult::failure_with_base_error("Cannot delete a new project");
        }

        // Validate permissions using contract
        if let Err(errors) = self.validate(&project) {
            return ServiceResult::failure(errors);
        }

        // In a real implementation, this would:
        // 1. Check for subprojects and handle accordingly
        // 2. Archive or delete associated work packages
        // 3. Remove project memberships
        // 4. Delete custom field values
        // 5. Remove from queries
        // 6. Clean up attachments
        // 7. Remove versions
        // 8. Delete wiki and wiki pages
        // 9. Remove forums and messages
        // 10. Delete news items
        // 11. Clean up time entries
        // 12. Remove repository associations
        // 13. Delete journals

        let mut service_result = ServiceResult::success(project.clone());

        // Handle notifications
        if self.send_notifications {
            // Would send deletion notifications here
        }

        service_result
    }

    /// Archive the project instead of deleting
    pub fn archive(self, mut project: ProjectEntity) -> ServiceResult<ProjectEntity> {
        // Ensure we're archiving an existing project
        if project.id.is_none() {
            return ServiceResult::failure_with_base_error("Cannot archive a new project");
        }

        // Validate permissions
        if let Err(errors) = self.validate(&project) {
            return ServiceResult::failure(errors);
        }

        // Set project as inactive (archived)
        project.active = false;

        // In a real implementation, this would:
        // 1. Update project in database
        // 2. Archive all subprojects
        // 3. Create journal entry

        let service_result = ServiceResult::success(project);

        // Handle notifications
        if self.send_notifications {
            // Would send archive notifications here
        }

        service_result
    }

    fn validate(&self, project: &ProjectEntity) -> Result<(), op_core::error::ValidationErrors> {
        let project_id = project.id.unwrap_or(0);
        let contract = DeleteProjectContract::new(self.user, project_id);

        // Create delete data - in real implementation, these would be fetched from database
        let delete_data = DeleteProjectData {
            id: project_id,
            has_children: false,  // Would be looked up from database
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

    fn create_existing_project() -> ProjectEntity {
        let mut project = ProjectEntity::new();
        project.id = Some(100);
        project.name = "Test Project".to_string();
        project.identifier = "test-project".to_string();
        project.active = true;
        project
    }

    #[test]
    fn test_delete_project_as_admin() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = DeleteProjectService::new(&user);

        let result = service.call(project);
        assert!(result.is_success());
    }

    #[test]
    fn test_cannot_delete_new_project() {
        let user = create_admin_user();
        let project = ProjectEntity::new(); // No ID = new
        let service = DeleteProjectService::new(&user);

        let result = service.call(project);
        assert!(result.is_failure());
    }

    #[test]
    fn test_delete_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let project = create_existing_project();
        let service = DeleteProjectService::new(&user);

        let result = service.call(project);
        assert!(result.is_failure());
    }

    #[test]
    fn test_archive_project_as_admin() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = DeleteProjectService::new(&user);

        let result = service.archive(project);
        assert!(result.is_success());

        let archived = result.result().unwrap();
        assert!(!archived.active);
    }

    #[test]
    fn test_cannot_archive_new_project() {
        let user = create_admin_user();
        let project = ProjectEntity::new();
        let service = DeleteProjectService::new(&user);

        let result = service.archive(project);
        assert!(result.is_failure());
    }

    #[test]
    fn test_archive_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let project = create_existing_project();
        let service = DeleteProjectService::new(&user);

        let result = service.archive(project);
        assert!(result.is_failure());
    }

    #[test]
    fn test_delete_without_notifications() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = DeleteProjectService::without_notifications(&user);

        let result = service.call(project);
        assert!(result.is_success());
    }
}
