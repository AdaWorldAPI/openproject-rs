//! Priority model (issue priority)
//!
//! Mirrors: app/models/issue_priority.rb (actually called IssuePriority in Rails)
//! Table: enumerations (with type = 'IssuePriority')

use chrono::{DateTime, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, HalRepresentable};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Work package priority entity
///
/// Priorities define urgency levels (Low, Normal, High, Immediate, etc.)
///
/// # Ruby equivalent
/// ```ruby
/// class IssuePriority < Enumeration
///   has_many :work_packages, foreign_key: 'priority_id'
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Priority {
    pub id: Option<Id>,

    /// Priority name (unique)
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Sort position (lower = higher priority typically)
    #[serde(default)]
    pub position: i32,

    /// Whether this is the default priority for new work packages
    #[serde(default)]
    pub is_default: bool,

    /// Whether this priority is active (can be selected)
    #[serde(default = "default_true")]
    pub active: bool,

    /// Color ID
    pub color_id: Option<Id>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

impl Default for Priority {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            position: 0,
            is_default: false,
            active: true,
            color_id: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Priority {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Priority {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl Entity for Priority {
    const TABLE_NAME: &'static str = "enumerations";
    const TYPE_NAME: &'static str = "Priority";
}

impl HalRepresentable for Priority {
    fn hal_type(&self) -> &'static str {
        "Priority"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/priorities/{}", self.id.unwrap_or(0))
    }
}

impl Priority {
    /// Create a new priority
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Standard priority names
    pub const LOW: &'static str = "Low";
    pub const NORMAL: &'static str = "Normal";
    pub const HIGH: &'static str = "High";
    pub const IMMEDIATE: &'static str = "Immediate";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_new() {
        let p = Priority::new("High");
        assert_eq!(p.name, "High");
        assert!(p.active);
    }
}
