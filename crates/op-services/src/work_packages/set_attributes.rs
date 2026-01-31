//! Set Attributes Service for Work Packages
//!
//! Mirrors: app/services/work_packages/set_attributes_service.rb

use op_contracts::base::UserContext;
use op_contracts::work_packages::{CreateWorkPackageContract, WorkPackageData};
use op_core::error::ValidationErrors;
use op_core::traits::Id;

use crate::result::ServiceResult;
use super::WorkPackageParams;

/// Work package entity for service operations
#[derive(Debug, Clone, Default)]
pub struct WorkPackageEntity {
    pub id: Option<Id>,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: Id,
    pub type_id: Id,
    pub status_id: Id,
    pub priority_id: Id,
    pub author_id: Id,
    pub assigned_to_id: Option<Id>,
    pub responsible_id: Option<Id>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: i32,
    pub parent_id: Option<Id>,
    pub version_id: Option<Id>,
    pub category_id: Option<Id>,
    pub lock_version: i32,
}

impl WorkPackageEntity {
    pub fn new(project_id: Id, type_id: Id, author_id: Id) -> Self {
        Self {
            id: None,
            subject: String::new(),
            description: None,
            project_id,
            type_id,
            status_id: 1, // Default status
            priority_id: 2, // Normal priority
            author_id,
            assigned_to_id: None,
            responsible_id: None,
            start_date: None,
            due_date: None,
            estimated_hours: None,
            done_ratio: 0,
            parent_id: None,
            version_id: None,
            category_id: None,
            lock_version: 0,
        }
    }

    pub fn is_new(&self) -> bool {
        self.id.is_none()
    }
}

/// Implement WorkPackageData trait for WorkPackageEntity
impl WorkPackageData for WorkPackageEntity {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn subject(&self) -> &str {
        &self.subject
    }

    fn project_id(&self) -> Id {
        self.project_id
    }

    fn type_id(&self) -> Id {
        self.type_id
    }

    fn status_id(&self) -> Id {
        self.status_id
    }

    fn author_id(&self) -> Id {
        self.author_id
    }

    fn assigned_to_id(&self) -> Option<Id> {
        self.assigned_to_id
    }

    fn priority_id(&self) -> Option<Id> {
        Some(self.priority_id)
    }

    fn version_id(&self) -> Option<Id> {
        self.version_id
    }

    fn parent_id(&self) -> Option<Id> {
        self.parent_id
    }

    fn done_ratio(&self) -> i32 {
        self.done_ratio
    }

    fn estimated_hours(&self) -> Option<f64> {
        self.estimated_hours
    }

    fn lock_version(&self) -> i32 {
        self.lock_version
    }
}

/// Service for setting attributes on a work package
pub struct SetAttributesService<'a, U: UserContext> {
    user: &'a U,
    model: WorkPackageEntity,
}

impl<'a, U: UserContext> SetAttributesService<'a, U> {
    pub fn new(user: &'a U, model: WorkPackageEntity) -> Self {
        Self { user, model }
    }

    /// Set attributes from params and validate
    pub fn call(mut self, params: &WorkPackageParams) -> ServiceResult<WorkPackageEntity> {
        // Set attributes from params
        self.set_attributes(params);

        // Run contract validation
        let validation_result = self.validate_create();

        if let Err(errors) = validation_result {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(self.model)
    }

    fn set_attributes(&mut self, params: &WorkPackageParams) {
        if let Some(ref subject) = params.subject {
            self.model.subject = subject.clone();
        }
        if let Some(ref description) = params.description {
            self.model.description = Some(description.clone());
        }
        if let Some(project_id) = params.project_id {
            self.model.project_id = project_id;
        }
        if let Some(type_id) = params.type_id {
            self.model.type_id = type_id;
        }
        if let Some(status_id) = params.status_id {
            self.model.status_id = status_id;
        }
        if let Some(priority_id) = params.priority_id {
            self.model.priority_id = priority_id;
        }
        if let Some(assigned_to_id) = params.assigned_to_id {
            self.model.assigned_to_id = Some(assigned_to_id);
        }
        if let Some(responsible_id) = params.responsible_id {
            self.model.responsible_id = Some(responsible_id);
        }
        if let Some(start_date) = params.start_date {
            self.model.start_date = Some(start_date);
        }
        if let Some(due_date) = params.due_date {
            self.model.due_date = Some(due_date);
        }
        if let Some(estimated_hours) = params.estimated_hours {
            self.model.estimated_hours = Some(estimated_hours);
        }
        if let Some(done_ratio) = params.done_ratio {
            self.model.done_ratio = done_ratio;
        }
        if let Some(parent_id) = params.parent_id {
            self.model.parent_id = Some(parent_id);
        }
        if let Some(version_id) = params.version_id {
            self.model.version_id = Some(version_id);
        }
        if let Some(category_id) = params.category_id {
            self.model.category_id = Some(category_id);
        }
    }

    fn validate_create(&self) -> Result<(), ValidationErrors> {
        use op_contracts::base::Contract;

        let contract = CreateWorkPackageContract::new(self.user, self.model.project_id);
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

    #[test]
    fn test_set_attributes_basic() {
        let user = create_admin_user();
        let entity = WorkPackageEntity::new(1, 1, user.id);
        let service = SetAttributesService::new(&user, entity);

        let params = WorkPackageParams::new()
            .with_subject("Test Work Package")
            .with_description("A test description");

        let result = service.call(&params);
        assert!(result.is_success());
        let wp = result.result().unwrap();
        assert_eq!(wp.subject, "Test Work Package");
        assert_eq!(wp.description, Some("A test description".to_string()));
    }

    #[test]
    fn test_set_attributes_validation_fails() {
        let user = create_admin_user();
        let entity = WorkPackageEntity::new(1, 1, user.id);
        let service = SetAttributesService::new(&user, entity);

        // Empty subject should fail validation
        let params = WorkPackageParams::new();

        let result = service.call(&params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("subject"));
    }
}
