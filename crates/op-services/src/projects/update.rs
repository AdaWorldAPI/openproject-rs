//! Update Service for Projects
//!
//! Mirrors: app/services/projects/update_service.rb

use op_contracts::base::UserContext;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::set_attributes::{SetAttributesService, ProjectEntity};
use super::ProjectParams;

/// Service for updating projects
///
/// # Example
/// ```ignore
/// let service = UpdateProjectService::new(&user);
/// let params = ProjectParams::new()
///     .with_name("Updated Name");
/// let result = service.call(project, params);
/// ```
pub struct UpdateProjectService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> UpdateProjectService<'a, U> {
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
        project: ProjectEntity,
        params: ProjectParams,
    ) -> ServiceResult<ProjectEntity> {
        // Ensure we're updating an existing project
        if project.id.is_none() {
            return ServiceResult::failure_with_base_error("Cannot update a new project");
        }

        // Store original values for change detection
        let original_parent_id = project.parent_id;
        let original_active = project.active;
        let original_public = project.public;

        // Set attributes and validate
        let set_attrs_service = SetAttributesService::new(self.user, project);
        let result = set_attrs_service.call(&params);

        if result.is_failure() {
            return result;
        }

        let project = result.unwrap();

        // In a real implementation, this would:
        // 1. Persist changes to database
        // 2. Create journal entry
        // 3. Handle parent change (update nested set)
        // 4. Handle visibility change
        // 5. Handle archive/unarchive

        let service_result = ServiceResult::success(project.clone());

        // Handle parent change (would update hierarchy in nested set)
        if project.parent_id != original_parent_id {
            // Would handle parent change here (move in hierarchy)
            // This involves updating lft/rgt values for nested set
        }

        // Handle active status change
        if project.active != original_active {
            if !project.active {
                // Would archive subprojects here
            } else {
                // Would check if parent is active before unarchiving
            }
        }

        // Handle public/private change
        if project.public != original_public {
            // Would update visibility-related permissions
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

    fn create_existing_project() -> ProjectEntity {
        let mut project = ProjectEntity::new();
        project.id = Some(100);
        project.name = "Original Name".to_string();
        project.identifier = "original-identifier".to_string();
        project
    }

    #[test]
    fn test_update_project_name() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = UpdateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("Updated Name");

        let result = service.call(project, params);
        assert!(result.is_success());

        let updated = result.result().unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.identifier, "original-identifier");
    }

    #[test]
    fn test_update_project_multiple_fields() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = UpdateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("New Name")
            .with_description("New description")
            .with_public(true);

        let result = service.call(project, params);
        assert!(result.is_success());

        let updated = result.result().unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.description, Some("New description".to_string()));
        assert!(updated.public);
    }

    #[test]
    fn test_cannot_update_new_project() {
        let user = create_admin_user();
        let project = ProjectEntity::new(); // No ID = new
        let service = UpdateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("Should Fail");

        let result = service.call(project, params);
        assert!(result.is_failure());
    }

    #[test]
    fn test_update_without_permission() {
        let user = MockUser {
            id: 2,
            admin: false,
            project_permissions: std::collections::HashMap::new(),
        };
        let project = create_existing_project();
        let service = UpdateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("Unauthorized Update");

        let result = service.call(project, params);
        assert!(result.is_failure());
    }

    #[test]
    fn test_update_project_archive() {
        let user = create_admin_user();
        let mut project = create_existing_project();
        project.active = true;
        let service = UpdateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_active(false);

        let result = service.call(project, params);
        assert!(result.is_success());

        let updated = result.result().unwrap();
        assert!(!updated.active);
    }

    #[test]
    fn test_update_project_change_parent() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = UpdateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_parent_id(50);

        let result = service.call(project, params);
        assert!(result.is_success());

        let updated = result.result().unwrap();
        assert_eq!(updated.parent_id, Some(50));
    }

    #[test]
    fn test_update_without_notifications() {
        let user = create_admin_user();
        let project = create_existing_project();
        let service = UpdateProjectService::without_notifications(&user);

        let params = ProjectParams::new()
            .with_name("Silent Update");

        let result = service.call(project, params);
        assert!(result.is_success());
    }
}
