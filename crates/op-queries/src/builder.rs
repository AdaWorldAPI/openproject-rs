//! Query Builder
//!
//! Provides a fluent API for constructing queries with filters, sorts, and columns.

use op_core::traits::Id;

use crate::columns::{Column, ColumnSet};
use crate::filters::{Filter, FilterOperator, FilterSet, FilterValue};
use crate::query::{DisplayRepresentation, GroupBy, Query, QueryVisibility};
use crate::sorts::{SortCriterion, SortDirection, SortOrder};

/// Builder for constructing queries fluently
#[derive(Debug, Default)]
pub struct QueryBuilder {
    name: String,
    project_id: Option<Id>,
    user_id: Option<Id>,
    visibility: QueryVisibility,
    starred: bool,
    display: DisplayRepresentation,
    filters: FilterSet,
    sorts: SortOrder,
    columns: ColumnSet,
    group_by: GroupBy,
    include_subprojects: bool,
    show_hierarchies: bool,
    show_sums: bool,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            name: "New Query".to_string(),
            include_subprojects: true,
            show_hierarchies: true,
            columns: ColumnSet::default_work_package(),
            sorts: crate::sorts::default_work_package_sort(),
            ..Default::default()
        }
    }

    /// Set the query name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the project
    pub fn project(mut self, project_id: Id) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the user/owner
    pub fn user(mut self, user_id: Id) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Make the query public
    pub fn public(mut self) -> Self {
        self.visibility = QueryVisibility::Public;
        self
    }

    /// Make the query private
    pub fn private(mut self) -> Self {
        self.visibility = QueryVisibility::Private;
        self
    }

    /// Make the query global (admin only)
    pub fn global(mut self) -> Self {
        self.visibility = QueryVisibility::Global;
        self
    }

    /// Star the query
    pub fn starred(mut self) -> Self {
        self.starred = true;
        self
    }

    /// Set display as list
    pub fn list_view(mut self) -> Self {
        self.display = DisplayRepresentation::List;
        self
    }

    /// Set display as board
    pub fn board_view(mut self) -> Self {
        self.display = DisplayRepresentation::Board;
        self
    }

    /// Set display as gantt
    pub fn gantt_view(mut self) -> Self {
        self.display = DisplayRepresentation::Gantt;
        self
    }

    /// Set display as calendar
    pub fn calendar_view(mut self) -> Self {
        self.display = DisplayRepresentation::Calendar;
        self
    }

    // Filter methods

    /// Add a raw filter
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.add(filter);
        self
    }

    /// Filter by status IDs
    pub fn status(mut self, status_ids: impl Into<Vec<Id>>) -> Self {
        let ids = status_ids.into();
        self.filters.add(Filter::equals(
            "status_id",
            FilterValue::from_ids(ids),
        ));
        self
    }

    /// Filter by open status
    pub fn open(mut self) -> Self {
        self.filters.add(Filter::new(
            "status_id",
            FilterOperator::IsNotNull,
            FilterValue::None,
        ));
        // In real implementation, this would filter by status.is_closed = false
        self
    }

    /// Filter by closed status
    pub fn closed(mut self) -> Self {
        self.filters.add(Filter::new(
            "status_id",
            FilterOperator::IsNull,
            FilterValue::None,
        ));
        // In real implementation, this would filter by status.is_closed = true
        self
    }

    /// Filter by project ID
    pub fn in_project(mut self, project_id: Id) -> Self {
        self.project_id = Some(project_id);
        self.filters.add(Filter::equals(
            "project_id",
            FilterValue::Id(project_id),
        ));
        self
    }

    /// Filter by type IDs
    pub fn type_ids(mut self, type_ids: impl Into<Vec<Id>>) -> Self {
        let ids = type_ids.into();
        self.filters.add(Filter::equals(
            "type_id",
            FilterValue::from_ids(ids),
        ));
        self
    }

    /// Filter by assignee IDs
    pub fn assigned_to(mut self, user_ids: impl Into<Vec<Id>>) -> Self {
        let ids = user_ids.into();
        self.filters.add(Filter::equals(
            "assigned_to_id",
            FilterValue::from_ids(ids),
        ));
        self
    }

    /// Filter by assigned to current user
    pub fn assigned_to_me(mut self) -> Self {
        self.filters.add(Filter::new(
            "assigned_to_id",
            FilterOperator::CurrentUser,
            FilterValue::Me,
        ));
        self
    }

    /// Filter unassigned
    pub fn unassigned(mut self) -> Self {
        self.filters.add(Filter::is_null("assigned_to_id"));
        self
    }

    /// Filter by author IDs
    pub fn authored_by(mut self, user_ids: impl Into<Vec<Id>>) -> Self {
        let ids = user_ids.into();
        self.filters.add(Filter::equals(
            "author_id",
            FilterValue::from_ids(ids),
        ));
        self
    }

    /// Filter by created by current user
    pub fn created_by_me(mut self) -> Self {
        self.filters.add(Filter::new(
            "author_id",
            FilterOperator::CurrentUser,
            FilterValue::Me,
        ));
        self
    }

    /// Filter by priority IDs
    pub fn priority(mut self, priority_ids: impl Into<Vec<Id>>) -> Self {
        let ids = priority_ids.into();
        self.filters.add(Filter::equals(
            "priority_id",
            FilterValue::from_ids(ids),
        ));
        self
    }

    /// Filter by version IDs
    pub fn version(mut self, version_ids: impl Into<Vec<Id>>) -> Self {
        let ids = version_ids.into();
        self.filters.add(Filter::equals(
            "version_id",
            FilterValue::from_ids(ids),
        ));
        self
    }

    /// Filter by subject containing text
    pub fn subject_contains(mut self, text: impl Into<String>) -> Self {
        self.filters.add(Filter::contains("subject", text));
        self
    }

    /// Filter by due date today
    pub fn due_today(mut self) -> Self {
        self.filters.add(Filter::new(
            "due_date",
            FilterOperator::Today,
            FilterValue::None,
        ));
        self
    }

    /// Filter by due this week
    pub fn due_this_week(mut self) -> Self {
        self.filters.add(Filter::new(
            "due_date",
            FilterOperator::ThisWeek,
            FilterValue::None,
        ));
        self
    }

    /// Filter overdue (due date in the past)
    pub fn overdue(mut self) -> Self {
        self.filters.add(Filter::new(
            "due_date",
            FilterOperator::LessThanDaysAgo(0),
            FilterValue::None,
        ));
        self
    }

    /// Filter by parent work package
    pub fn parent(mut self, parent_id: Id) -> Self {
        self.filters.add(Filter::equals(
            "parent_id",
            FilterValue::Id(parent_id),
        ));
        self
    }

    /// Filter for root work packages (no parent)
    pub fn roots_only(mut self) -> Self {
        self.filters.add(Filter::is_null("parent_id"));
        self
    }

    // Sort methods

    /// Set sort order
    pub fn sort(mut self, sorts: SortOrder) -> Self {
        self.sorts = sorts;
        self
    }

    /// Sort by attribute ascending
    pub fn sort_by_asc(mut self, attribute: impl Into<String>) -> Self {
        self.sorts = SortOrder::by_asc(attribute);
        self
    }

    /// Sort by attribute descending
    pub fn sort_by_desc(mut self, attribute: impl Into<String>) -> Self {
        self.sorts = SortOrder::by_desc(attribute);
        self
    }

    /// Add secondary sort ascending
    pub fn then_by_asc(mut self, attribute: impl Into<String>) -> Self {
        self.sorts.add(SortCriterion::asc(attribute));
        self
    }

    /// Add secondary sort descending
    pub fn then_by_desc(mut self, attribute: impl Into<String>) -> Self {
        self.sorts.add(SortCriterion::desc(attribute));
        self
    }

    /// Sort by ID (default)
    pub fn sort_by_id(mut self) -> Self {
        self.sorts = SortOrder::by_desc("id");
        self
    }

    /// Sort by updated date (most recent first)
    pub fn sort_by_updated(mut self) -> Self {
        self.sorts = SortOrder::by_desc("updated_at");
        self
    }

    /// Sort by created date (most recent first)
    pub fn sort_by_created(mut self) -> Self {
        self.sorts = SortOrder::by_desc("created_at");
        self
    }

    /// Sort by priority (highest first)
    pub fn sort_by_priority(mut self) -> Self {
        self.sorts = SortOrder::by_asc("priority");
        self
    }

    /// Sort by due date (earliest first)
    pub fn sort_by_due_date(mut self) -> Self {
        self.sorts = SortOrder::by_asc("due_date");
        self
    }

    // Column methods

    /// Set columns
    pub fn columns(mut self, columns: ColumnSet) -> Self {
        self.columns = columns;
        self
    }

    /// Add a column
    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.add(column);
        self
    }

    /// Add a column by name
    pub fn with_column(mut self, name: impl Into<String>) -> Self {
        self.columns.add(Column::property(name));
        self
    }

    /// Use default columns
    pub fn default_columns(mut self) -> Self {
        self.columns = ColumnSet::default_work_package();
        self
    }

    /// Clear columns
    pub fn no_columns(mut self) -> Self {
        self.columns = ColumnSet::new();
        self
    }

    // Grouping methods

    /// Group by attribute
    pub fn group_by(mut self, attribute: impl Into<String>) -> Self {
        self.group_by = GroupBy::by(attribute);
        self
    }

    /// Group by status
    pub fn group_by_status(mut self) -> Self {
        self.group_by = GroupBy::by("status");
        self
    }

    /// Group by type
    pub fn group_by_type(mut self) -> Self {
        self.group_by = GroupBy::by("type");
        self
    }

    /// Group by assignee
    pub fn group_by_assignee(mut self) -> Self {
        self.group_by = GroupBy::by("assigned_to");
        self
    }

    /// Group by priority
    pub fn group_by_priority(mut self) -> Self {
        self.group_by = GroupBy::by("priority");
        self
    }

    /// No grouping
    pub fn ungrouped(mut self) -> Self {
        self.group_by = GroupBy::none();
        self
    }

    // Display options

    /// Include subprojects
    pub fn include_subprojects(mut self) -> Self {
        self.include_subprojects = true;
        self
    }

    /// Exclude subprojects
    pub fn exclude_subprojects(mut self) -> Self {
        self.include_subprojects = false;
        self
    }

    /// Show hierarchies
    pub fn with_hierarchies(mut self) -> Self {
        self.show_hierarchies = true;
        self
    }

    /// Hide hierarchies (flat list)
    pub fn flat(mut self) -> Self {
        self.show_hierarchies = false;
        self
    }

    /// Show sums row
    pub fn with_sums(mut self) -> Self {
        self.show_sums = true;
        self
    }

    /// Build the query
    pub fn build(self) -> Query {
        Query {
            id: None,
            name: self.name,
            user_id: self.user_id,
            project_id: self.project_id,
            visibility: self.visibility,
            starred: self.starred,
            display: self.display,
            filters: self.filters,
            sorts: self.sorts,
            columns: self.columns,
            group_by: self.group_by,
            highlighting: Default::default(),
            timeline_zoom_level: Default::default(),
            show_timeline: self.display == DisplayRepresentation::Gantt,
            include_subprojects: self.include_subprojects,
            show_hierarchies: self.show_hierarchies,
            show_sums: self.show_sums,
        }
    }
}

