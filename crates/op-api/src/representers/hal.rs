//! HAL+JSON Representers
//!
//! Implements Hypertext Application Language (HAL) format used by OpenProject API v3.
//! See: https://datatracker.ietf.org/doc/html/draft-kelly-json-hal-08

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A HAL link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalLink {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl HalLink {
    /// Create a simple link with just an href
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            title: None,
            method: None,
            templated: None,
            payload: None,
        }
    }

    /// Create a link with title
    pub fn with_title(href: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            title: Some(title.into()),
            method: None,
            templated: None,
            payload: None,
        }
    }

    /// Create a templated link
    pub fn templated(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            title: None,
            method: None,
            templated: Some(true),
            payload: None,
        }
    }

    /// Add method to link
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Add payload schema
    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }
}

/// Collection of HAL links
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HalLinks(HashMap<String, HalLinkValue>);

/// A link value can be a single link or an array of links
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HalLinkValue {
    Single(HalLink),
    Array(Vec<HalLink>),
}

impl HalLinks {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Add a single link
    pub fn add(&mut self, rel: impl Into<String>, link: HalLink) {
        self.0.insert(rel.into(), HalLinkValue::Single(link));
    }

    /// Add multiple links for a relation
    pub fn add_array(&mut self, rel: impl Into<String>, links: Vec<HalLink>) {
        self.0.insert(rel.into(), HalLinkValue::Array(links));
    }

    /// Builder pattern: add single link
    pub fn with(mut self, rel: impl Into<String>, link: HalLink) -> Self {
        self.add(rel, link);
        self
    }

    /// Builder pattern: add link array
    pub fn with_array(mut self, rel: impl Into<String>, links: Vec<HalLink>) -> Self {
        self.add_array(rel, links);
        self
    }

    /// Check if a relation exists
    pub fn has(&self, rel: &str) -> bool {
        self.0.contains_key(rel)
    }

    /// Get a link by relation
    pub fn get(&self, rel: &str) -> Option<&HalLinkValue> {
        self.0.get(rel)
    }
}

/// Embedded resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HalEmbedded(HashMap<String, serde_json::Value>);

impl HalEmbedded {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Add an embedded resource
    pub fn add<T: Serialize>(&mut self, rel: impl Into<String>, resource: T) {
        self.0.insert(
            rel.into(),
            serde_json::to_value(resource).unwrap_or(serde_json::Value::Null),
        );
    }

    /// Builder pattern: add embedded resource
    pub fn with<T: Serialize>(mut self, rel: impl Into<String>, resource: T) -> Self {
        self.add(rel, resource);
        self
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A HAL resource wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalResource<T> {
    #[serde(rename = "_type")]
    pub resource_type: String,
    #[serde(flatten)]
    pub resource: T,
    #[serde(rename = "_links")]
    pub links: HalLinks,
    #[serde(rename = "_embedded", skip_serializing_if = "Option::is_none")]
    pub embedded: Option<HalEmbedded>,
}

impl<T: Serialize> HalResource<T> {
    /// Create a new HAL resource
    pub fn new(resource_type: impl Into<String>, resource: T) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource,
            links: HalLinks::new(),
            embedded: None,
        }
    }

    /// Add self link
    pub fn with_self_link(mut self, href: impl Into<String>) -> Self {
        self.links.add("self", HalLink::new(href));
        self
    }

    /// Add a link
    pub fn with_link(mut self, rel: impl Into<String>, link: HalLink) -> Self {
        self.links.add(rel, link);
        self
    }

    /// Add links
    pub fn with_links(mut self, links: HalLinks) -> Self {
        self.links = links;
        self
    }

    /// Add embedded resources
    pub fn with_embedded(mut self, embedded: HalEmbedded) -> Self {
        if !embedded.is_empty() {
            self.embedded = Some(embedded);
        }
        self
    }
}

/// A HAL collection (paginated)
#[derive(Debug, Clone, Serialize)]
pub struct HalCollection<T> {
    #[serde(rename = "_type")]
    pub collection_type: String,
    pub count: i64,
    pub total: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    pub offset: i64,
    #[serde(rename = "_links")]
    pub links: HalLinks,
    #[serde(rename = "_embedded")]
    pub embedded: HalCollectionEmbedded<T>,
}

