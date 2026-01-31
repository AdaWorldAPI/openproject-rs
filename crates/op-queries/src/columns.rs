//! Query Columns
//!
//! Mirrors: app/models/queries/columns/*
//!
//! Columns define which attributes are displayed in query results.

use std::collections::HashSet;

/// Column type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Standard attribute column
    Property,
    /// Relation column (parent, children, etc.)
    Relation,
    /// Custom field column
    CustomField,
    /// Computed/derived column
    Computed,
}

/// A query column definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// Column identifier (attribute name)
    pub name: String,
    /// Column type
    pub column_type: ColumnType,
    /// Display caption/label
    pub caption: Option<String>,
    /// Whether column is sortable
    pub sortable: bool,
    /// Whether column is groupable
    pub groupable: bool,
    /// Custom field ID (if applicable)
    pub custom_field_id: Option<i64>,
}

impl Column {
    /// Create a new property column
    pub fn property(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            column_type: ColumnType::Property,
            caption: None,
            sortable: true,
            groupable: false,
            custom_field_id: None,
        }
    }

    /// Create a new relation column
    pub fn relation(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            column_type: ColumnType::Relation,
            caption: None,
            sortable: false,
            groupable: false,
            custom_field_id: None,
        }
    }

    /// Create a new custom field column
    pub fn custom_field(cf_id: i64) -> Self {
        Self {
            name: format!("cf_{}", cf_id),
            column_type: ColumnType::CustomField,
            caption: None,
            sortable: true,
            groupable: true,
            custom_field_id: Some(cf_id),
        }
    }

    /// Create a computed column
    pub fn computed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            column_type: ColumnType::Computed,
            caption: None,
            sortable: false,
            groupable: false,
            custom_field_id: None,
        }
    }

    /// Set the caption
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Set sortable flag
    pub fn with_sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Set groupable flag
    pub fn with_groupable(mut self, groupable: bool) -> Self {
        self.groupable = groupable;
        self
    }

    /// Check if this is a custom field column
    pub fn is_custom_field(&self) -> bool {
        self.column_type == ColumnType::CustomField
    }

    /// Get the display caption
    pub fn display_caption(&self) -> &str {
        self.caption.as_deref().unwrap_or(&self.name)
    }
}

/// Standard work package columns
pub mod standard {
    use super::*;

    pub fn id() -> Column {
        Column::property("id").with_caption("ID").with_sortable(true)
    }

    pub fn subject() -> Column {
        Column::property("subject")
            .with_caption("Subject")
            .with_sortable(true)
    }

