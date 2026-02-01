//! Query Filters
//!
//! Mirrors: app/models/queries/filters/*
//!
//! Filters are the core building blocks of queries in OpenProject.
//! Each filter represents a condition on a specific attribute.

use op_core::traits::Id;
use std::collections::HashSet;

/// Filter operators that can be applied to values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterOperator {
    /// Equals (=)
    Equals,
    /// Not equals (!)
    NotEquals,
    /// Contains (~)
    Contains,
    /// Does not contain (!~)
    NotContains,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Between two values (<>d)
    Between,
    /// Is null/empty (*)
    IsNull,
    /// Is not null/empty (!*)
    IsNotNull,
    /// Today (t)
    Today,
    /// This week (w)
    ThisWeek,
    /// Days ago (t-)
    DaysAgo(i32),
    /// Days from now (t+)
    DaysFromNow(i32),
    /// Less than days ago (<t-)
    LessThanDaysAgo(i32),
    /// More than days ago (>t-)
    MoreThanDaysAgo(i32),
    /// Less than days from now (<t+)
    LessThanDaysFromNow(i32),
    /// More than days from now (>t+)
    MoreThanDaysFromNow(i32),
    /// Current user (me)
    CurrentUser,
}

impl FilterOperator {
    /// Parse operator from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" => Some(Self::Equals),
            "!" => Some(Self::NotEquals),
            "~" => Some(Self::Contains),
            "!~" => Some(Self::NotContains),
            "**" => Some(Self::StartsWith),
            "*~" => Some(Self::EndsWith),
            ">" => Some(Self::GreaterThan),
            ">=" => Some(Self::GreaterThanOrEqual),
            "<" => Some(Self::LessThan),
            "<=" => Some(Self::LessThanOrEqual),
            "<>d" => Some(Self::Between),
            "*" | "o" => Some(Self::IsNull),
            "!*" | "c" => Some(Self::IsNotNull),
            "t" => Some(Self::Today),
            "w" => Some(Self::ThisWeek),
            s if s.starts_with("t-") => s[2..].parse().ok().map(Self::DaysAgo),
            s if s.starts_with("t+") => s[2..].parse().ok().map(Self::DaysFromNow),
            s if s.starts_with("<t-") => s[3..].parse().ok().map(Self::LessThanDaysAgo),
            s if s.starts_with(">t-") => s[3..].parse().ok().map(Self::MoreThanDaysAgo),
            s if s.starts_with("<t+") => s[3..].parse().ok().map(Self::LessThanDaysFromNow),
            s if s.starts_with(">t+") => s[3..].parse().ok().map(Self::MoreThanDaysFromNow),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::Equals => "=".to_string(),
            Self::NotEquals => "!".to_string(),
            Self::Contains => "~".to_string(),
            Self::NotContains => "!~".to_string(),
            Self::StartsWith => "**".to_string(),
            Self::EndsWith => "*~".to_string(),
            Self::GreaterThan => ">".to_string(),
            Self::GreaterThanOrEqual => ">=".to_string(),
            Self::LessThan => "<".to_string(),
            Self::LessThanOrEqual => "<=".to_string(),
            Self::Between => "<>d".to_string(),
            Self::IsNull => "*".to_string(),
            Self::IsNotNull => "!*".to_string(),
            Self::Today => "t".to_string(),
            Self::ThisWeek => "w".to_string(),
            Self::DaysAgo(n) => format!("t-{}", n),
            Self::DaysFromNow(n) => format!("t+{}", n),
            Self::LessThanDaysAgo(n) => format!("<t-{}", n),
            Self::MoreThanDaysAgo(n) => format!(">t-{}", n),
            Self::LessThanDaysFromNow(n) => format!("<t+{}", n),
            Self::MoreThanDaysFromNow(n) => format!(">t+{}", n),
            Self::CurrentUser => "=".to_string(), // Special handling needed
        }
    }

    /// Check if this operator requires values
    pub fn requires_values(&self) -> bool {
        !matches!(
            self,
            Self::IsNull
                | Self::IsNotNull
                | Self::Today
                | Self::ThisWeek
                | Self::CurrentUser
        )
    }
}