/// Embedded elements in a collection
#[derive(Debug, Clone, Serialize)]
pub struct HalCollectionEmbedded<T> {
    pub elements: Vec<T>,
}

impl<T: Serialize> HalCollection<T> {
    /// Create a new collection
    pub fn new(
        collection_type: impl Into<String>,
        elements: Vec<T>,
        total: i64,
        page_size: i64,
        offset: i64,
    ) -> Self {
        let count = elements.len() as i64;
        Self {
            collection_type: collection_type.into(),
            count,
            total,
            page_size,
            offset,
            links: HalLinks::new(),
            embedded: HalCollectionEmbedded { elements },
        }
    }

    /// Add pagination links
    pub fn with_pagination_links(mut self, base_url: &str, page: i64, per_page: i64) -> Self {
        let total_pages = (self.total + per_page - 1) / per_page;

        // Self link
        self.links.add(
            "self",
            HalLink::new(format!("{}?offset={}&pageSize={}", base_url, self.offset, self.page_size)),
        );

        // Jump to page link (templated)
        self.links.add(
            "jumpTo",
            HalLink::templated(format!("{}{{?offset,pageSize}}", base_url)),
        );

        // Change size link (templated)
        self.links.add(
            "changeSize",
            HalLink::templated(format!("{}{{?offset,pageSize}}", base_url)),
        );

        // Previous page
        if page > 1 {
            let prev_offset = (page - 2) * per_page;
            self.links.add(
                "previousByOffset",
                HalLink::new(format!("{}?offset={}&pageSize={}", base_url, prev_offset, per_page)),
            );
        }

        // Next page
        if page < total_pages {
            let next_offset = page * per_page;
            self.links.add(
                "nextByOffset",
                HalLink::new(format!("{}?offset={}&pageSize={}", base_url, next_offset, per_page)),
            );
        }

        self
    }

    /// Add a custom link
    pub fn with_link(mut self, rel: impl Into<String>, link: HalLink) -> Self {
        self.links.add(rel, link);
        self
    }
}

/// Error response in HAL format
#[derive(Debug, Clone, Serialize)]
pub struct HalError {
    #[serde(rename = "_type")]
    pub error_type: String,
    #[serde(rename = "errorIdentifier")]
    pub error_identifier: String,
    pub message: String,
    #[serde(rename = "_embedded", skip_serializing_if = "Option::is_none")]
    pub embedded: Option<HalErrorEmbedded>,
}

/// Embedded error details
#[derive(Debug, Clone, Serialize)]
pub struct HalErrorEmbedded {
    pub details: HalErrorDetails,
}

/// Detailed error information
#[derive(Debug, Clone, Serialize)]
pub struct HalErrorDetails {
    #[serde(rename = "_type")]
    pub details_type: String,
    pub attribute: Option<String>,
    pub message: String,
}

impl HalError {
    /// Create a simple error
    pub fn new(identifier: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: "Error".to_string(),
            error_identifier: identifier.into(),
            message: message.into(),
            embedded: None,
        }
    }

    /// Create a not found error
    pub fn not_found(resource: &str) -> Self {
        Self::new(
            "urn:openproject-org:api:v3:errors:NotFound",
            format!("The requested {} does not exist.", resource),
        )
    }

    /// Create an unauthorized error
    pub fn unauthorized() -> Self {
        Self::new(
            "urn:openproject-org:api:v3:errors:Unauthenticated",
            "You need to be authenticated to access this resource.",
        )
    }

    /// Create a forbidden error
    pub fn forbidden() -> Self {
        Self::new(
            "urn:openproject-org:api:v3:errors:MissingPermission",
            "You are not authorized to access this resource.",
        )
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            "urn:openproject-org:api:v3:errors:PropertyConstraintViolation",
            message,
        )
    }

    /// Create a multi-validation error with details
    pub fn validation_errors(errors: Vec<(String, String)>) -> Self {
        let messages: Vec<String> = errors.iter().map(|(_, msg)| msg.clone()).collect();
        let mut error = Self::new(
            "urn:openproject-org:api:v3:errors:MultipleErrors",
            messages.join(" "),
        );

        let details: Vec<HalErrorDetails> = errors
            .into_iter()
            .map(|(attr, msg)| HalErrorDetails {
                details_type: "Error".to_string(),
                attribute: Some(attr),
                message: msg,
            })
            .collect();

        if !details.is_empty() {
            error.embedded = Some(HalErrorEmbedded {
                details: details[0].clone(),
            });
        }

        error
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            "urn:openproject-org:api:v3:errors:InternalServerError",
            message,
        )
    }

    /// Create a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(
            "urn:openproject-org:api:v3:errors:UpdateConflict",
            message,
        )
    }
}

