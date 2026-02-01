//! Query HAL Representer
//!
//! Converts query models to HAL+JSON format compatible with OpenProject API v3.

use chrono::{DateTime, Utc};
use op_core::traits::Id;
use op_queries::{
    DisplayRepresentation, Filter, FilterOperator, FilterValue, Query, QueryVisibility,
    SortCriterion, SortDirection,
};
use serde::Serialize;

use super::hal::{HalCollection, HalLink, HalLinks, HalResource, rels};

/// Query representation for API responses
#[derive(Debug, Clone, Serialize)]
pub struct QueryRepresentation {
    pub id: Option<Id>,
    pub name: String,
    pub filters: Vec<FilterRepresentation>,
    #[serde(rename = "sums")]
    pub show_sums: bool,
    #[serde(rename = "timelineVisible")]
    pub timeline_visible: bool,
    #[serde(rename = "timelineZoomLevel")]
    pub timeline_zoom_level: String,
    #[serde(rename = "showHierarchies")]
    pub show_hierarchies: bool,
    pub starred: bool,
    pub public: bool,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Filter representation
#[derive(Debug, Clone, Serialize)]
pub struct FilterRepresentation {
    pub name: String,
    #[serde(rename = "_links")]
    pub links: FilterLinks,
}

/// Filter links (for filter schema and operator)
#[derive(Debug, Clone, Serialize)]
pub struct FilterLinks {
    pub filter: HalLink,
    pub operator: HalLink,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<HalLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<HalLink>,
}

/// Query representer
pub struct QueryRepresenter;

impl QueryRepresenter {
    /// Create a HAL resource for a single query
    pub fn represent(query: &Query, project_id: Option<Id>) -> HalResource<QueryRepresentation> {
        let filters = Self::represent_filters(&query.filters.filters());

        let rep = QueryRepresentation {
            id: query.id,
            name: query.name.clone(),
            filters,
            show_sums: query.show_sums,
            timeline_visible: query.show_timeline,
            timeline_zoom_level: Self::zoom_level_name(&query),
            show_hierarchies: query.show_hierarchies,
            starred: query.starred,
            public: query.is_public(),
            created_at: None, // Would come from database
            updated_at: None, // Would come from database
        };

        let links = Self::build_links(query, project_id);

        HalResource::new("Query", rep).with_links(links)
    }

    /// Create a HAL collection of queries
    pub fn represent_collection(
        queries: Vec<&Query>,
        total: i64,
        offset: i64,
        page_size: i64,
        base_url: &str,
        project_id: Option<Id>,
    ) -> HalCollection<HalResource<QueryRepresentation>> {
        let page = (offset / page_size) + 1;
        let elements: Vec<HalResource<QueryRepresentation>> = queries
            .into_iter()
            .map(|q| Self::represent(q, project_id))
            .collect();

        HalCollection::new("QueryCollection", elements, total, page_size, offset)
            .with_pagination_links(base_url, page, page_size)
            .with_link(
                "createQuery",
                HalLink::new("/api/v3/queries/form").method("POST"),
            )
    }

    /// Build links for a query
    fn build_links(query: &Query, project_id: Option<Id>) -> HalLinks {
        let mut links = HalLinks::new();

        // Self link
        if let Some(id) = query.id {
            let base = format!("/api/v3/queries/{}", id);
            links.add(rels::SELF, HalLink::new(&base));
            links.add(rels::UPDATE, HalLink::new(format!("{}/form", base)).method("POST"));
            links.add(rels::UPDATE_IMMEDIATELY, HalLink::new(&base).method("PATCH"));
            links.add(rels::DELETE, HalLink::new(&base).method("DELETE"));
            links.add("star", HalLink::new(format!("{}/star", base)).method("PATCH"));
            links.add("unstar", HalLink::new(format!("{}/unstar", base)).method("PATCH"));
        } else {
            // Unsaved query
            links.add(rels::SELF, HalLink::templated("/api/v3/queries/new"));
        }

        // Results link
        let results_url = if let Some(id) = query.id {
            format!("/api/v3/queries/{}/results", id)
        } else {
            "/api/v3/work_packages".to_string()
        };
        links.add("results", HalLink::new(results_url));

        // Project link
        if let Some(pid) = project_id.or(query.project_id) {
            links.add("project", HalLink::new(format!("/api/v3/projects/{}", pid)));
        }

        // User link
        if let Some(user_id) = query.user_id {
            links.add("user", HalLink::new(format!("/api/v3/users/{}", user_id)));
        }

        // Columns link
        let column_links: Vec<HalLink> = query
            .columns
            .columns()
            .iter()
            .map(|col| {
                HalLink::with_title(
                    format!("/api/v3/queries/columns/{}", col.name),
                    col.display_caption(),
                )
            })
            .collect();
        if !column_links.is_empty() {
            links.add_array("columns", column_links);
        }

        // Sort by link
        let sort_links: Vec<HalLink> = query
            .sorts
            .criteria()
            .iter()
            .map(|criterion| {
                let direction = match criterion.direction {
                    SortDirection::Asc => "asc",
                    SortDirection::Desc => "desc",
                };
                HalLink::with_title(
                    format!(
                        "/api/v3/queries/sort_bys/{}:{}",
                        criterion.attribute, direction
                    ),
                    format!("{} ({})", criterion.attribute, direction),
                )
            })
            .collect();
        if !sort_links.is_empty() {
            links.add_array("sortBy", sort_links);
        }

        // Group by link
        if let Some(ref group_attr) = query.group_by.attribute {
            links.add(
                "groupBy",
                HalLink::with_title(
                    format!("/api/v3/queries/group_bys/{}", group_attr),
                    group_attr.clone(),
                ),
            );
        }

        links
    }

