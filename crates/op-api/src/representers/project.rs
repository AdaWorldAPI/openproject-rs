//! Project HAL Representer
//!
//! Converts project models to HAL+JSON format compatible with OpenProject API v3.

use chrono::{DateTime, Utc};
use op_core::traits::Id;
use serde::Serialize;

use super::hal::{HalCollection, HalLink, HalLinks, HalResource, rels};

/// Project representation for API responses
#[derive(Debug, Clone, Serialize)]
pub struct ProjectRepresentation {
    pub id: Id,
    pub identifier: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<FormattableText>,
    pub public: bool,
    pub active: bool,
    #[serde(rename = "statusExplanation", skip_serializing_if = "Option::is_none")]
    pub status_explanation: Option<FormattableText>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// Formattable text
#[derive(Debug, Clone, Serialize)]
pub struct FormattableText {
    pub format: String,
    pub raw: String,
    pub html: String,
}

/// Project representer
pub struct ProjectRepresenter;

impl ProjectRepresenter {
    /// Create a HAL resource for a single project
    pub fn represent(project: ProjectData) -> HalResource<ProjectRepresentation> {
        let rep = ProjectRepresentation {
            id: project.id,
            identifier: project.identifier.clone(),
            name: project.name.clone(),
            description: project.description.as_ref().map(|d| FormattableText {
                format: "markdown".to_string(),
                raw: d.clone(),
                html: format!("<p>{}</p>", html_escape(d)),
            }),
            public: project.public,
            active: project.active,
            status_explanation: None,
            created_at: project.created_at,
            updated_at: project.updated_at,
        };

        let links = Self::build_links(&project);

        HalResource::new("Project", rep).with_links(links)
    }

    /// Create a HAL collection of projects
    pub fn represent_collection(
        projects: Vec<ProjectData>,
        total: i64,
        offset: i64,
        page_size: i64,
        base_url: &str,
    ) -> HalCollection<HalResource<ProjectRepresentation>> {
        let page = (offset / page_size) + 1;
        let elements: Vec<HalResource<ProjectRepresentation>> = projects
            .into_iter()
            .map(|p| Self::represent(p))
            .collect();

        HalCollection::new("ProjectCollection", elements, total, page_size, offset)
            .with_pagination_links(base_url, page, page_size)
            .with_link(
                "createProject",
                HalLink::new("/api/v3/projects/form").method("POST"),
            )
            .with_link(
                "createProjectImmediate",
                HalLink::new("/api/v3/projects").method("POST"),
            )
    }

    /// Build links for a project
    fn build_links(project: &ProjectData) -> HalLinks {
        let base = format!("/api/v3/projects/{}", project.id);
        let by_identifier = format!("/api/v3/projects/{}", project.identifier);

        let mut links = HalLinks::new()
            .with(rels::SELF, HalLink::new(&base))
            .with(rels::SCHEMA, HalLink::new("/api/v3/projects/schema"))
            .with(rels::UPDATE, HalLink::new(format!("{}/form", base)).method("POST"))
            .with(rels::UPDATE_IMMEDIATELY, HalLink::new(&base).method("PATCH"))
            .with(rels::DELETE, HalLink::new(&base).method("DELETE"))
            .with("createWorkPackage", HalLink::new(format!("{}/work_packages/form", base)).method("POST"))
            .with("createWorkPackageImmediate", HalLink::new(format!("{}/work_packages", base)).method("POST"))
            .with("workPackages", HalLink::new(format!("{}/work_packages", base)))
            .with("categories", HalLink::new(format!("{}/categories", base)))
            .with("versions", HalLink::new(format!("{}/versions", base)))
            .with("memberships", HalLink::new(format!("/api/v3/memberships?filters=[{{\"project\":{{\"operator\":\"=\",\"values\":[\"{}\"]}}}}]", project.id)))
            .with("types", HalLink::new(format!("{}/types", base)))
            .with("storages", HalLink::new(format!("{}/storages", base)));

        // Parent link
        if let Some(parent_id) = project.parent_id {
            links.add(
                "parent",
                HalLink::with_title(
                    format!("/api/v3/projects/{}", parent_id),
                    project.parent_name.as_deref().unwrap_or(""),
                ),
            );
        }

        // Ancestors (if any)
        if !project.ancestors.is_empty() {
            let ancestor_links: Vec<HalLink> = project
                .ancestors
                .iter()
                .map(|(id, name)| {
                    HalLink::with_title(format!("/api/v3/projects/{}", id), name.as_str())
                })
                .collect();
            links.add_array("ancestors", ancestor_links);
        }

        links
    }
}

/// Project data for representation
#[derive(Debug, Clone)]
pub struct ProjectData {
    pub id: Id,
    pub identifier: String,
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
    pub active: bool,
    pub parent_id: Option<Id>,
    pub parent_name: Option<String>,
    pub ancestors: Vec<(Id, String)>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> ProjectData {
        ProjectData {
            id: 1,
            identifier: "test-project".to_string(),
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
            public: true,
            active: true,
            parent_id: None,
            parent_name: None,
            ancestors: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_project_representation() {
        let project = create_test_project();
        let hal = ProjectRepresenter::represent(project);

        let json = serde_json::to_value(&hal).unwrap();
        assert_eq!(json["_type"], "Project");
        assert_eq!(json["id"], 1);
        assert_eq!(json["identifier"], "test-project");
        assert_eq!(json["name"], "Test Project");
    }

    #[test]
    fn test_project_links() {
        let project = create_test_project();
        let hal = ProjectRepresenter::represent(project);

        let json = serde_json::to_value(&hal).unwrap();
        assert!(json["_links"]["self"]["href"].as_str().is_some());
        assert!(json["_links"]["workPackages"]["href"].as_str().is_some());
    }

    #[test]
    fn test_project_with_parent() {
        let mut project = create_test_project();
        project.parent_id = Some(10);
        project.parent_name = Some("Parent Project".to_string());

        let hal = ProjectRepresenter::represent(project);
        let json = serde_json::to_value(&hal).unwrap();

        assert_eq!(json["_links"]["parent"]["href"], "/api/v3/projects/10");
        assert_eq!(json["_links"]["parent"]["title"], "Parent Project");
    }
}
