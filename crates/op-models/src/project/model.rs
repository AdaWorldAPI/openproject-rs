//! Project model
//!
//! Mirrors: app/models/project.rb
//! Table: projects

use chrono::{DateTime, NaiveDate, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, HalRepresentable};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Project status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatusCode {
    #[default]
    OnTrack,
    AtRisk,
    OffTrack,
    NotSet,
}

impl ProjectStatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnTrack => "on_track",
            Self::AtRisk => "at_risk",
            Self::OffTrack => "off_track",
            Self::NotSet => "not_set",
        }
    }
}

/// Project entity
///
/// Represents a project in OpenProject.
///
/// # Ruby equivalent
/// ```ruby
/// class Project < ApplicationRecord
///   acts_as_customizable
///   acts_as_tree
///   has_many :members
///   has_many :work_packages
///   has_many :versions
///   # ...
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Option<Id>,

    /// Unique identifier (URL-safe slug)
    #[validate(length(min = 1, max = 100))]
    pub identifier: String,

    /// Display name
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Description (can be markdown/textile)
    pub description: Option<String>,

    /// Description format (plain, markdown)
    #[serde(default = "default_description_format")]
    pub description_format: String,

    /// Whether the project is public (visible to non-members)
    #[serde(default)]
    pub public: bool,

    /// Parent project ID (for hierarchy)
    pub parent_id: Option<Id>,

    /// Left boundary for nested set
    #[serde(skip)]
    pub lft: Option<i32>,

    /// Right boundary for nested set
    #[serde(skip)]
    pub rgt: Option<i32>,

    /// Project status code
    #[serde(default)]
    pub status_code: ProjectStatusCode,

    /// Status explanation text
    pub status_explanation: Option<String>,

    /// Whether the project is active (not archived)
    #[serde(default = "default_true")]
    pub active: bool,

    /// Whether project is a template
    #[serde(default)]
    pub templated: bool,

    /// Project created by user ID
    pub created_by_id: Option<Id>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_description_format() -> String {
    "markdown".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: None,
            identifier: String::new(),
            name: String::new(),
            description: None,
            description_format: "markdown".to_string(),
            public: false,
            parent_id: None,
            lft: None,
            rgt: None,
            status_code: ProjectStatusCode::NotSet,
            status_explanation: None,
            active: true,
            templated: false,
            created_by_id: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Project {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Project {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl Entity for Project {
    const TABLE_NAME: &'static str = "projects";
    const TYPE_NAME: &'static str = "Project";
}

impl HalRepresentable for Project {
    fn hal_type(&self) -> &'static str {
        "Project"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/projects/{}", self.id.unwrap_or(0))
    }

    fn hal_links(&self) -> serde_json::Value {
        let mut links = serde_json::json!({
            "self": { "href": self.self_href() },
            "workPackages": { "href": format!("/api/v3/projects/{}/work_packages", self.id.unwrap_or(0)) },
            "categories": { "href": format!("/api/v3/projects/{}/categories", self.id.unwrap_or(0)) },
            "versions": { "href": format!("/api/v3/projects/{}/versions", self.id.unwrap_or(0)) },
            "memberships": { "href": format!("/api/v3/memberships?filters=[{{\"project\":{{\"operator\":\"=\",\"values\":[\"{}\"]}}}}]", self.id.unwrap_or(0)) },
            "types": { "href": format!("/api/v3/projects/{}/types", self.id.unwrap_or(0)) }
        });

        if let Some(parent_id) = self.parent_id {
            links["parent"] = serde_json::json!({ "href": format!("/api/v3/projects/{}", parent_id) });
        }

        links
    }
}

impl Project {
    /// Create a new project with minimal required fields
    pub fn new(identifier: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Check if project is archived
    pub fn archived(&self) -> bool {
        !self.active
    }

    /// Check if this is a root project (no parent)
    pub fn root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Generate a valid identifier from a name
    pub fn identifier_from_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
}

/// DTO for creating a new project
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectDto {
    #[validate(length(min = 1, max = 100))]
    pub identifier: Option<String>,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub description: Option<String>,
    pub public: Option<bool>,
    pub parent_id: Option<Id>,
    pub status_code: Option<ProjectStatusCode>,
    pub status_explanation: Option<String>,
    pub templated: Option<bool>,
}

impl From<CreateProjectDto> for Project {
    fn from(dto: CreateProjectDto) -> Self {
        let identifier = dto.identifier
            .unwrap_or_else(|| Project::identifier_from_name(&dto.name));

        Self {
            identifier,
            name: dto.name,
            description: dto.description,
            public: dto.public.unwrap_or(false),
            parent_id: dto.parent_id,
            status_code: dto.status_code.unwrap_or_default(),
            status_explanation: dto.status_explanation,
            templated: dto.templated.unwrap_or(false),
            ..Default::default()
        }
    }
}

/// DTO for updating a project
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
    pub parent_id: Option<Id>,
    pub status_code: Option<ProjectStatusCode>,
    pub status_explanation: Option<String>,
    pub active: Option<bool>,
}

impl UpdateProjectDto {
    /// Apply updates to a project
    pub fn apply_to(&self, project: &mut Project) {
        if let Some(ref name) = self.name {
            project.name = name.clone();
        }
        if let Some(ref description) = self.description {
            project.description = Some(description.clone());
        }
        if let Some(public) = self.public {
            project.public = public;
        }
        if let Some(parent_id) = self.parent_id {
            project.parent_id = Some(parent_id);
        }
        if let Some(status_code) = self.status_code {
            project.status_code = status_code;
        }
        if let Some(ref explanation) = self.status_explanation {
            project.status_explanation = Some(explanation.clone());
        }
        if let Some(active) = self.active {
            project.active = active;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("my-project", "My Project");
        assert_eq!(project.identifier, "my-project");
        assert_eq!(project.name, "My Project");
        assert!(project.active);
        assert!(!project.public);
    }

    #[test]
    fn test_identifier_from_name() {
        assert_eq!(Project::identifier_from_name("My Project"), "my-project");
        assert_eq!(Project::identifier_from_name("Test 123"), "test-123");
        assert_eq!(Project::identifier_from_name("  Spaces  "), "spaces");
    }

    #[test]
    fn test_project_hierarchy() {
        let mut child = Project::new("child", "Child Project");
        child.parent_id = Some(1);

        assert!(!child.root());

        let root = Project::new("root", "Root Project");
        assert!(root.root());
    }
}
