//! Pagination types for API responses
//!
//! Mirrors OpenProject's API v3 pagination patterns.

use serde::{Deserialize, Serialize};

/// Pagination parameters (from query string)
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: i64,

    /// Offset (alternative to page)
    pub offset: Option<i64>,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

impl PaginationParams {
    pub fn new(page: i64, per_page: i64) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 1000),
            offset: None,
        }
    }

    /// Calculate the SQL offset
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or_else(|| (self.page - 1) * self.per_page)
    }

    /// Calculate the SQL limit
    pub fn limit(&self) -> i64 {
        self.per_page
    }
}

/// Paginated collection response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    /// HAL type
    #[serde(rename = "_type")]
    pub hal_type: String,

    /// Total count of items
    pub total: i64,

    /// Number of items in this page
    pub count: i64,

    /// Current page size
    pub page_size: i64,

    /// Current offset
    pub offset: i64,

    /// HAL links
    #[serde(rename = "_links")]
    pub links: PaginationLinks,

    /// Embedded items
    #[serde(rename = "_embedded")]
    pub embedded: PaginatedEmbedded<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationLinks {
    #[serde(rename = "self")]
    pub self_link: LinkObject,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "jumpTo")]
    pub jump_to: Option<LinkObject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "changeSize")]
    pub change_size: Option<LinkObject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "previousByOffset")]
    pub previous: Option<LinkObject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "nextByOffset")]
    pub next: Option<LinkObject>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkObject {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedEmbedded<T> {
    pub elements: Vec<T>,
}

impl<T> PaginatedResponse<T> {
    pub fn new(
        items: Vec<T>,
        total: i64,
        params: &PaginationParams,
        base_url: &str,
        element_type: &str,
    ) -> Self {
        let offset = params.offset();
        let count = items.len() as i64;
        let page_size = params.per_page;

        let self_href = format!("{}?offset={}&pageSize={}", base_url, offset, page_size);

        let previous = if offset > 0 {
            let prev_offset = (offset - page_size).max(0);
            Some(LinkObject {
                href: format!("{}?offset={}&pageSize={}", base_url, prev_offset, page_size),
                title: None,
            })
        } else {
            None
        };

        let next = if offset + count < total {
            let next_offset = offset + page_size;
            Some(LinkObject {
                href: format!("{}?offset={}&pageSize={}", base_url, next_offset, page_size),
                title: None,
            })
        } else {
            None
        };

        Self {
            hal_type: "Collection".to_string(),
            total,
            count,
            page_size,
            offset,
            links: PaginationLinks {
                self_link: LinkObject {
                    href: self_href,
                    title: None,
                },
                jump_to: Some(LinkObject {
                    href: format!("{}?offset={{offset}}&pageSize={}", base_url, page_size),
                    title: None,
                }),
                change_size: Some(LinkObject {
                    href: format!("{}?offset={}&pageSize={{size}}", base_url, offset),
                    title: None,
                }),
                previous,
                next,
            },
            embedded: PaginatedEmbedded { elements: items },
        }
    }

    /// Get the collection type name for the HAL response
    pub fn collection_type(element_type: &str) -> String {
        format!("{}Collection", element_type)
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

/// Sort parameter
#[derive(Debug, Clone, Deserialize)]
pub struct SortParam {
    pub field: String,
    pub direction: SortDirection,
}

impl SortParam {
    pub fn parse(sort_string: &str) -> Vec<Self> {
        sort_string
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() {
                    return None;
                }

                let (field, direction) = if let Some(field) = part.strip_suffix(":desc") {
                    (field.to_string(), SortDirection::Desc)
                } else if let Some(field) = part.strip_suffix(":asc") {
                    (field.to_string(), SortDirection::Asc)
                } else {
                    (part.to_string(), SortDirection::Asc)
                };

                Some(SortParam { field, direction })
            })
            .collect()
    }
}

/// Filter parameters for API queries
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FilterParams {
    /// JSON-encoded filters (OpenProject API v3 format)
    pub filters: Option<String>,
}

impl FilterParams {
    /// Parse OpenProject API v3 filter format
    /// Format: [{"field":{"operator":"value"}}]
    pub fn parse(&self) -> Vec<Filter> {
        // TODO: Implement proper parsing
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

impl FilterOperator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" | "==" => Some(Self::Equals),
            "!" | "!=" | "<>" => Some(Self::NotEquals),
            "~" => Some(Self::Contains),
            "!~" => Some(Self::NotContains),
            ">" | ">" => Some(Self::GreaterThan),
            "<" | "<" => Some(Self::LessThan),
            ">=" | "≥" => Some(Self::GreaterOrEqual),
            "<=" | "≤" => Some(Self::LessOrEqual),
            "|" => Some(Self::In),
            "!|" => Some(Self::NotIn),
            "!*" => Some(Self::IsNull),
            "*" => Some(Self::IsNotNull),
            _ => None,
        }
    }
}
