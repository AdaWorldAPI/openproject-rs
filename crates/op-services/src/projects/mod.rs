//! Project services
//!
//! Mirrors:
//! - app/services/projects/create_service.rb
//! - app/services/projects/update_service.rb
//! - app/services/projects/delete_service.rb
//! - app/services/projects/set_attributes_service.rb

mod create;
mod update;
mod delete;
mod set_attributes;

pub use create::CreateProjectService;
pub use update::UpdateProjectService;
pub use delete::DeleteProjectService;
pub use set_attributes::{ProjectEntity, SetAttributesService};

/// Project service params
#[derive(Debug, Clone, Default)]
pub struct ProjectParams {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
    pub active: Option<bool>,
    pub parent_id: Option<i64>,
    pub send_notifications: bool,
}

impl ProjectParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_public(mut self, public: bool) -> Self {
        self.public = Some(public);
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
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