/// Filter value types
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// Single integer ID
    Id(Id),
    /// List of integer IDs
    Ids(Vec<Id>),
    /// Single string value
    String(String),
    /// List of string values
    Strings(Vec<String>),
    /// Boolean value
    Bool(bool),
    /// Date string (YYYY-MM-DD)
    Date(String),
    /// Date range
    DateRange { from: String, to: String },
    /// Numeric value
    Number(f64),
    /// Special "me" value for current user
    Me,
    /// No value (for null checks)
    None,
}

impl FilterValue {
    /// Create from a list of IDs
    pub fn from_ids(ids: Vec<Id>) -> Self {
        if ids.len() == 1 {
            Self::Id(ids[0])
        } else {
            Self::Ids(ids)
        }
    }

    /// Create from a list of strings
    pub fn from_strings(strings: Vec<String>) -> Self {
        if strings.len() == 1 {
            Self::String(strings[0].clone())
        } else {
            Self::Strings(strings)
        }
    }

    /// Get as list of IDs
    pub fn as_ids(&self) -> Vec<Id> {
        match self {
            Self::Id(id) => vec![*id],
            Self::Ids(ids) => ids.clone(),
            _ => vec![],
        }
    }

    /// Get as list of strings
    pub fn as_strings(&self) -> Vec<String> {
        match self {
            Self::String(s) => vec![s.clone()],
            Self::Strings(ss) => ss.clone(),
            Self::Id(id) => vec![id.to_string()],
            Self::Ids(ids) => ids.iter().map(|id| id.to_string()).collect(),
            _ => vec![],
        }
    }
}

/// A single filter condition
#[derive(Debug, Clone)]
pub struct Filter {
    /// The attribute being filtered (e.g., "status_id", "assigned_to_id")
    pub attribute: String,
    /// The operator to apply
    pub operator: FilterOperator,
    /// The values to filter by
    pub values: FilterValue,
}

impl Filter {
    /// Create a new filter
    pub fn new(attribute: impl Into<String>, operator: FilterOperator, values: FilterValue) -> Self {
        Self {
            attribute: attribute.into(),
            operator,
            values,
        }
    }

    /// Create an equals filter
    pub fn equals(attribute: impl Into<String>, values: FilterValue) -> Self {
        Self::new(attribute, FilterOperator::Equals, values)
    }

    /// Create a not equals filter
    pub fn not_equals(attribute: impl Into<String>, values: FilterValue) -> Self {
        Self::new(attribute, FilterOperator::NotEquals, values)
    }

    /// Create a contains filter
    pub fn contains(attribute: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            attribute,
            FilterOperator::Contains,
            FilterValue::String(value.into()),
        )
    }

    /// Create an is null filter
    pub fn is_null(attribute: impl Into<String>) -> Self {
        Self::new(attribute, FilterOperator::IsNull, FilterValue::None)
    }

    /// Create an is not null filter
    pub fn is_not_null(attribute: impl Into<String>) -> Self {
        Self::new(attribute, FilterOperator::IsNotNull, FilterValue::None)
    }

    /// Check if this filter is valid
    pub fn is_valid(&self) -> bool {
        if self.attribute.is_empty() {
            return false;
        }

        if self.operator.requires_values() {
            !matches!(self.values, FilterValue::None)
        } else {
            true
        }
    }
}

/// Known filter attributes for work packages
pub mod attributes {
    pub const STATUS_ID: &str = "status_id";
    pub const PROJECT_ID: &str = "project_id";
    pub const TYPE_ID: &str = "type_id";
    pub const PRIORITY_ID: &str = "priority_id";
    pub const ASSIGNED_TO_ID: &str = "assigned_to_id";
    pub const AUTHOR_ID: &str = "author_id";
    pub const VERSION_ID: &str = "version_id";
    pub const CATEGORY_ID: &str = "category_id";
    pub const SUBJECT: &str = "subject";
    pub const DESCRIPTION: &str = "description";
    pub const START_DATE: &str = "start_date";
    pub const DUE_DATE: &str = "due_date";
    pub const ESTIMATED_HOURS: &str = "estimated_hours";
    pub const DONE_RATIO: &str = "done_ratio";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
    pub const PARENT_ID: &str = "parent_id";
    pub const SUBPROJECT_ID: &str = "subproject_id";
    pub const WATCHER_ID: &str = "watcher_id";
    pub const RESPONSIBLE_ID: &str = "responsible_id";
    pub const MANUAL_SORT: &str = "manual_sort";
    pub const ID: &str = "id";
}

