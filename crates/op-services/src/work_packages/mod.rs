//! Work Package services
//!
//! Mirrors:
//! - app/services/work_packages/create_service.rb
//! - app/services/work_packages/update_service.rb
//! - app/services/work_packages/delete_service.rb
//! - app/services/work_packages/set_attributes_service.rb

mod create;
mod update;
mod delete;
mod set_attributes;

pub use create::CreateWorkPackageService;
pub use update::UpdateWorkPackageService;
pub use delete::DeleteWorkPackageService;
pub use set_attributes::SetAttributesService;

/// Work package service params
#[derive(Debug, Clone, Default)]
pub struct WorkPackageParams {
    pub subject: Option<String>,
    pub description: Option<String>,
    pub project_id: Option<i64>,
    pub type_id: Option<i64>,
    pub status_id: Option<i64>,
    pub priority_id: Option<i64>,
    pub assigned_to_id: Option<i64>,
    pub responsible_id: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: Option<i32>,
    pub parent_id: Option<i64>,
    pub version_id: Option<i64>,
    pub category_id: Option<i64>,
    pub send_notifications: bool,
}

impl WorkPackageParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn with_project_id(mut self, project_id: i64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_type_id(mut self, type_id: i64) -> Self {
        self.type_id = Some(type_id);
        self
    }

    pub fn with_status_id(mut self, status_id: i64) -> Self {
        self.status_id = Some(status_id);
        self
    }

    pub fn with_priority_id(mut self, priority_id: i64) -> Self {
        self.priority_id = Some(priority_id);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_assigned_to_id(mut self, assigned_to_id: i64) -> Self {
        self.assigned_to_id = Some(assigned_to_id);
        self
    }

    pub fn with_dates(mut self, start_date: chrono::NaiveDate, due_date: chrono::NaiveDate) -> Self {
        self.start_date = Some(start_date);
        self.due_date = Some(due_date);
        self
    }

    pub fn with_estimated_hours(mut self, hours: f64) -> Self {
        self.estimated_hours = Some(hours);
        self
    }

    pub fn with_done_ratio(mut self, ratio: i32) -> Self {
        self.done_ratio = Some(ratio);
        self
    }

    pub fn with_parent_id(mut self, parent_id: i64) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn send_notifications(mut self, send: bool) -> Self {
        self.send_notifications = send;
        self
    }
}