/// Common HAL link relations
pub mod rels {
    pub const SELF: &str = "self";
    pub const SCHEMA: &str = "schema";
    pub const UPDATE: &str = "update";
    pub const UPDATE_IMMEDIATELY: &str = "updateImmediately";
    pub const DELETE: &str = "delete";
    pub const LOG_TIME: &str = "logTime";
    pub const MOVE: &str = "move";
    pub const COPY: &str = "copy";
    pub const PDF: &str = "pdf";
    pub const ATOM: &str = "atom";
    pub const AVAILABLE_WATCHERS: &str = "availableWatchers";
    pub const WATCH: &str = "watch";
    pub const UNWATCH: &str = "unwatch";
    pub const ADD_WATCHER: &str = "addWatcher";
    pub const REMOVE_WATCHER: &str = "removeWatcher";
    pub const ADD_RELATION: &str = "addRelation";
    pub const ADD_CHILD: &str = "addChild";
    pub const CHANGE_PARENT: &str = "changeParent";
    pub const ADD_COMMENT: &str = "addComment";
    pub const PREVIEW_MARKUP: &str = "previewMarkup";
    pub const AVAILABLE_PROJECTS: &str = "availableProjects";
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hal_link() {
        let link = HalLink::new("/api/v3/work_packages/1");
        let json = serde_json::to_value(&link).unwrap();
        assert_eq!(json["href"], "/api/v3/work_packages/1");
    }

    #[test]
    fn test_hal_link_with_title() {
        let link = HalLink::with_title("/api/v3/users/1", "John Doe");
        let json = serde_json::to_value(&link).unwrap();
        assert_eq!(json["href"], "/api/v3/users/1");
        assert_eq!(json["title"], "John Doe");
    }

    #[test]
    fn test_hal_links() {
        let links = HalLinks::new()
            .with("self", HalLink::new("/api/v3/work_packages/1"))
            .with("project", HalLink::with_title("/api/v3/projects/1", "My Project"));

        assert!(links.has("self"));
        assert!(links.has("project"));
        assert!(!links.has("unknown"));
    }

    #[test]
    fn test_hal_resource() {
        #[derive(Serialize)]
        struct TestResource {
            id: i64,
            name: String,
        }

        let resource = HalResource::new(
            "TestResource",
            TestResource {
                id: 1,
                name: "Test".to_string(),
            },
        )
        .with_self_link("/api/v3/test/1");

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["_type"], "TestResource");
        assert_eq!(json["id"], 1);
        assert_eq!(json["name"], "Test");
        assert_eq!(json["_links"]["self"]["href"], "/api/v3/test/1");
    }

    #[test]
    fn test_hal_collection() {
        #[derive(Serialize)]
        struct Item {
            id: i64,
        }

        let items = vec![Item { id: 1 }, Item { id: 2 }];
        let collection = HalCollection::new("Items", items, 10, 20, 0)
            .with_pagination_links("/api/v3/items", 1, 20);

        let json = serde_json::to_value(&collection).unwrap();
        assert_eq!(json["_type"], "Items");
        assert_eq!(json["count"], 2);
        assert_eq!(json["total"], 10);
        assert_eq!(json["pageSize"], 20);
    }

    #[test]
    fn test_hal_error() {
        let error = HalError::not_found("WorkPackage");
        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["_type"], "Error");
        assert_eq!(
            json["errorIdentifier"],
            "urn:openproject-org:api:v3:errors:NotFound"
        );
    }

    #[test]
    fn test_validation_error() {
        let error = HalError::validation_errors(vec![
            ("subject".to_string(), "can't be blank".to_string()),
        ]);
        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["_type"], "Error");
        assert!(json["errorIdentifier"]
            .as_str()
            .unwrap()
            .contains("MultipleErrors"));
    }
}
