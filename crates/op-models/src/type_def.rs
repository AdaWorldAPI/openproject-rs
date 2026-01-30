//! Type model (work package type)
//!
//! Mirrors: app/models/type.rb
//! Table: types
//!
//! Note: Named `type_def` because `type` is a Rust reserved keyword

use chrono::{DateTime, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, HalRepresentable};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Work package type entity
///
/// Types categorize work packages (Task, Bug, Feature, Epic, etc.)
///
/// # Ruby equivalent
/// ```ruby
/// class Type < ApplicationRecord
///   has_many :work_packages
///   has_and_belongs_to_many :projects
///   has_many :workflows
///   validates :name, presence: true, uniqueness: true
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    pub id: Option<Id>,

    /// Type name (unique)
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Sort position
    #[serde(default)]
    pub position: i32,

    /// Whether this is the default type for new work packages
    #[serde(default)]
    pub is_default: bool,

    /// Whether this type is shown in the roadmap
    #[serde(default = "default_true")]
    pub is_in_roadmap: bool,

    /// Whether this is a milestone type (no duration, just a date)
    #[serde(default)]
    pub is_milestone: bool,

    /// Color ID
    pub color_id: Option<Id>,

    /// Description
    pub description: Option<String>,

    /// Attribute groups configuration (JSON)
    /// Defines which attributes are shown in which groups for this type
    pub attribute_groups: Option<serde_json::Value>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

impl Default for Type {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            position: 0,
            is_default: false,
            is_in_roadmap: true,
            is_milestone: false,
            color_id: None,
            description: None,
            attribute_groups: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Type {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Type {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl Entity for Type {
    const TABLE_NAME: &'static str = "types";
    const TYPE_NAME: &'static str = "Type";
}

impl HalRepresentable for Type {
    fn hal_type(&self) -> &'static str {
        "Type"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/types/{}", self.id.unwrap_or(0))
    }
}

impl Type {
    /// Create a new type
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Standard type names used in OpenProject
    pub const TASK: &'static str = "Task";
    pub const MILESTONE: &'static str = "Milestone";
    pub const PHASE: &'static str = "Phase";
    pub const FEATURE: &'static str = "Feature";
    pub const EPIC: &'static str = "Epic";
    pub const USER_STORY: &'static str = "User story";
    pub const BUG: &'static str = "Bug";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_new() {
        let t = Type::new("Bug");
        assert_eq!(t.name, "Bug");
        assert!(!t.is_milestone);
    }

    #[test]
    fn test_milestone_type() {
        let mut t = Type::new("Milestone");
        t.is_milestone = true;
        assert!(t.is_milestone);
    }
}
