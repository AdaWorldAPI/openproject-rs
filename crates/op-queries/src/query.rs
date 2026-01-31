//! Query Model
//!
//! Mirrors: app/models/query.rb
//!
//! A Query is a saved configuration for filtering, sorting, and displaying
//! work packages. Queries can be saved as personal or shared views.

use op_core::traits::Id;

use crate::columns::ColumnSet;
use crate::filters::{Filter, FilterSet};
use crate::sorts::SortOrder;

/// Query visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryVisibility {
    /// Only visible to the owner
    #[default]
    Private,
    /// Visible to all users in the project
    Public,
    /// Visible globally (admin only)
    Global,
}

/// Display representation for the query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayRepresentation {
    /// List/table view
    #[default]
    List,
    /// Card/board view
    Board,
    /// Gantt chart view
    Gantt,
    /// Calendar view
    Calendar,
    /// Team planner view
    TeamPlanner,
}

impl DisplayRepresentation {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "list" | "table" => Some(Self::List),
            "board" | "cards" => Some(Self::Board),
            "gantt" => Some(Self::Gantt),
            "calendar" => Some(Self::Calendar),
            "team_planner" | "teamplanner" => Some(Self::TeamPlanner),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Board => "board",
            Self::Gantt => "gantt",
            Self::Calendar => "calendar",
            Self::TeamPlanner => "team_planner",
        }
    }
}

/// Timeline zoom level for Gantt view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineZoomLevel {
    /// Days
    Days,
    /// Weeks
    #[default]
    Weeks,
    /// Months
    Months,
    /// Quarters
    Quarters,
    /// Years
    Years,
    /// Auto-fit
    Auto,
}

/// Grouping configuration
#[derive(Debug, Clone, Default)]
pub struct GroupBy {
    /// Attribute to group by
    pub attribute: Option<String>,
    /// Collapse groups by default
    pub collapsed: bool,
}

impl GroupBy {
    /// Create a new grouping
    pub fn by(attribute: impl Into<String>) -> Self {
        Self {
            attribute: Some(attribute.into()),
            collapsed: false,
        }
    }

    /// Create with collapsed groups
    pub fn by_collapsed(attribute: impl Into<String>) -> Self {
        Self {
            attribute: Some(attribute.into()),
            collapsed: true,
        }
    }

    /// No grouping
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if grouping is enabled
    pub fn is_grouped(&self) -> bool {
        self.attribute.is_some()
    }
}

/// Highlighting configuration
#[derive(Debug, Clone, Default)]
pub struct Highlighting {
    /// Highlighting mode
    pub mode: HighlightingMode,
    /// Specific attributes to highlight (for inline mode)
    pub highlighted_attributes: Vec<String>,
}

/// Highlighting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HighlightingMode {
    /// No highlighting
    #[default]
    None,
    /// Inline attribute highlighting
    Inline,
    /// Status-based row highlighting
    Status,
    /// Priority-based row highlighting
    Priority,
    /// Type-based row highlighting
    Type,
}

/// A saved query
#[derive(Debug, Clone)]
pub struct Query {
    /// Query ID (None for unsaved queries)
    pub id: Option<Id>,
    /// Query name
    pub name: String,
    /// User who created the query
    pub user_id: Option<Id>,
    /// Project the query belongs to (None for global)
    pub project_id: Option<Id>,
    /// Visibility setting
    pub visibility: QueryVisibility,
    /// Whether this is a starred/favorite query
    pub starred: bool,
    /// Display representation
    pub display: DisplayRepresentation,
    /// Filter configuration
    pub filters: FilterSet,
    /// Sort order configuration
    pub sorts: SortOrder,
    /// Column configuration
    pub columns: ColumnSet,
    /// Group by configuration
    pub group_by: GroupBy,
    /// Highlighting configuration
    pub highlighting: Highlighting,
    /// Timeline/Gantt zoom level
    pub timeline_zoom_level: TimelineZoomLevel,
    /// Show timeline (Gantt) view
    pub show_timeline: bool,
    /// Include subprojects in results
    pub include_subprojects: bool,
    /// Show hierarchies (work package tree)
    pub show_hierarchies: bool,
    /// Show sums row
    pub show_sums: bool,
}

