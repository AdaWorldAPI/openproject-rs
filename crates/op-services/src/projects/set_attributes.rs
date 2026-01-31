//! Set Attributes Service for Projects
//!
//! Mirrors: app/services/projects/set_attributes_service.rb

use op_contracts::base::UserContext;
use op_contracts::projects::{CreateProjectContract, ProjectData};
use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::ProjectParams;

/// Project entity for service operations
#[derive(Debug, Clone, Default)]
pub struct ProjectEntity {
    pub id: Option<Id>,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub public: bool,
    pub active: bool,
    pub parent_id: Option<Id>,
}

impl ProjectEntity {
    pub fn new() -> Self {
        Self {
            id: None,
            name: String::new(),
            identifier: String::new(),
            description: None,
            public: false,
            active: true,
            parent_id: None,
        }
    }

    pub fn is_new(&self) -> bool {
        self.id.is_none()
    }
}

/// Implement ProjectData trait for ProjectEntity
impl ProjectData for ProjectEntity {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn public(&self) -> bool {
        self.public
    }

    fn active(&self) -> bool {
        self.active
    }

    fn parent_id(&self) -> Option<Id> {
        self.parent_id
    }
}

/// Service for setting attributes on a project
pub struct SetAttributesService<'a, U: UserContext> {
    user: &'a U,
    model: ProjectEntity,
}

impl<'a, U: UserContext> SetAttributesService<'a, U> {
    pub fn new(user: &'a U, model: ProjectEntity) -> Self {
        Self { user, model }
    }

    /// Set attributes from params and validate
    pub fn call(mut self, params: &ProjectParams) -> ServiceResult<ProjectEntity> {
        // Set attributes from params
        self.set_attributes(params);

        // Run contract validation
        let validation_result = self.validate();

        if let Err(errors) = validation_result {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(self.model)
    }

    fn set_attributes(&mut self, params: &ProjectParams) {
        if let Some(ref name) = params.name {
            self.model.name = name.clone();
        }
        if let Some(ref identifier) = params.identifier {
            self.model.identifier = identifier.clone();
        }
        if let Some(ref description) = params.description {
            self.model.description = Some(description.clone());
        }
        if let Some(public) = params.public {
            self.model.public = public;
        }
        if let Some(active) = params.active {
            self.model.active = active;
        }
        if let Some(parent_id) = params.parent_id {
            self.model.parent_id = Some(parent_id);
        }
    }

    fn validate(&self) -> Result<(), ValidationErrors> {
        use op_contracts::base::Contract;

        let contract = CreateProjectContract::new(self.user);
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
        let entity = ProjectEntity::new();
        let service = SetAttributesService::new(&user, entity);

        let params = ProjectParams::new()
            .with_name("Test Project")
            .with_identifier("test-project")
            .with_description("A test project");

        let result = service.call(&params);
        assert!(result.is_success());
        let project = result.result().unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.identifier, "test-project");
        assert_eq!(project.description, Some("A test project".to_string()));
    }

    #[test]
    fn test_set_attributes_validation_fails() {
        let user = create_admin_user();
        let entity = ProjectEntity::new();
        let service = SetAttributesService::new(&user, entity);

        // Empty name should fail validation
        let params = ProjectParams::new().with_identifier("test");

        let result = service.call(&params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("name"));
    }
}
