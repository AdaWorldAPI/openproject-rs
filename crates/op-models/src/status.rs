//! Status model
//!
//! Mirrors: app/models/status.rb
//! Table: statuses

use chrono::{DateTime, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, HalRepresentable};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Work package status entity
///
/// Statuses represent the lifecycle state of work packages (New, In Progress, Closed, etc.)
///
/// # Ruby equivalent
/// ```ruby
/// class Status < ApplicationRecord
///   has_many :work_packages
///   has_many :workflows
///   validates :name, presence: true, uniqueness: true
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub id: Option<Id>,

    /// Status name (unique)
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Whether this status means "closed/done"
    #[serde(default)]
    pub is_closed: bool,

    /// Whether this status is the default for new work packages
    #[serde(default)]
    pub is_default: bool,

    /// Whether this status is read-only (prevents further edits)
    #[serde(default)]
    pub is_readonly: bool,

    /// Sort position
    #[serde(default)]
    pub position: i32,

    /// Default done ratio for this status (0-100)
    pub default_done_ratio: Option<i32>,

    /// Color as hex code (e.g., "#FF0000")
    pub color_id: Option<Id>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            is_closed: false,
            is_default: false,
            is_readonly: false,
            position: 0,
            default_done_ratio: None,
            color_id: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Status {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Status {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl Entity for Status {
    const TABLE_NAME: &'static str = "statuses";
    const TYPE_NAME: &'static str = "Status";
}

impl HalRepresentable for Status {
    fn hal_type(&self) -> &'static str {
        "Status"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/statuses/{}", self.id.unwrap_or(0))
    }
}

impl Status {
    /// Create a new status
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Common status names
    pub const NEW: &'static str = "New";
    pub const IN_PROGRESS: &'static str = "In progress";
    pub const CLOSED: &'static str = "Closed";
    pub const REJECTED: &'static str = "Rejected";
    pub const ON_HOLD: &'static str = "On hold";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_new() {
        let status = Status::new("In Progress");
        assert_eq!(status.name, "In Progress");
        assert!(!status.is_closed);
    }

    #[test]
    fn test_status_closed() {
        let mut status = Status::new("Done");
        status.is_closed = true;
        assert!(status.is_closed);
    }
}
