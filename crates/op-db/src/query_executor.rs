//! Query Executor
//!
//! Translates op-queries Query objects into SQL and executes them against
//! the OpenProject database. This provides full compatibility with
//! OpenProject's query system.

use op_core::traits::Id;
use op_queries::{
    Filter, FilterOperator, FilterSet, FilterValue,
    Query, SortCriterion, SortDirection, SortOrder,
};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};

use crate::repository::{Pagination, PaginatedResult, RepositoryError, RepositoryResult};

/// Query executor for work packages
pub struct WorkPackageQueryExecutor<'a> {
    pool: &'a PgPool,
}

impl<'a> WorkPackageQueryExecutor<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Execute a query and return paginated work package results
    pub async fn execute(
        &self,
        query: &Query,
        pagination: &Pagination,
        current_user_id: Option<Id>,
    ) -> RepositoryResult<PaginatedResult<WorkPackageRow>> {
        let (where_clause, params) = self.build_where_clause(&query.filters, current_user_id);
        let order_clause = self.build_order_clause(&query.sorts);

        // Build the main query
        let sql = format!(
            r#"
            SELECT
                wp.id,
                wp.subject,
                wp.description,
                wp.project_id,
                wp.type_id,
                wp.status_id,
                wp.priority_id,
                wp.author_id,
                wp.assigned_to_id,
                wp.responsible_id,
                wp.category_id,
                wp.version_id,
                wp.parent_id,
                wp.start_date,
                wp.due_date,
                wp.estimated_hours,
                wp.done_ratio,
                wp.lock_version,
                wp.created_at,
                wp.updated_at,
                wp.position,
                wp.story_points,
                wp.remaining_hours,
                wp.schedule_manually,
                wp.duration
            FROM work_packages wp
            LEFT JOIN statuses s ON wp.status_id = s.id
            LEFT JOIN types t ON wp.type_id = t.id
            LEFT JOIN enumerations p ON wp.priority_id = p.id AND p.type = 'IssuePriority'
            {}
            {}
            LIMIT $1 OFFSET $2
            "#,
            if where_clause.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", where_clause)
            },
            order_clause
        );

        // Build count query
        let count_sql = format!(
            r#"
            SELECT COUNT(*) as count
            FROM work_packages wp
            LEFT JOIN statuses s ON wp.status_id = s.id
            LEFT JOIN types t ON wp.type_id = t.id
            LEFT JOIN enumerations p ON wp.priority_id = p.id AND p.type = 'IssuePriority'
            {}
            "#,
            if where_clause.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", where_clause)
            }
        );

        // Execute count query
        let count_row: (i64,) = sqlx::query_as(&count_sql)
            .fetch_one(self.pool)
            .await
            .map_err(RepositoryError::Database)?;
        let total = count_row.0;

        // Execute main query
        let rows = sqlx::query_as::<_, WorkPackageRow>(&sql)
            .bind(pagination.limit)
            .bind(pagination.offset)
            .fetch_all(self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Build WHERE clause from filter set
    fn build_where_clause(
        &self,
        filters: &FilterSet,
        current_user_id: Option<Id>,
    ) -> (String, Vec<SqlParam>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        for filter in filters.filters() {
            if let Some(condition) = self.filter_to_sql(filter, current_user_id, &mut params) {
                conditions.push(condition);
            }
        }

        (conditions.join(" AND "), params)
    }

    /// Convert a single filter to SQL condition
    fn filter_to_sql(
        &self,
        filter: &Filter,
        current_user_id: Option<Id>,
        _params: &mut Vec<SqlParam>,
    ) -> Option<String> {
        let column = self.attribute_to_column(&filter.attribute)?;

        match &filter.operator {
            FilterOperator::Equals => {
                let values = self.values_to_sql(&filter.values, current_user_id);
                if values.len() == 1 {
                    Some(format!("{} = {}", column, values[0]))
                } else {
                    Some(format!("{} IN ({})", column, values.join(", ")))
                }
            }
            FilterOperator::NotEquals => {
                let values = self.values_to_sql(&filter.values, current_user_id);
                if values.len() == 1 {
                    Some(format!("{} != {}", column, values[0]))
                } else {
                    Some(format!("{} NOT IN ({})", column, values.join(", ")))
                }
            }
            FilterOperator::Contains => {
                if let FilterValue::String(s) = &filter.values {
                    Some(format!("{} ILIKE '%{}%'", column, escape_like(s)))
                } else {
                    None
                }
            }
            FilterOperator::NotContains => {
                if let FilterValue::String(s) = &filter.values {
                    Some(format!("{} NOT ILIKE '%{}%'", column, escape_like(s)))
                } else {
                    None
                }
            }
            FilterOperator::StartsWith => {
                if let FilterValue::String(s) = &filter.values {
                    Some(format!("{} ILIKE '{}%'", column, escape_like(s)))
                } else {
                    None
                }
            }
            FilterOperator::EndsWith => {
                if let FilterValue::String(s) = &filter.values {
                    Some(format!("{} ILIKE '%{}'", column, escape_like(s)))
                } else {
                    None
                }
            }
            FilterOperator::GreaterThan => {
                let values = self.values_to_sql(&filter.values, current_user_id);
                values.first().map(|v| format!("{} > {}", column, v))
            }
            FilterOperator::GreaterThanOrEqual => {
                let values = self.values_to_sql(&filter.values, current_user_id);
                values.first().map(|v| format!("{} >= {}", column, v))
            }
            FilterOperator::LessThan => {
                let values = self.values_to_sql(&filter.values, current_user_id);
                values.first().map(|v| format!("{} < {}", column, v))
            }
            FilterOperator::LessThanOrEqual => {
                let values = self.values_to_sql(&filter.values, current_user_id);
                values.first().map(|v| format!("{} <= {}", column, v))
            }
            FilterOperator::Between => {
                if let FilterValue::DateRange { from, to } = &filter.values {
                    Some(format!("{} BETWEEN '{}' AND '{}'", column, from, to))
                } else {
                    None
                }
            }
            FilterOperator::IsNull => Some(format!("{} IS NULL", column)),
            FilterOperator::IsNotNull => Some(format!("{} IS NOT NULL", column)),
            FilterOperator::Today => Some(format!("{} = CURRENT_DATE", column)),
            FilterOperator::ThisWeek => Some(format!(
                "{} >= date_trunc('week', CURRENT_DATE) AND {} < date_trunc('week', CURRENT_DATE) + interval '1 week'",
                column, column
            )),
            FilterOperator::DaysAgo(n) => {
                Some(format!("{} = CURRENT_DATE - interval '{} days'", column, n))
            }
            FilterOperator::DaysFromNow(n) => {
                Some(format!("{} = CURRENT_DATE + interval '{} days'", column, n))
            }
            FilterOperator::LessThanDaysAgo(n) => {
                Some(format!("{} > CURRENT_DATE - interval '{} days'", column, n))
            }
            FilterOperator::MoreThanDaysAgo(n) => {
                Some(format!("{} < CURRENT_DATE - interval '{} days'", column, n))
            }
            FilterOperator::LessThanDaysFromNow(n) => {
                Some(format!("{} < CURRENT_DATE + interval '{} days'", column, n))
            }
            FilterOperator::MoreThanDaysFromNow(n) => {
                Some(format!("{} > CURRENT_DATE + interval '{} days'", column, n))
            }
            FilterOperator::CurrentUser => {
                if let Some(user_id) = current_user_id {
                    Some(format!("{} = {}", column, user_id))
                } else {
                    // Anonymous user, no match
                    Some("1 = 0".to_string())
                }
            }
        }
    }

    /// Map attribute names to database columns
    fn attribute_to_column(&self, attribute: &str) -> Option<String> {
        attribute_to_column(attribute)
    }

    /// Convert filter values to SQL literals
    fn values_to_sql(&self, values: &FilterValue, current_user_id: Option<Id>) -> Vec<String> {
        values_to_sql(values, current_user_id)
    }

    /// Build ORDER BY clause from sort order
    fn build_order_clause(&self, sorts: &SortOrder) -> String {
        build_order_clause(sorts)
    }

    /// Map sort attribute names to database columns
    fn sort_attribute_to_column(&self, attribute: &str) -> Option<String> {
        sort_attribute_to_column(attribute)
    }
}