impl Query {
    /// Create a new query with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            user_id: None,
            project_id: None,
            visibility: QueryVisibility::Private,
            starred: false,
            display: DisplayRepresentation::List,
            filters: FilterSet::new(),
            sorts: crate::sorts::default_work_package_sort(),
            columns: ColumnSet::default_work_package(),
            group_by: GroupBy::none(),
            highlighting: Highlighting::default(),
            timeline_zoom_level: TimelineZoomLevel::default(),
            show_timeline: false,
            include_subprojects: true,
            show_hierarchies: true,
            show_sums: false,
        }
    }

    /// Create a new query for a specific project
    pub fn for_project(name: impl Into<String>, project_id: Id) -> Self {
        let mut query = Self::new(name);
        query.project_id = Some(project_id);
        query
    }

    /// Set the user who owns this query
    pub fn with_user(mut self, user_id: Id) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set visibility
    pub fn with_visibility(mut self, visibility: QueryVisibility) -> Self {
        self.visibility = visibility;
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

    /// Star the query
    pub fn starred(mut self) -> Self {
        self.starred = true;
        self
    }

    /// Set display representation
    pub fn with_display(mut self, display: DisplayRepresentation) -> Self {
        self.display = display;
        self
    }

    /// Set as board view
    pub fn board(mut self) -> Self {
        self.display = DisplayRepresentation::Board;
        self
    }

    /// Set as Gantt view
    pub fn gantt(mut self) -> Self {
        self.display = DisplayRepresentation::Gantt;
        self.show_timeline = true;
        self
    }

    /// Add a filter
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.add(filter);
        self
    }

    /// Set filters
    pub fn with_filters(mut self, filters: FilterSet) -> Self {
        self.filters = filters;
        self
    }

    /// Set sort order
    pub fn with_sorts(mut self, sorts: SortOrder) -> Self {
        self.sorts = sorts;
        self
    }

    /// Set columns
    pub fn with_columns(mut self, columns: ColumnSet) -> Self {
        self.columns = columns;
        self
    }

    /// Set group by
    pub fn with_group_by(mut self, group_by: GroupBy) -> Self {
        self.group_by = group_by;
        self
    }

    /// Group by an attribute
    pub fn grouped_by(mut self, attribute: impl Into<String>) -> Self {
        self.group_by = GroupBy::by(attribute);
        self
    }

    /// Include subprojects
    pub fn with_subprojects(mut self, include: bool) -> Self {
        self.include_subprojects = include;
        self
    }

    /// Show hierarchies
    pub fn with_hierarchies(mut self, show: bool) -> Self {
        self.show_hierarchies = show;
        self
    }

    /// Show sums
    pub fn with_sums(mut self) -> Self {
        self.show_sums = true;
        self
    }

    /// Check if this query is saved
    pub fn is_saved(&self) -> bool {
        self.id.is_some()
    }

    /// Check if this is a global query
    pub fn is_global(&self) -> bool {
        self.project_id.is_none()
    }

    /// Check if this is public
    pub fn is_public(&self) -> bool {
        self.visibility == QueryVisibility::Public
    }

    /// Check if this query has any filters
    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    /// Check if this query has custom sorting
    pub fn has_custom_sort(&self) -> bool {
        !self.sorts.is_empty()
    }

    /// Check if results are grouped
    pub fn is_grouped(&self) -> bool {
        self.group_by.is_grouped()
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new("Unnamed query")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::FilterValue;

    #[test]
    fn test_query_creation() {
        let query = Query::new("My Query");
        assert_eq!(query.name, "My Query");
        assert!(query.id.is_none());
        assert!(!query.is_saved());
        assert!(query.is_global());
    }

    #[test]
    fn test_query_for_project() {
        let query = Query::for_project("Project Query", 42);
        assert_eq!(query.project_id, Some(42));
        assert!(!query.is_global());
    }

    #[test]
    fn test_query_visibility() {
        let private_query = Query::new("Private").private();
        assert_eq!(private_query.visibility, QueryVisibility::Private);
        assert!(!private_query.is_public());

        let public_query = Query::new("Public").public();
        assert_eq!(public_query.visibility, QueryVisibility::Public);
        assert!(public_query.is_public());
    }

    #[test]
    fn test_query_display_modes() {
        let list_query = Query::new("List");
        assert_eq!(list_query.display, DisplayRepresentation::List);

        let board_query = Query::new("Board").board();
        assert_eq!(board_query.display, DisplayRepresentation::Board);

        let gantt_query = Query::new("Gantt").gantt();
        assert_eq!(gantt_query.display, DisplayRepresentation::Gantt);
        assert!(gantt_query.show_timeline);
    }

    #[test]
    fn test_query_with_filter() {
        let query = Query::new("Filtered")
            .with_filter(Filter::equals("status_id", FilterValue::Id(1)));

        assert!(query.has_filters());
        assert_eq!(query.filters.len(), 1);
    }

    #[test]
    fn test_query_grouping() {
        let ungrouped = Query::new("Ungrouped");
        assert!(!ungrouped.is_grouped());

        let grouped = Query::new("Grouped").grouped_by("status");
        assert!(grouped.is_grouped());
        assert_eq!(grouped.group_by.attribute, Some("status".to_string()));
    }

    #[test]
    fn test_display_representation_parsing() {
        assert_eq!(
            DisplayRepresentation::from_str("list"),
            Some(DisplayRepresentation::List)
        );
        assert_eq!(
            DisplayRepresentation::from_str("Board"),
            Some(DisplayRepresentation::Board)
        );
        assert_eq!(
            DisplayRepresentation::from_str("GANTT"),
            Some(DisplayRepresentation::Gantt)
        );
    }

    #[test]
    fn test_group_by() {
        let group = GroupBy::by("priority");
        assert!(group.is_grouped());
        assert!(!group.collapsed);

        let collapsed = GroupBy::by_collapsed("status");
        assert!(collapsed.is_grouped());
        assert!(collapsed.collapsed);

        let none = GroupBy::none();
        assert!(!none.is_grouped());
    }
}
