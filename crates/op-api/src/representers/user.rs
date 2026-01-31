//! User HAL Representer
//!
//! Converts user models to HAL+JSON format compatible with OpenProject API v3.

use chrono::{DateTime, Utc};
use op_core::traits::Id;
use serde::Serialize;

use super::hal::{HalCollection, HalLink, HalLinks, HalResource, rels};

/// User representation for API responses
#[derive(Debug, Clone, Serialize)]
pub struct UserRepresentation {
    pub id: Id,
    pub login: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub name: String,
    pub email: Option<String>,
    pub admin: bool,
    pub avatar: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// User representer
pub struct UserRepresenter;

impl UserRepresenter {
    /// Create a HAL resource for a single user
    pub fn represent(user: UserData, can_view_email: bool) -> HalResource<UserRepresentation> {
        let name = format!("{} {}", user.first_name, user.last_name);
        let avatar = Self::gravatar_url(&user.email, 64);

        let rep = UserRepresentation {
            id: user.id,
            login: user.login.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            name,
            email: if can_view_email { Some(user.email.clone()) } else { None },
            admin: user.admin,
            avatar,
            status: Self::status_name(user.status),
            language: user.language.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

        let links = Self::build_links(&user, can_view_email);

        HalResource::new("User", rep).with_links(links)
    }

    /// Create a HAL resource for the current user (self)
    pub fn represent_me(user: UserData) -> HalResource<UserRepresentation> {
        let mut hal = Self::represent(user, true);
        hal.resource_type = "User".to_string();
        hal
    }

    /// Create a HAL collection of users
    pub fn represent_collection(
        users: Vec<UserData>,
        total: i64,
        offset: i64,
        page_size: i64,
        base_url: &str,
        can_view_emails: bool,
    ) -> HalCollection<HalResource<UserRepresentation>> {
        let page = (offset / page_size) + 1;
        let elements: Vec<HalResource<UserRepresentation>> = users
            .into_iter()
            .map(|u| Self::represent(u, can_view_emails))
            .collect();

        HalCollection::new("UserCollection", elements, total, page_size, offset)
            .with_pagination_links(base_url, page, page_size)
    }

    /// Build links for a user
    fn build_links(user: &UserData, can_manage: bool) -> HalLinks {
        let base = format!("/api/v3/users/{}", user.id);

        let mut links = HalLinks::new()
            .with(rels::SELF, HalLink::new(&base))
            .with("showUser", HalLink::new(format!("/users/{}", user.id)))
            .with("memberships", HalLink::new(format!(
                "/api/v3/memberships?filters=[{{\"principal\":{{\"operator\":\"=\",\"values\":[\"{}\"]}}}}]",
                user.id
            )));

        if can_manage {
            links.add(rels::UPDATE, HalLink::new(format!("{}/form", base)).method("POST"));
            links.add(rels::UPDATE_IMMEDIATELY, HalLink::new(&base).method("PATCH"));
            links.add(rels::DELETE, HalLink::new(&base).method("DELETE"));
            links.add("lock", HalLink::new(format!("{}/lock", base)).method("POST"));
            links.add("unlock", HalLink::new(format!("{}/unlock", base)).method("DELETE"));
        }

        links
    }

    /// Convert status code to name
    fn status_name(status: i32) -> String {
        match status {
            0 => "builtin".to_string(),
            1 => "active".to_string(),
            2 => "registered".to_string(),
            3 => "locked".to_string(),
            4 => "invited".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Generate gravatar URL from email
    fn gravatar_url(email: &str, size: u32) -> String {
        let hash = md5_hash(email.trim().to_lowercase().as_bytes());
        format!(
            "https://secure.gravatar.com/avatar/{}?default=404&secure=true&size={}",
            hash, size
        )
    }
}

/// User data for representation
#[derive(Debug, Clone)]
pub struct UserData {
    pub id: Id,
    pub login: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub admin: bool,
    pub status: i32,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simple MD5 hash (for gravatar URLs)
fn md5_hash(data: &[u8]) -> String {
    use std::fmt::Write;

    // Simple MD5 implementation for gravatar URLs
    // In production, use the md5 crate
    let digest = md5::compute(data);
    let mut result = String::with_capacity(32);
    for byte in digest.iter() {
        write!(result, "{:02x}", byte).unwrap();
    }
    result
}

/// Placeholder user representation
#[derive(Debug, Clone, Serialize)]
pub struct PlaceholderUserRepresentation {
    #[serde(rename = "_type")]
    pub user_type: String,
    pub name: String,
}

/// System user representation
#[derive(Debug, Clone, Serialize)]
pub struct SystemUserRepresentation {
    #[serde(rename = "_type")]
    pub user_type: String,
    pub id: &'static str,
    pub name: String,
}

impl SystemUserRepresentation {
    /// Create the "System" user representation
    pub fn system() -> Self {
        Self {
            user_type: "User".to_string(),
            id: "system",
            name: "System".to_string(),
        }
    }

    /// Create the "Deleted user" representation
    pub fn deleted() -> Self {
        Self {
            user_type: "User".to_string(),
            id: "deleted_user",
            name: "Deleted user".to_string(),
        }
    }

    /// Create an anonymous user representation
    pub fn anonymous() -> Self {
        Self {
            user_type: "User".to_string(),
            id: "anonymous",
            name: "Anonymous".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> UserData {
        UserData {
            id: 1,
            login: "john.doe".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            admin: false,
            status: 1,
            language: Some("en".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_user_representation() {
        let user = create_test_user();
        let hal = UserRepresenter::represent(user, true);

        let json = serde_json::to_value(&hal).unwrap();
        assert_eq!(json["_type"], "User");
        assert_eq!(json["id"], 1);
        assert_eq!(json["login"], "john.doe");
        assert_eq!(json["firstName"], "John");
        assert_eq!(json["lastName"], "Doe");
        assert_eq!(json["name"], "John Doe");
    }

    #[test]
    fn test_user_without_email_permission() {
        let user = create_test_user();
        let hal = UserRepresenter::represent(user, false);

        let json = serde_json::to_value(&hal).unwrap();
        assert!(json["email"].is_null());
    }

    #[test]
    fn test_status_name() {
        assert_eq!(UserRepresenter::status_name(1), "active");
        assert_eq!(UserRepresenter::status_name(3), "locked");
        assert_eq!(UserRepresenter::status_name(2), "registered");
    }

    #[test]
    fn test_system_user() {
        let system = SystemUserRepresentation::system();
        let json = serde_json::to_value(&system).unwrap();
        assert_eq!(json["_type"], "User");
        assert_eq!(json["id"], "system");
    }
}