/// Map attribute names to database columns (standalone function for testing)
pub fn attribute_to_column(attribute: &str) -> Option<String> {
    match attribute {
        "id" => Some("wp.id".to_string()),
        "subject" => Some("wp.subject".to_string()),
        "description" => Some("wp.description".to_string()),
        "project_id" => Some("wp.project_id".to_string()),
        "type_id" => Some("wp.type_id".to_string()),
        "status_id" => Some("wp.status_id".to_string()),
        "priority_id" => Some("wp.priority_id".to_string()),
        "author_id" => Some("wp.author_id".to_string()),
        "assigned_to_id" => Some("wp.assigned_to_id".to_string()),
        "responsible_id" => Some("wp.responsible_id".to_string()),
        "category_id" => Some("wp.category_id".to_string()),
        "version_id" => Some("wp.version_id".to_string()),
        "parent_id" => Some("wp.parent_id".to_string()),
        "start_date" => Some("wp.start_date".to_string()),
        "due_date" => Some("wp.due_date".to_string()),
        "estimated_hours" => Some("wp.estimated_hours".to_string()),
        "done_ratio" => Some("wp.done_ratio".to_string()),
        "created_at" => Some("wp.created_at".to_string()),
        "updated_at" => Some("wp.updated_at".to_string()),
        // Status attributes
        "status" => Some("s.name".to_string()),
        "status_is_closed" => Some("s.is_closed".to_string()),
        // Type attributes
        "type" => Some("t.name".to_string()),
        // Priority attributes
        "priority" => Some("p.name".to_string()),
        "priority_position" => Some("p.position".to_string()),
        // Custom field pattern: cf_123
        cf if cf.starts_with("cf_") => {
            // Custom fields require a join - return None for now
            // TODO: Implement custom field joins
            None
        }
        _ => None,
    }
}

