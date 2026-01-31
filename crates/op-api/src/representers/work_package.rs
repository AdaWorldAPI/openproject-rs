//! Work Package HAL Representer
//!
//! Converts work package models to HAL+JSON format compatible with OpenProject API v3.

use chrono::{DateTime, NaiveDate, Utc};
use op_core::traits::Id;
use serde::Serialize;

use super::hal::{HalCollection, HalEmbedded, HalLink, HalLinks, HalResource, rels};

/// Work package representation for API responses
#[derive(Debug, Clone, Serialize)]
pub struct WorkPackageRepresentation {
    pub id: Id,
    #[serde(rename = "lockVersion")]
    pub lock_version: i32,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<FormattableText>,
    #[serde(rename = "scheduleManually")]
    pub schedule_manually: bool,
    #[serde(rename = "startDate", skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(rename = "dueDate", skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(rename = "derivedStartDate", skip_serializing_if = "Option::is_none")]
    pub derived_start_date: Option<String>,
    #[serde(rename = "derivedDueDate", skip_serializing_if = "Option::is_none")]
    pub derived_due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    #[serde(rename = "estimatedTime", skip_serializing_if = "Option::is_none")]
    pub estimated_time: Option<String>,
    #[serde(rename = "derivedEstimatedTime", skip_serializing_if = "Option::is_none")]
    pub derived_estimated_time: Option<String>,
    #[serde(rename = "spentTime", skip_serializing_if = "Option::is_none")]
    pub spent_time: Option<String>,
    #[serde(rename = "percentageDone")]
    pub percentage_done: i32,
    #[serde(rename = "derivedPercentageDone", skip_serializing_if = "Option::is_none")]
    pub derived_percentage_done: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(rename = "storyPoints", skip_serializing_if = "Option::is_none")]
    pub story_points: Option<i32>,
    #[serde(rename = "remainingTime", skip_serializing_if = "Option::is_none")]
    pub remaining_time: Option<String>,
}

/// Formattable text (supports HTML/Markdown)
#[derive(Debug, Clone, Serialize)]
pub struct FormattableText {
    pub format: String,
    pub raw: String,
    pub html: String,
}

impl FormattableText {
    pub fn plain(text: &str) -> Self {
        Self {
            format: "plain".to_string(),
            raw: text.to_string(),
            html: format!("<p>{}</p>", html_escape(text)),
        }
    }

    pub fn markdown(text: &str) -> Self {
        Self {
            format: "markdown".to_string(),
            raw: text.to_string(),
            // TODO: Actually render markdown to HTML
            html: format!("<p>{}</p>", html_escape(text)),
        }
    }
}

/// Work package representer - builds HAL responses
pub struct WorkPackageRepresenter;

impl WorkPackageRepresenter {
    /// Create a HAL resource for a single work package
    pub fn represent(
        wp: WorkPackageData,
        embed_options: &EmbedOptions,
    ) -> HalResource<WorkPackageRepresentation> {
        let rep = WorkPackageRepresentation {
            id: wp.id,
            lock_version: wp.lock_version,
            subject: wp.subject.clone(),
            description: wp.description.as_ref().map(|d| FormattableText::markdown(d)),
            schedule_manually: wp.schedule_manually,
            start_date: wp.start_date.map(|d| d.to_string()),
            due_date: wp.due_date.map(|d| d.to_string()),
            derived_start_date: None, // Computed from children
            derived_due_date: None,   // Computed from children
            duration: wp.duration.map(|d| format!("P{}D", d)),
            estimated_time: wp.estimated_hours.map(|h| format_duration(h)),
            derived_estimated_time: None, // Computed from children
            spent_time: wp.spent_hours.map(|h| format_duration(h)),
            percentage_done: wp.done_ratio,
            derived_percentage_done: None,
            created_at: wp.created_at,
            updated_at: wp.updated_at,
            position: wp.position,
            story_points: wp.story_points,
            remaining_time: wp.remaining_hours.map(|h| format_duration(h)),
        };

        let links = Self::build_links(&wp);
        let embedded = Self::build_embedded(&wp, embed_options);

        HalResource::new("WorkPackage", rep)
            .with_links(links)
            .with_embedded(embedded)
    }

    /// Create a HAL collection of work packages
    pub fn represent_collection(
        work_packages: Vec<WorkPackageData>,
        total: i64,
        offset: i64,
        page_size: i64,
        base_url: &str,
        embed_options: &EmbedOptions,
    ) -> HalCollection<HalResource<WorkPackageRepresentation>> {
        let page = (offset / page_size) + 1;
        let elements: Vec<HalResource<WorkPackageRepresentation>> = work_packages
            .into_iter()
            .map(|wp| Self::represent(wp, embed_options))
            .collect();

        HalCollection::new("WorkPackageCollection", elements, total, page_size, offset)
            .with_pagination_links(base_url, page, page_size)
            .with_link(
                "createWorkPackage",
                HalLink::new("/api/v3/work_packages/form").method("POST"),
            )
            .with_link(
                "createWorkPackageImmediate",
                HalLink::new("/api/v3/work_packages").method("POST"),
            )
            .with_link("schemas", HalLink::new("/api/v3/work_packages/schemas"))
    }