/// Helper functions for common queries
pub mod presets {
    use super::*;

    /// Create a query for "My work packages"
    pub fn my_work_packages() -> Query {
        QueryBuilder::new()
            .name("My work packages")
            .assigned_to_me()
            .open()
            .sort_by_updated()
            .build()
    }

    /// Create a query for "Created by me"
    pub fn created_by_me() -> Query {
        QueryBuilder::new()
            .name("Created by me")
            .created_by_me()
            .sort_by_updated()
            .build()
    }

    /// Create a query for "Watched by me"
    pub fn watched_by_me() -> Query {
        QueryBuilder::new()
            .name("Watched")
            .filter(Filter::new(
                "watcher_id",
                FilterOperator::CurrentUser,
                FilterValue::Me,
            ))
            .sort_by_updated()
            .build()
    }

    /// Create a query for all open work packages
    pub fn all_open() -> Query {
        QueryBuilder::new()
            .name("All open")
            .open()
            .sort_by_updated()
            .build()
    }

    /// Create a query for recently updated
    pub fn recently_updated() -> Query {
        QueryBuilder::new()
            .name("Recently updated")
            .sort_by_updated()
            .build()
    }

    /// Create a basic board query grouped by status
    pub fn basic_board() -> Query {
        QueryBuilder::new()
            .name("Basic board")
            .board_view()
            .group_by_status()
            .open()
            .build()
    }