/// Convert filter values to SQL literals (standalone function for testing)
pub fn values_to_sql(values: &FilterValue, current_user_id: Option<Id>) -> Vec<String> {
    match values {
        FilterValue::Id(id) => vec![id.to_string()],
        FilterValue::Ids(ids) => ids.iter().map(|id| id.to_string()).collect(),
        FilterValue::String(s) => vec![format!("'{}'", escape_string(s))],
        FilterValue::Strings(ss) => ss.iter().map(|s| format!("'{}'", escape_string(s))).collect(),
        FilterValue::Bool(b) => vec![if *b { "true".to_string() } else { "false".to_string() }],
        FilterValue::Date(d) => vec![format!("'{}'", d)],
        FilterValue::DateRange { from, to } => vec![format!("'{}'", from), format!("'{}'", to)],
        FilterValue::Number(n) => vec![n.to_string()],
        FilterValue::Me => {
            if let Some(user_id) = current_user_id {
                vec![user_id.to_string()]
            } else {
                vec![]
            }
        }
        FilterValue::None => vec![],
    }
}

/// Map sort attribute names to database columns (standalone function for testing)
pub fn sort_attribute_to_column(attribute: &str) -> Option<String> {
    match attribute {
        "id" => Some("wp.id".to_string()),
        "subject" => Some("wp.subject".to_string()),
        "project" => Some("wp.project_id".to_string()),
        "type" => Some("t.position".to_string()),
        "status" => Some("s.position".to_string()),
        "priority" => Some("p.position".to_string()),
        "author" => Some("wp.author_id".to_string()),
        "assigned_to" => Some("wp.assigned_to_id".to_string()),
        "responsible" => Some("wp.responsible_id".to_string()),
        "start_date" => Some("wp.start_date".to_string()),
        "due_date" => Some("wp.due_date".to_string()),
        "estimated_hours" => Some("wp.estimated_hours".to_string()),
        "done_ratio" => Some("wp.done_ratio".to_string()),
        "created_at" => Some("wp.created_at".to_string()),
        "updated_at" => Some("wp.updated_at".to_string()),
        "version" => Some("wp.version_id".to_string()),
        "category" => Some("wp.category_id".to_string()),
        "parent" => Some("wp.parent_id".to_string()),
        _ => None,
    }
}

/// Build ORDER BY clause from sort order (standalone function for testing)
pub fn build_order_clause(sorts: &SortOrder) -> String {
    if sorts.is_empty() {
        return "ORDER BY wp.id DESC".to_string();
    }

    let order_parts: Vec<String> = sorts
        .criteria()
        .iter()
        .filter_map(|criterion| sort_to_sql(criterion))
        .collect();

    if order_parts.is_empty() {
        "ORDER BY wp.id DESC".to_string()
    } else {
        format!("ORDER BY {}", order_parts.join(", "))
    }
}

