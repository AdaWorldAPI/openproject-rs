//! Create Service for Projects
//!
//! Mirrors: app/services/projects/create_service.rb

use op_contracts::base::UserContext;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::set_attributes::{SetAttributesService, ProjectEntity};
use super::ProjectParams;

/// Service for creating projects
///
/// # Example
/// ```ignore
/// let service = CreateProjectService::new(&user);
/// let params = ProjectParams::new()
///     .with_name("My Project")
///     .with_identifier("my-project");
/// let result = service.call(params);
/// ```
pub struct CreateProjectService<'a, U: UserContext> {
    user: &'a U,
    send_notifications: bool,
}

impl<'a, U: UserContext> CreateProjectService<'a, U> {
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
    pub fn call(self, params: ProjectParams) -> ServiceResult<ProjectEntity> {
        // Create new project with defaults
        let project = ProjectEntity::new();

        // Set attributes and validate
        let set_attrs_service = SetAttributesService::new(self.user, project);
        let result = set_attrs_service.call(&params);

        if result.is_failure() {
            return result;
        }

        let mut project = result.unwrap();

        // In a real implementation, this would persist to database
        // For now, we simulate assigning an ID
        project.id = Some(self.next_id());

        // Create result
        let mut service_result = ServiceResult::success(project.clone());

        // In a real implementation, this would:
        // 1. Persist project to database
        // 2. Create default project settings
        // 3. Set up default types and statuses
        // 4. Create initial custom fields
        // 5. Set up default members (add creator as admin)

        // Handle notifications
        if self.send_notifications && params.send_notifications {
            // Would send project creation notifications here
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

    fn create_user_with_permission() -> MockUser {
        let mut permissions = HashSet::new();
        permissions.insert("add_project".to_string());
        MockUser {
            id: 2,
            admin: false,
            global_permissions: permissions,
        }
    }

    #[test]
    fn test_create_project_as_admin() {
        let user = create_admin_user();
        let service = CreateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("Test Project")
            .with_identifier("test-project");

        let result = service.call(params);
        assert!(result.is_success());

        let project = result.result().unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.identifier, "test-project");
        assert!(project.id.is_some());
    }

    #[test]
    fn test_create_project_with_permission() {
        let user = create_user_with_permission();
        let service = CreateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("User Project")
            .with_identifier("user-project");

        let result = service.call(params);
        assert!(result.is_success());
    }

    #[test]
    fn test_create_project_without_permission() {
        let user = MockUser {
            id: 3,
            admin: false,
            global_permissions: HashSet::new(),
        };
        let service = CreateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("Unauthorized Project")
            .with_identifier("unauthorized");

        let result = service.call(params);
        assert!(result.is_failure());
    }

    #[test]
    fn test_create_project_validation_failure() {
        let user = create_admin_user();
        let service = CreateProjectService::new(&user);

        // Missing name should fail validation
        let params = ProjectParams::new()
            .with_identifier("no-name");

        let result = service.call(params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("name"));
    }

    #[test]
    fn test_create_project_with_all_fields() {
        let user = create_admin_user();
        let service = CreateProjectService::new(&user);

        let params = ProjectParams::new()
            .with_name("Full Project")
            .with_identifier("full-project")
            .with_description("A project with all fields set")
            .with_public(true)
            .with_active(true)
            .with_parent_id(1);

        let result = service.call(params);
        assert!(result.is_success());

        let project = result.result().unwrap();
        assert_eq!(project.name, "Full Project");
        assert_eq!(project.identifier, "full-project");
        assert_eq!(project.description, Some("A project with all fields set".to_string()));
        assert!(project.public);
        assert!(project.active);
        assert_eq!(project.parent_id, Some(1));
    }

    #[test]
    fn test_create_without_notifications() {
        let user = create_admin_user();
        let service = CreateProjectService::without_notifications(&user);

        let params = ProjectParams::new()
            .with_name("Silent Project")
            .with_identifier("silent-project");

        let result = service.call(params);
        assert!(result.is_success());
    }
}
