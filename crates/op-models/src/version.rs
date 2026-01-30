//! Version model
//!
//! Mirrors: app/models/version.rb
//! Table: versions

use chrono::{DateTime, NaiveDate, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, ProjectScoped, HalRepresentable};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Version sharing options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VersionSharing {
    /// Version is only visible in its own project
    #[default]
    None,
    /// Version is visible in all subprojects
    Descendants,
    /// Version is visible in parent projects and siblings
    Hierarchy,
    /// Version is visible in all projects within the tree
    Tree,
    /// Version is visible in all projects
    System,
}

/// Version status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VersionStatus {
    #[default]
    Open,
    Locked,
    Closed,
}

/// Version entity
///
/// Versions represent releases/milestones that work packages can target.
///
/// # Ruby equivalent
/// ```ruby
/// class Version < ApplicationRecord
///   belongs_to :project
///   has_many :work_packages
///   validates :name, presence: true
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: Option<Id>,

    /// Version name
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Owning project
    pub project_id: Id,

    /// Target completion date
    pub effective_date: Option<NaiveDate>,

    /// Start date
    pub start_date: Option<NaiveDate>,

    /// Version status
    #[serde(default)]
    pub status: VersionStatus,

    /// Sharing scope
    #[serde(default)]
    pub sharing: VersionSharing,

    /// Wiki page name for release notes
    pub wiki_page_title: Option<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            description: None,
            project_id: 0,
            effective_date: None,
            start_date: None,
            status: VersionStatus::Open,
            sharing: VersionSharing::None,
            wiki_page_title: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Version {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Version {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl ProjectScoped for Version {
    fn project_id(&self) -> Option<Id> {
        Some(self.project_id)
    }
}

impl Entity for Version {
    const TABLE_NAME: &'static str = "versions";
    const TYPE_NAME: &'static str = "Version";
}

impl HalRepresentable for Version {
    fn hal_type(&self) -> &'static str {
        "Version"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/versions/{}", self.id.unwrap_or(0))
    }

    fn hal_links(&self) -> serde_json::Value {
        serde_json::json!({
            "self": { "href": self.self_href() },
            "definingProject": { "href": format!("/api/v3/projects/{}", self.project_id) }
        })
    }
}

impl Version {
    /// Create a new version
    pub fn new(name: impl Into<String>, project_id: Id) -> Self {
        Self {
            name: name.into(),
            project_id,
            ..Default::default()
        }
    }

    /// Check if version is open
    pub fn open(&self) -> bool {
        matches!(self.status, VersionStatus::Open)
    }

    /// Check if version is closed
    pub fn closed(&self) -> bool {
        matches!(self.status, VersionStatus::Closed)
    }

    /// Check if version is overdue
    pub fn overdue(&self) -> bool {
        if let Some(date) = self.effective_date {
            let today = chrono::Utc::now().date_naive();
            date < today && self.open()
        } else {
            false
        }
    }
}

/// DTO for creating a version
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionDto {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub status: Option<VersionStatus>,
    pub sharing: Option<VersionSharing>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_new() {
        let v = Version::new("v1.0.0", 1);
        assert_eq!(v.name, "v1.0.0");
        assert_eq!(v.project_id, 1);
        assert!(v.open());
    }

    #[test]
    fn test_version_status() {
        let mut v = Version::new("v1.0.0", 1);
        assert!(v.open());
        assert!(!v.closed());

        v.status = VersionStatus::Closed;
        assert!(!v.open());
        assert!(v.closed());
    }
}
