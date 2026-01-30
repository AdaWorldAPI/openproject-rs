//! Member model
//!
//! Mirrors: app/models/member.rb
//! Table: members

use chrono::{DateTime, Utc};
use op_core::traits::{Entity, Id, Identifiable, Timestamped, ProjectScoped, HalRepresentable};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Member entity
///
/// Represents a user's membership in a project with associated roles.
///
/// # Ruby equivalent
/// ```ruby
/// class Member < ApplicationRecord
///   belongs_to :principal
///   belongs_to :project
///   has_many :member_roles
///   has_many :roles, through: :member_roles
/// end
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: Option<Id>,

    /// User or Group ID (principal)
    pub principal_id: Id,

    /// Project ID (None for global memberships)
    pub project_id: Option<Id>,

    /// Role IDs assigned through this membership
    #[serde(default)]
    pub role_ids: Vec<Id>,

    /// Optional notification message when member was added
    pub notification_message: Option<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for Member {
    fn default() -> Self {
        Self {
            id: None,
            principal_id: 0,
            project_id: None,
            role_ids: Vec::new(),
            notification_message: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl Identifiable for Member {
    fn id(&self) -> Option<Id> {
        self.id
    }
}

impl Timestamped for Member {
    fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

impl ProjectScoped for Member {
    fn project_id(&self) -> Option<Id> {
        self.project_id
    }
}

impl Entity for Member {
    const TABLE_NAME: &'static str = "members";
    const TYPE_NAME: &'static str = "Membership";
}

impl HalRepresentable for Member {
    fn hal_type(&self) -> &'static str {
        "Membership"
    }

    fn self_href(&self) -> String {
        format!("/api/v3/memberships/{}", self.id.unwrap_or(0))
    }

    fn hal_links(&self) -> serde_json::Value {
        let mut links = serde_json::json!({
            "self": { "href": self.self_href() },
            "principal": { "href": format!("/api/v3/principals/{}", self.principal_id) }
        });

        if let Some(project_id) = self.project_id {
            links["project"] = serde_json::json!({ "href": format!("/api/v3/projects/{}", project_id) });
        }

        links
    }
}

impl Member {
    /// Create a new member
    pub fn new(principal_id: Id, project_id: Option<Id>) -> Self {
        Self {
            principal_id,
            project_id,
            ..Default::default()
        }
    }

    /// Add a role to this membership
    pub fn add_role(&mut self, role_id: Id) {
        if !self.role_ids.contains(&role_id) {
            self.role_ids.push(role_id);
        }
    }

    /// Remove a role from this membership
    pub fn remove_role(&mut self, role_id: Id) {
        self.role_ids.retain(|&id| id != role_id);
    }

    /// Check if this is a global membership
    pub fn global(&self) -> bool {
        self.project_id.is_none()
    }
}

/// DTO for creating a member
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemberDto {
    pub principal_id: Id,
    pub project_id: Option<Id>,
    #[validate(length(min = 1))]
    pub role_ids: Vec<Id>,
    pub notification_message: Option<String>,
}

impl From<CreateMemberDto> for Member {
    fn from(dto: CreateMemberDto) -> Self {
        Self {
            principal_id: dto.principal_id,
            project_id: dto.project_id,
            role_ids: dto.role_ids,
            notification_message: dto.notification_message,
            ..Default::default()
        }
    }
}

/// DTO for updating a member
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberDto {
    pub role_ids: Option<Vec<Id>>,
    pub notification_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_new() {
        let m = Member::new(1, Some(2));
        assert_eq!(m.principal_id, 1);
        assert_eq!(m.project_id, Some(2));
        assert!(!m.global());
    }

    #[test]
    fn test_global_member() {
        let m = Member::new(1, None);
        assert!(m.global());
    }

    #[test]
    fn test_member_roles() {
        let mut m = Member::new(1, Some(2));
        m.add_role(1);
        m.add_role(2);
        assert_eq!(m.role_ids.len(), 2);

        m.add_role(1); // duplicate
        assert_eq!(m.role_ids.len(), 2);

        m.remove_role(1);
        assert_eq!(m.role_ids.len(), 1);
        assert_eq!(m.role_ids[0], 2);
    }
}