    /// Create a Gantt chart query
    pub fn gantt_chart() -> Query {
        QueryBuilder::new()
            .name("Gantt chart")
            .gantt_view()
            .sort_by_asc("start_date")
            .then_by_asc("due_date")
            .build()
    }

    /// Create a query for overdue work packages
    pub fn overdue() -> Query {
        QueryBuilder::new()
            .name("Overdue")
            .overdue()
            .open()
            .sort_by_asc("due_date")
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let query = QueryBuilder::new()
            .name("Test Query")
            .build();

        assert_eq!(query.name, "Test Query");
        assert!(query.id.is_none());
    }

    #[test]
    fn test_builder_with_project() {
        let query = QueryBuilder::new()
            .name("Project Query")
            .project(42)
            .build();

        assert_eq!(query.project_id, Some(42));
    }

    #[test]
    fn test_builder_with_filters() {
        let query = QueryBuilder::new()
            .name("Filtered")
            .status(vec![1, 2])
            .assigned_to(vec![5])
            .subject_contains("bug")
            .build();

        assert!(query.has_filters());
        assert_eq!(query.filters.len(), 3);
    }

    #[test]
    fn test_builder_with_sorts() {
        let query = QueryBuilder::new()
            .name("Sorted")
            .sort_by_priority()
            .then_by_asc("id")
            .build();

        assert!(query.has_custom_sort());
        assert_eq!(query.sorts.len(), 2);
    }