    /// Build links for a work package
    fn build_links(wp: &WorkPackageData) -> HalLinks {
        let base = format!("/api/v3/work_packages/{}", wp.id);

        let mut links = HalLinks::new()
            .with(rels::SELF, HalLink::new(&base))
            .with(rels::SCHEMA, HalLink::new(format!("{}/schema", base)))
            .with(
                rels::UPDATE,
                HalLink::new(format!("{}/form", base)).method("POST"),
            )
            .with(
                rels::UPDATE_IMMEDIATELY,
                HalLink::new(&base).method("PATCH"),
            )
            .with(rels::DELETE, HalLink::new(&base).method("DELETE"))
            .with(
                rels::LOG_TIME,
                HalLink::new("/api/v3/time_entries/form").method("POST"),
            )
            .with(rels::MOVE, HalLink::new(format!("{}/move", base)))
            .with(rels::COPY, HalLink::new(format!("{}/copy", base)))
            .with("activities", HalLink::new(format!("{}/activities", base)))
            .with("revisions", HalLink::new(format!("{}/revisions", base)))
            .with("watchers", HalLink::new(format!("{}/watchers", base)))
            .with("attachments", HalLink::new(format!("{}/attachments", base)))
            .with("relations", HalLink::new(format!("{}/relations", base)))
            .with("children", HalLink::new(format!("{}/children", base)));

        // Project link
        links.add(
            "project",
            HalLink::with_title(
                format!("/api/v3/projects/{}", wp.project_id),
                wp.project_name.as_deref().unwrap_or(""),
            ),
        );

        // Status link
        links.add(
            "status",
            HalLink::with_title(
                format!("/api/v3/statuses/{}", wp.status_id),
                wp.status_name.as_deref().unwrap_or(""),
            ),
        );

        // Type link
        links.add(
            "type",
            HalLink::with_title(
                format!("/api/v3/types/{}", wp.type_id),
                wp.type_name.as_deref().unwrap_or(""),
            ),
        );

        // Priority link
        if let Some(priority_id) = wp.priority_id {
            links.add(
                "priority",
                HalLink::with_title(
                    format!("/api/v3/priorities/{}", priority_id),
                    wp.priority_name.as_deref().unwrap_or(""),
                ),
            );
        }

        // Author link
        if let Some(author_id) = wp.author_id {
            links.add(
                "author",
                HalLink::with_title(
                    format!("/api/v3/users/{}", author_id),
                    wp.author_name.as_deref().unwrap_or(""),
                ),
            );
        }

        // Assignee link
        if let Some(assignee_id) = wp.assigned_to_id {
            links.add(
                "assignee",
                HalLink::with_title(
                    format!("/api/v3/users/{}", assignee_id),
                    wp.assignee_name.as_deref().unwrap_or(""),
                ),
            );
        }

        // Responsible link
        if let Some(responsible_id) = wp.responsible_id {
            links.add(
                "responsible",
                HalLink::with_title(
                    format!("/api/v3/users/{}", responsible_id),
                    wp.responsible_name.as_deref().unwrap_or(""),
                ),
            );
        }

        // Parent link
        if let Some(parent_id) = wp.parent_id {
            links.add(
                "parent",
                HalLink::with_title(
                    format!("/api/v3/work_packages/{}", parent_id),
                    wp.parent_subject.as_deref().unwrap_or(""),
                ),
            );
        }

        // Version link
        if let Some(version_id) = wp.version_id {
            links.add(
                "version",
                HalLink::with_title(
                    format!("/api/v3/versions/{}", version_id),
                    wp.version_name.as_deref().unwrap_or(""),
                ),
            );
        }

        // Category link
        if let Some(category_id) = wp.category_id {
            links.add(
                "category",
                HalLink::new(format!("/api/v3/categories/{}", category_id)),
            );
        }

        links
    }

    /// Build embedded resources
    fn build_embedded(wp: &WorkPackageData, options: &EmbedOptions) -> HalEmbedded {
        let mut embedded = HalEmbedded::new();

        if options.embed_status {
            if let (Some(name), Some(color), Some(is_closed)) =
                (&wp.status_name, &wp.status_color, wp.status_is_closed)
            {
                embedded.add(
                    "status",
                    StatusEmbedded {
                        _type: "Status".to_string(),
                        id: wp.status_id,
                        name: name.clone(),
                        color: color.clone(),
                        is_closed,
                    },
                );
            }
        }

        if options.embed_type {
            if let Some(name) = &wp.type_name {
                embedded.add(
                    "type",
                    TypeEmbedded {
                        _type: "Type".to_string(),
                        id: wp.type_id,
                        name: name.clone(),
                        color: wp.type_color.clone(),
                    },
                );
            }
        }

        if options.embed_priority {
            if let (Some(priority_id), Some(name)) = (wp.priority_id, &wp.priority_name) {
                embedded.add(
                    "priority",
                    PriorityEmbedded {
                        _type: "Priority".to_string(),
                        id: priority_id,
                        name: name.clone(),
                        color: wp.priority_color.clone(),
                    },
                );
            }
        }

        embedded
    }
}