    /// Represent filters as API format
    fn represent_filters(filters: &[Filter]) -> Vec<FilterRepresentation> {
        filters
            .iter()
            .map(|filter| {
                let operator_href = Self::operator_href(&filter.operator);
                let values = Self::values_to_links(&filter.attribute, &filter.values);

                FilterRepresentation {
                    name: filter.attribute.clone(),
                    links: FilterLinks {
                        filter: HalLink::new(format!(
                            "/api/v3/queries/filters/{}",
                            filter.attribute
                        )),
                        operator: HalLink::new(operator_href),
                        schema: Some(HalLink::new(format!(
                            "/api/v3/queries/filter_instance_schemas/{}",
                            filter.attribute
                        ))),
                        values,
                    },
                }
            })
            .collect()
    }

    /// Get operator href
    fn operator_href(op: &FilterOperator) -> String {
        let op_name = match op {
            FilterOperator::Equals => "=",
            FilterOperator::NotEquals => "!",
            FilterOperator::Contains => "~",
            FilterOperator::NotContains => "!~",
            FilterOperator::StartsWith => "**",
            FilterOperator::EndsWith => "*~",
            FilterOperator::GreaterThan => ">",
            FilterOperator::GreaterThanOrEqual => ">=",
            FilterOperator::LessThan => "<",
            FilterOperator::LessThanOrEqual => "<=",
            FilterOperator::Between => "<>d",
            FilterOperator::IsNull => "o",
            FilterOperator::IsNotNull => "c",
            FilterOperator::Today => "t",
            FilterOperator::ThisWeek => "w",
            FilterOperator::DaysAgo(_) => "t-",
            FilterOperator::DaysFromNow(_) => "t+",
            FilterOperator::LessThanDaysAgo(_) => "<t-",
            FilterOperator::MoreThanDaysAgo(_) => ">t-",
            FilterOperator::LessThanDaysFromNow(_) => "<t+",
            FilterOperator::MoreThanDaysFromNow(_) => ">t+",
            FilterOperator::CurrentUser => "=",
        };
        format!("/api/v3/queries/operators/{}", op_name)
    }

    /// Convert filter values to links
    fn values_to_links(attribute: &str, values: &FilterValue) -> Vec<HalLink> {
        match values {
            FilterValue::Id(id) => vec![Self::value_link(attribute, *id)],
            FilterValue::Ids(ids) => ids.iter().map(|id| Self::value_link(attribute, *id)).collect(),
            FilterValue::Me => vec![HalLink::with_title("/api/v3/users/me", "Me")],
            _ => vec![],
        }
    }

    /// Create a value link for filter
    fn value_link(attribute: &str, id: Id) -> HalLink {
        let href = match attribute {
            "status_id" => format!("/api/v3/statuses/{}", id),
            "type_id" => format!("/api/v3/types/{}", id),
            "priority_id" => format!("/api/v3/priorities/{}", id),
            "project_id" => format!("/api/v3/projects/{}", id),
            "author_id" | "assigned_to_id" | "responsible_id" | "watcher_id" => {
                format!("/api/v3/users/{}", id)
            }
            "version_id" => format!("/api/v3/versions/{}", id),
            "category_id" => format!("/api/v3/categories/{}", id),
            "parent_id" => format!("/api/v3/work_packages/{}", id),
            _ => format!("/api/v3/custom_options/{}", id),
        };
        HalLink::new(href)
    }

    /// Get zoom level name
    fn zoom_level_name(query: &Query) -> String {
        use op_queries::query::TimelineZoomLevel;
        match query.timeline_zoom_level {
            TimelineZoomLevel::Days => "days".to_string(),
            TimelineZoomLevel::Weeks => "weeks".to_string(),
            TimelineZoomLevel::Months => "months".to_string(),
            TimelineZoomLevel::Quarters => "quarters".to_string(),
            TimelineZoomLevel::Years => "years".to_string(),
            TimelineZoomLevel::Auto => "auto".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use op_queries::QueryBuilder;

    #[test]
    fn test_query_representation() {
        let query = QueryBuilder::new()
            .name("My Query")
            .status(vec![1, 2])
            .build();

        let hal = QueryRepresenter::represent(&query, Some(1));
        let json = serde_json::to_value(&hal).unwrap();

        assert_eq!(json["_type"], "Query");
        assert_eq!(json["name"], "My Query");
    }

    #[test]
    fn test_filter_representation() {
        let query = QueryBuilder::new()
            .name("Filtered")
            .status(vec![1])
            .assigned_to(vec![5])
            .build();

        let hal = QueryRepresenter::represent(&query, None);
        let json = serde_json::to_value(&hal).unwrap();

        let filters = json["filters"].as_array().unwrap();
        assert_eq!(filters.len(), 2);
    }

    #[test]
    fn test_sort_links() {
        let query = QueryBuilder::new()
            .name("Sorted")
            .sort_by_desc("updated_at")
            .then_by_asc("id")
            .build();

        let hal = QueryRepresenter::represent(&query, None);
        let json = serde_json::to_value(&hal).unwrap();

        let sort_links = json["_links"]["sortBy"].as_array();
        assert!(sort_links.is_some());
        assert_eq!(sort_links.unwrap().len(), 2);
    }

    #[test]
    fn test_operator_href() {
        assert_eq!(
            QueryRepresenter::operator_href(&FilterOperator::Equals),
            "/api/v3/queries/operators/="
        );
        assert_eq!(
            QueryRepresenter::operator_href(&FilterOperator::Contains),
            "/api/v3/queries/operators/~"
        );
    }
}