    #[test]
    fn test_builder_with_grouping() {
        let query = QueryBuilder::new()
            .name("Grouped")
            .group_by_status()
            .build();

        assert!(query.is_grouped());
        assert_eq!(query.group_by.attribute, Some("status".to_string()));
    }

    #[test]
    fn test_builder_board_view() {
        let query = QueryBuilder::new()
            .name("Board")
            .board_view()
            .group_by_status()
            .build();

        assert_eq!(query.display, DisplayRepresentation::Board);
    }

    #[test]
    fn test_builder_gantt_view() {
        let query = QueryBuilder::new()
            .name("Gantt")
            .gantt_view()
            .build();

        assert_eq!(query.display, DisplayRepresentation::Gantt);
        assert!(query.show_timeline);
    }

    #[test]
    fn test_preset_my_work_packages() {
        let query = presets::my_work_packages();
        assert_eq!(query.name, "My work packages");
        assert!(query.has_filters());
    }

    #[test]
    fn test_preset_basic_board() {
        let query = presets::basic_board();
        assert_eq!(query.display, DisplayRepresentation::Board);
        assert!(query.is_grouped());
    }

    #[test]
    fn test_builder_chaining() {
        let query = QueryBuilder::new()
            .name("Complex Query")
            .project(1)
            .user(2)
            .public()
            .starred()
            .board_view()
            .status(vec![1, 2, 3])
            .assigned_to_me()
            .sort_by_priority()
            .group_by_status()
            .include_subprojects()
            .with_hierarchies()
            .with_sums()
            .build();

        assert_eq!(query.project_id, Some(1));
        assert_eq!(query.user_id, Some(2));
        assert!(query.is_public());
        assert!(query.starred);
        assert_eq!(query.display, DisplayRepresentation::Board);
        assert!(query.has_filters());
        assert!(query.is_grouped());
        assert!(query.include_subprojects);
        assert!(query.show_hierarchies);
        assert!(query.show_sums);
    }
}