/// Filter set - a collection of filters with AND semantics
#[derive(Debug, Clone, Default)]
pub struct FilterSet {
    filters: Vec<Filter>,
}

impl FilterSet {
    /// Create a new empty filter set
    pub fn new() -> Self {
        Self { filters: vec![] }
    }

    /// Add a filter to the set
    pub fn add(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);
        self
    }

    /// Add a filter and return self (builder pattern)
    pub fn with(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Get all filters
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Check if any filters are set
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Get number of filters
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Get filters for a specific attribute
    pub fn filters_for(&self, attribute: &str) -> Vec<&Filter> {
        self.filters
            .iter()
            .filter(|f| f.attribute == attribute)
            .collect()
    }

    /// Check if a specific attribute is being filtered
    pub fn has_filter_for(&self, attribute: &str) -> bool {
        self.filters.iter().any(|f| f.attribute == attribute)
    }

    /// Remove filters for a specific attribute
    pub fn remove_filters_for(&mut self, attribute: &str) {
        self.filters.retain(|f| f.attribute != attribute);
    }

    /// Get all filtered attribute names
    pub fn filtered_attributes(&self) -> HashSet<&str> {
        self.filters.iter().map(|f| f.attribute.as_str()).collect()
    }

    /// Validate all filters
    pub fn is_valid(&self) -> bool {
        self.filters.iter().all(|f| f.is_valid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_operator_parsing() {
        assert_eq!(
            FilterOperator::from_str("="),
            Some(FilterOperator::Equals)
        );
        assert_eq!(
            FilterOperator::from_str("!"),
            Some(FilterOperator::NotEquals)
        );
        assert_eq!(
            FilterOperator::from_str("~"),
            Some(FilterOperator::Contains)
        );
        assert_eq!(
            FilterOperator::from_str("*"),
            Some(FilterOperator::IsNull)
        );
        assert_eq!(
            FilterOperator::from_str("t-5"),
            Some(FilterOperator::DaysAgo(5))
        );
        assert_eq!(
            FilterOperator::from_str(">t+10"),
            Some(FilterOperator::MoreThanDaysFromNow(10))
        );
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter::equals("status_id", FilterValue::Id(1));
        assert_eq!(filter.attribute, "status_id");
        assert_eq!(filter.operator, FilterOperator::Equals);
        assert!(filter.is_valid());
    }

    #[test]
    fn test_filter_contains() {
        let filter = Filter::contains("subject", "bug");
        assert_eq!(filter.attribute, "subject");
        assert_eq!(filter.operator, FilterOperator::Contains);
        assert!(filter.is_valid());
    }

    #[test]
    fn test_filter_is_null() {
        let filter = Filter::is_null("assigned_to_id");
        assert_eq!(filter.attribute, "assigned_to_id");
        assert_eq!(filter.operator, FilterOperator::IsNull);
        assert!(filter.is_valid());
    }

    #[test]
    fn test_filter_set() {
        let filters = FilterSet::new()
            .with(Filter::equals("status_id", FilterValue::Id(1)))
            .with(Filter::equals("project_id", FilterValue::Id(2)));

        assert_eq!(filters.len(), 2);
        assert!(filters.has_filter_for("status_id"));
        assert!(filters.has_filter_for("project_id"));
        assert!(!filters.has_filter_for("type_id"));
    }

    #[test]
    fn test_filter_value_ids() {
        let single = FilterValue::from_ids(vec![1]);
        assert!(matches!(single, FilterValue::Id(1)));

        let multiple = FilterValue::from_ids(vec![1, 2, 3]);
        assert!(matches!(multiple, FilterValue::Ids(_)));
        assert_eq!(multiple.as_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn test_filter_operator_requires_values() {
        assert!(FilterOperator::Equals.requires_values());
        assert!(FilterOperator::Contains.requires_values());
        assert!(!FilterOperator::IsNull.requires_values());
        assert!(!FilterOperator::Today.requires_values());
    }
}