    pub fn status() -> Column {
        Column::property("status")
            .with_caption("Status")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn type_column() -> Column {
        Column::property("type")
            .with_caption("Type")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn priority() -> Column {
        Column::property("priority")
            .with_caption("Priority")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn assigned_to() -> Column {
        Column::property("assigned_to")
            .with_caption("Assignee")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn author() -> Column {
        Column::property("author")
            .with_caption("Author")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn project() -> Column {
        Column::property("project")
            .with_caption("Project")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn start_date() -> Column {
        Column::property("start_date")
            .with_caption("Start date")
            .with_sortable(true)
    }

    pub fn due_date() -> Column {
        Column::property("due_date")
            .with_caption("Finish date")
            .with_sortable(true)
    }

    pub fn estimated_hours() -> Column {
        Column::property("estimated_hours")
            .with_caption("Estimated time")
            .with_sortable(true)
    }

    pub fn spent_hours() -> Column {
        Column::computed("spent_hours")
            .with_caption("Spent time")
            .with_sortable(true)
    }

    pub fn remaining_hours() -> Column {
        Column::computed("remaining_hours")
            .with_caption("Remaining time")
            .with_sortable(true)
    }

    pub fn done_ratio() -> Column {
        Column::property("done_ratio")
            .with_caption("% Done")
            .with_sortable(true)
    }

    pub fn created_at() -> Column {
        Column::property("created_at")
            .with_caption("Created on")
            .with_sortable(true)
    }

    pub fn updated_at() -> Column {
        Column::property("updated_at")
            .with_caption("Updated on")
            .with_sortable(true)
    }

    pub fn version() -> Column {
        Column::property("version")
            .with_caption("Version")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn category() -> Column {
        Column::property("category")
            .with_caption("Category")
            .with_sortable(true)
            .with_groupable(true)
    }

    pub fn parent() -> Column {
        Column::relation("parent")
            .with_caption("Parent")
            .with_sortable(true)
    }

    pub fn responsible() -> Column {
        Column::property("responsible")
            .with_caption("Accountable")
            .with_sortable(true)
            .with_groupable(true)
    }
}

/// A set of columns for display
#[derive(Debug, Clone, Default)]
pub struct ColumnSet {
    columns: Vec<Column>,
}

impl ColumnSet {
    /// Create a new empty column set
    pub fn new() -> Self {
        Self { columns: vec![] }
    }

    /// Create default column set for work packages
    pub fn default_work_package() -> Self {
        Self {
            columns: vec![
                standard::id(),
                standard::subject(),
                standard::type_column(),
                standard::status(),
                standard::assigned_to(),
                standard::priority(),
            ],
        }
    }

    /// Add a column
    pub fn add(&mut self, column: Column) -> &mut Self {
        self.columns.push(column);
        self
    }

    /// Add a column (builder pattern)
    pub fn with(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Add a column by name (creates a property column)
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.columns.push(Column::property(name));
        self
    }

    /// Get all columns
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Check if any columns are set
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Get number of columns
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Check if a specific column is included
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.iter().any(|c| c.name == name)
    }

    /// Remove a column by name
    pub fn remove(&mut self, name: &str) {
        self.columns.retain(|c| c.name != name);
    }

    /// Get all column names
    pub fn names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// Get unique column names as a set
    pub fn name_set(&self) -> HashSet<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// Get sortable columns
    pub fn sortable_columns(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.sortable).collect()
    }

    /// Get groupable columns
    pub fn groupable_columns(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.groupable).collect()
    }

    /// Reorder columns to match the given order
    pub fn reorder(&mut self, names: &[&str]) {
        let mut reordered = Vec::with_capacity(self.columns.len());
        for name in names {
            if let Some(pos) = self.columns.iter().position(|c| c.name == *name) {
                reordered.push(self.columns.remove(pos));
            }
        }
        // Add any remaining columns at the end
        reordered.append(&mut self.columns);
        self.columns = reordered;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let col = Column::property("subject").with_caption("Subject");
        assert_eq!(col.name, "subject");
        assert_eq!(col.display_caption(), "Subject");
        assert!(col.sortable);
        assert!(!col.is_custom_field());
    }

    #[test]
    fn test_custom_field_column() {
        let col = Column::custom_field(42);
        assert_eq!(col.name, "cf_42");
        assert!(col.is_custom_field());
        assert_eq!(col.custom_field_id, Some(42));
    }

    #[test]
    fn test_column_set() {
        let set = ColumnSet::new()
            .with(standard::id())
            .with(standard::subject())
            .with(standard::status());

        assert_eq!(set.len(), 3);
        assert!(set.has_column("id"));
        assert!(set.has_column("subject"));
        assert!(!set.has_column("priority"));
    }

    #[test]
    fn test_default_columns() {
        let set = ColumnSet::default_work_package();
        assert!(!set.is_empty());
        assert!(set.has_column("id"));
        assert!(set.has_column("subject"));
        assert!(set.has_column("status"));
    }

    #[test]
    fn test_column_set_operations() {
        let mut set = ColumnSet::new()
            .with(standard::id())
            .with(standard::subject())
            .with(standard::status());

        assert_eq!(set.len(), 3);

        set.remove("subject");
        assert_eq!(set.len(), 2);
        assert!(!set.has_column("subject"));

        let names = set.names();
        assert!(names.contains(&"id"));
        assert!(names.contains(&"status"));
    }

    #[test]
    fn test_sortable_columns() {
        let set = ColumnSet::default_work_package();
        let sortable = set.sortable_columns();
        assert!(!sortable.is_empty());
        assert!(sortable.iter().all(|c| c.sortable));
    }

    #[test]
    fn test_groupable_columns() {
        let set = ColumnSet::default_work_package();
        let groupable = set.groupable_columns();
        assert!(!groupable.is_empty());
        assert!(groupable.iter().all(|c| c.groupable));
    }
}