/// Convert a sort criterion to SQL (standalone function)
fn sort_to_sql(criterion: &SortCriterion) -> Option<String> {
    let column = sort_attribute_to_column(&criterion.attribute)?;
    let direction = match criterion.direction {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };

    // Handle NULLS for optional columns
    let nulls = match criterion.direction {
        SortDirection::Asc => "NULLS LAST",
        SortDirection::Desc => "NULLS FIRST",
    };

    Some(format!("{} {} {}", column, direction, nulls))
}

/// Work package row from database
#[derive(Debug, Clone, FromRow)]
pub struct WorkPackageRow {
    pub id: i64,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: i64,
    pub type_id: i64,
    pub status_id: i64,
    pub priority_id: Option<i64>,
    pub author_id: Option<i64>,
    pub assigned_to_id: Option<i64>,
    pub responsible_id: Option<i64>,
    pub category_id: Option<i64>,
    pub version_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: i32,
    pub lock_version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub position: Option<i32>,
    pub story_points: Option<i32>,
    pub remaining_hours: Option<f64>,
    pub schedule_manually: bool,
    pub duration: Option<i32>,
}

/// Parameter for prepared statements
#[derive(Debug, Clone)]
pub enum SqlParam {
    Int(i64),
    String(String),
    Float(f64),
    Bool(bool),
    Date(String),
    Null,
}

/// Escape string for SQL (prevent SQL injection)
fn escape_string(s: &str) -> String {
    s.replace('\'', "''")
}

/// Escape string for LIKE patterns
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
        .replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("test"), "test");
        assert_eq!(escape_string("it's"), "it''s");
        assert_eq!(escape_string("O'Brien"), "O''Brien");
    }

    #[test]
    fn test_escape_like() {
        assert_eq!(escape_like("test"), "test");
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("it's"), "it''s");
    }

    #[test]
    fn test_attribute_to_column() {
        // Use standalone function directly - no pool needed
        assert_eq!(
            attribute_to_column("id"),
            Some("wp.id".to_string())
        );
        assert_eq!(
            attribute_to_column("subject"),
            Some("wp.subject".to_string())
        );
        assert_eq!(
            attribute_to_column("status"),
            Some("s.name".to_string())
        );
        assert_eq!(attribute_to_column("unknown"), None);
    }

    #[test]
    fn test_values_to_sql() {
        // Use standalone function directly - no pool needed
        assert_eq!(
            values_to_sql(&FilterValue::Id(42), None),
            vec!["42"]
        );
        assert_eq!(
            values_to_sql(&FilterValue::Ids(vec![1, 2, 3]), None),
            vec!["1", "2", "3"]
        );
        assert_eq!(
            values_to_sql(&FilterValue::String("test".to_string()), None),
            vec!["'test'"]
        );
        assert_eq!(
            values_to_sql(&FilterValue::Me, Some(5)),
            vec!["5"]
        );
        assert!(values_to_sql(&FilterValue::Me, None).is_empty());
    }

    #[test]
    fn test_build_order_clause() {
        // Use standalone function directly - no pool needed

        // Empty sorts
        let empty = SortOrder::new();
        assert_eq!(build_order_clause(&empty), "ORDER BY wp.id DESC");

        // Single sort
        let single = SortOrder::by_desc("updated_at");
        assert_eq!(
            build_order_clause(&single),
            "ORDER BY wp.updated_at DESC NULLS FIRST"
        );

        // Multiple sorts
        let multiple = SortOrder::by_asc("priority").then_desc("id");
        assert_eq!(
            build_order_clause(&multiple),
            "ORDER BY p.position ASC NULLS LAST, wp.id DESC NULLS FIRST"
        );
    }

    #[test]
    fn test_sort_attribute_to_column() {
        assert_eq!(
            sort_attribute_to_column("id"),
            Some("wp.id".to_string())
        );
        assert_eq!(
            sort_attribute_to_column("priority"),
            Some("p.position".to_string())
        );
        assert_eq!(
            sort_attribute_to_column("status"),
            Some("s.position".to_string())
        );
        assert_eq!(sort_attribute_to_column("unknown"), None);
    }
}