/// Work package data for representation
#[derive(Debug, Clone)]
pub struct WorkPackageData {
    pub id: Id,
    pub lock_version: i32,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: Id,
    pub project_name: Option<String>,
    pub type_id: Id,
    pub type_name: Option<String>,
    pub type_color: Option<String>,
    pub status_id: Id,
    pub status_name: Option<String>,
    pub status_color: Option<String>,
    pub status_is_closed: Option<bool>,
    pub priority_id: Option<Id>,
    pub priority_name: Option<String>,
    pub priority_color: Option<String>,
    pub author_id: Option<Id>,
    pub author_name: Option<String>,
    pub assigned_to_id: Option<Id>,
    pub assignee_name: Option<String>,
    pub responsible_id: Option<Id>,
    pub responsible_name: Option<String>,
    pub category_id: Option<Id>,
    pub version_id: Option<Id>,
    pub version_name: Option<String>,
    pub parent_id: Option<Id>,
    pub parent_subject: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub spent_hours: Option<f64>,
    pub remaining_hours: Option<f64>,
    pub done_ratio: i32,
    pub schedule_manually: bool,
    pub duration: Option<i32>,
    pub position: Option<i32>,
    pub story_points: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Embedded status representation
#[derive(Debug, Clone, Serialize)]
struct StatusEmbedded {
    _type: String,
    id: Id,
    name: String,
    color: String,
    #[serde(rename = "isClosed")]
    is_closed: bool,
}

/// Embedded type representation
#[derive(Debug, Clone, Serialize)]
struct TypeEmbedded {
    _type: String,
    id: Id,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

/// Embedded priority representation
#[derive(Debug, Clone, Serialize)]
struct PriorityEmbedded {
    _type: String,
    id: Id,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

/// Options for embedding related resources
#[derive(Debug, Clone, Default)]
pub struct EmbedOptions {
    pub embed_status: bool,
    pub embed_type: bool,
    pub embed_priority: bool,
    pub embed_author: bool,
    pub embed_assignee: bool,
    pub embed_responsible: bool,
    pub embed_project: bool,
    pub embed_version: bool,
    pub embed_parent: bool,
}

impl EmbedOptions {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn all() -> Self {
        Self {
            embed_status: true,
            embed_type: true,
            embed_priority: true,
            embed_author: true,
            embed_assignee: true,
            embed_responsible: true,
            embed_project: true,
            embed_version: true,
            embed_parent: true,
        }
    }

    pub fn from_query_params(params: &str) -> Self {
        let mut options = Self::none();

        if params.is_empty() {
            return options;
        }

        for embed in params.split(',') {
            match embed.trim() {
                "status" => options.embed_status = true,
                "type" => options.embed_type = true,
                "priority" => options.embed_priority = true,
                "author" => options.embed_author = true,
                "assignee" => options.embed_assignee = true,
                "responsible" => options.embed_responsible = true,
                "project" => options.embed_project = true,
                "version" => options.embed_version = true,
                "parent" => options.embed_parent = true,
                _ => {}
            }
        }

        options
    }
}

/// Format hours as ISO 8601 duration
fn format_duration(hours: f64) -> String {
    let total_minutes = (hours * 60.0).round() as i64;
    let h = total_minutes / 60;
    let m = total_minutes % 60;

    if m == 0 {
        format!("PT{}H", h)
    } else {
        format!("PT{}H{}M", h, m)
    }
}

/// Simple HTML escaping
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(1.0), "PT1H");
        assert_eq!(format_duration(1.5), "PT1H30M");
        assert_eq!(format_duration(0.5), "PT0H30M");
        assert_eq!(format_duration(8.0), "PT8H");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("A & B"), "A &amp; B");
    }

    #[test]
    fn test_embed_options_from_query() {
        let options = EmbedOptions::from_query_params("status,type,priority");
        assert!(options.embed_status);
        assert!(options.embed_type);
        assert!(options.embed_priority);
        assert!(!options.embed_author);
    }

    #[test]
    fn test_formattable_text() {
        let text = FormattableText::plain("Hello <world>");
        assert_eq!(text.format, "plain");
        assert_eq!(text.raw, "Hello <world>");
        assert!(text.html.contains("&lt;world&gt;"));
    }
}
