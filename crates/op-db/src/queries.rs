//! Queries repository
//!
//! Mirrors: app/models/query.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Query row from database
#[derive(Debug, Clone, FromRow)]
pub struct QueryRow {
    pub id: i64,
    pub project_id: Option<i64>,
    pub user_id: i64,
    pub name: String,
    pub filters: Option<String>,
    pub column_names: Option<String>,
    pub sort_criteria: Option<String>,
    pub group_by: Option<String>,
    pub display_sums: bool,
    pub show_hierarchies: bool,
    pub include_subprojects: bool,
    pub timeline_visible: bool,
    pub timestamps: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl QueryRow {
    /// Check if this is a global query (not scoped to a project)
    pub fn is_global(&self) -> bool {
        self.project_id.is_none()
    }

    /// Check if this query shows hierarchies
    pub fn is_hierarchical(&self) -> bool {
        self.show_hierarchies && self.group_by.is_none()
    }

    /// Check if this query is grouped
    pub fn is_grouped(&self) -> bool {
        self.group_by.is_some()
    }
}

/// Query with starred status
#[derive(Debug, Clone)]
pub struct QueryWithStarred {
    pub query: QueryRow,
    pub starred: bool,
}

/// DTO for creating a query
#[derive(Debug, Clone)]
pub struct CreateQueryDto {
    pub project_id: Option<i64>,
    pub user_id: i64,
    pub name: String,
    pub filters: Option<String>,
    pub column_names: Option<String>,
    pub sort_criteria: Option<String>,
    pub group_by: Option<String>,
    pub display_sums: Option<bool>,
    pub show_hierarchies: Option<bool>,
    pub include_subprojects: Option<bool>,
    pub timeline_visible: Option<bool>,
    pub timestamps: Option<String>,
}

/// DTO for updating a query
#[derive(Debug, Clone, Default)]
pub struct UpdateQueryDto {
    pub name: Option<String>,
    pub filters: Option<Option<String>>,
    pub column_names: Option<Option<String>>,
    pub sort_criteria: Option<Option<String>>,
    pub group_by: Option<Option<String>>,
    pub display_sums: Option<bool>,
    pub show_hierarchies: Option<bool>,
    pub include_subprojects: Option<bool>,
    pub timeline_visible: Option<bool>,
    pub timestamps: Option<Option<String>>,
}

/// Query repository
pub struct QueryRepository {
    pool: PgPool,
}

impl QueryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find queries by project
    pub async fn find_by_project(
        &self,
        project_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<QueryRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, QueryRow>(
            r#"
            SELECT id, project_id, user_id, name, filters, column_names, sort_criteria,
                   group_by, display_sums, show_hierarchies, include_subprojects,
                   timeline_visible, timestamps, created_at, updated_at
            FROM queries
            WHERE project_id = $1
            ORDER BY name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(project_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM queries WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find global queries (not scoped to a project)
    pub async fn find_global(
        &self,
        pagination: Pagination,
    ) -> Result<PaginatedResult<QueryRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, QueryRow>(
            r#"
            SELECT id, project_id, user_id, name, filters, column_names, sort_criteria,
                   group_by, display_sums, show_hierarchies, include_subprojects,
                   timeline_visible, timestamps, created_at, updated_at
            FROM queries
            WHERE project_id IS NULL
            ORDER BY name ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM queries WHERE project_id IS NULL",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find queries by user
    pub async fn find_by_user(
        &self,
        user_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<QueryRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, QueryRow>(
            r#"
            SELECT id, project_id, user_id, name, filters, column_names, sort_criteria,
                   group_by, display_sums, show_hierarchies, include_subprojects,
                   timeline_visible, timestamps, created_at, updated_at
            FROM queries
            WHERE user_id = $1
            ORDER BY name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM queries WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find visible queries for a user (own queries + public queries)
    pub async fn find_visible(
        &self,
        user_id: i64,
        project_id: Option<i64>,
        pagination: Pagination,
    ) -> Result<PaginatedResult<QueryWithStarred>, RepositoryError> {
        let rows = match project_id {
            Some(pid) => {
                sqlx::query_as::<_, QueryRow>(
                    r#"
                    SELECT q.id, q.project_id, q.user_id, q.name, q.filters, q.column_names,
                           q.sort_criteria, q.group_by, q.display_sums, q.show_hierarchies,
                           q.include_subprojects, q.timeline_visible, q.timestamps,
                           q.created_at, q.updated_at
                    FROM queries q
                    LEFT JOIN views v ON v.query_id = q.id
                    WHERE (q.user_id = $1 OR v.type = 'Views::WorkPackagesTable')
                      AND (q.project_id = $2 OR q.project_id IS NULL)
                    ORDER BY q.name ASC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(user_id)
                .bind(pid)
                .bind(pagination.limit)
                .bind(pagination.offset)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, QueryRow>(
                    r#"
                    SELECT q.id, q.project_id, q.user_id, q.name, q.filters, q.column_names,
                           q.sort_criteria, q.group_by, q.display_sums, q.show_hierarchies,
                           q.include_subprojects, q.timeline_visible, q.timestamps,
                           q.created_at, q.updated_at
                    FROM queries q
                    LEFT JOIN views v ON v.query_id = q.id
                    WHERE q.user_id = $1 OR v.type = 'Views::WorkPackagesTable'
                    ORDER BY q.name ASC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(user_id)
                .bind(pagination.limit)
                .bind(pagination.offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        // Check starred status for each query
        let mut items = Vec::with_capacity(rows.len());
        for query in rows {
            let starred = self.is_starred(query.id, user_id).await?;
            items.push(QueryWithStarred { query, starred });
        }

        let total = match project_id {
            Some(pid) => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(DISTINCT q.id) FROM queries q
                    LEFT JOIN views v ON v.query_id = q.id
                    WHERE (q.user_id = $1 OR v.type = 'Views::WorkPackagesTable')
                      AND (q.project_id = $2 OR q.project_id IS NULL)
                    "#,
                )
                .bind(user_id)
                .bind(pid)
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(DISTINCT q.id) FROM queries q
                    LEFT JOIN views v ON v.query_id = q.id
                    WHERE q.user_id = $1 OR v.type = 'Views::WorkPackagesTable'
                    "#,
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Star a query for a user
    pub async fn star(&self, query_id: i64, user_id: i64) -> Result<(), RepositoryError> {
        // Check if query exists
        if !self.exists(query_id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Query {} not found",
                query_id
            )));
        }

        // Insert into query_menu_items or starred_queries (depending on schema)
        // OpenProject uses query_menu_items table for starring
        sqlx::query(
            r#"
            INSERT INTO query_menu_items (navigatable_id, name, title)
            SELECT $1, q.name, q.name
            FROM queries q
            WHERE q.id = $1
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(query_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Unstar a query for a user
    pub async fn unstar(&self, query_id: i64, _user_id: i64) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM query_menu_items WHERE navigatable_id = $1")
            .bind(query_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Check if a query is starred
    pub async fn is_starred(&self, query_id: i64, _user_id: i64) -> Result<bool, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM query_menu_items WHERE navigatable_id = $1",
        )
        .bind(query_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Get query with starred status
    pub async fn find_by_id_with_starred(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<Option<QueryWithStarred>, RepositoryError> {
        let query = self.find_by_id(id).await?;

        match query {
            Some(q) => {
                let starred = self.is_starred(id, user_id).await?;
                Ok(Some(QueryWithStarred { query: q, starred }))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl Repository<QueryRow, CreateQueryDto, UpdateQueryDto> for QueryRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<QueryRow>, RepositoryError> {
        let row = sqlx::query_as::<_, QueryRow>(
            r#"
            SELECT id, project_id, user_id, name, filters, column_names, sort_criteria,
                   group_by, display_sums, show_hierarchies, include_subprojects,
                   timeline_visible, timestamps, created_at, updated_at
            FROM queries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<QueryRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, QueryRow>(
            r#"
            SELECT id, project_id, user_id, name, filters, column_names, sort_criteria,
                   group_by, display_sums, show_hierarchies, include_subprojects,
                   timeline_visible, timestamps, created_at, updated_at
            FROM queries
            ORDER BY name ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM queries")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM queries WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateQueryDto) -> Result<QueryRow, RepositoryError> {
        // Validate name
        if dto.name.trim().is_empty() {
            return Err(RepositoryError::Validation(
                "Name can't be blank".to_string(),
            ));
        }

        let display_sums = dto.display_sums.unwrap_or(false);
        let show_hierarchies = dto.show_hierarchies.unwrap_or(true);
        let include_subprojects = dto.include_subprojects.unwrap_or(true);
        let timeline_visible = dto.timeline_visible.unwrap_or(false);

        let row = sqlx::query_as::<_, QueryRow>(
            r#"
            INSERT INTO queries (
                project_id, user_id, name, filters, column_names, sort_criteria,
                group_by, display_sums, show_hierarchies, include_subprojects,
                timeline_visible, timestamps, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
            RETURNING id, project_id, user_id, name, filters, column_names, sort_criteria,
                      group_by, display_sums, show_hierarchies, include_subprojects,
                      timeline_visible, timestamps, created_at, updated_at
            "#,
        )
        .bind(dto.project_id)
        .bind(dto.user_id)
        .bind(&dto.name)
        .bind(&dto.filters)
        .bind(&dto.column_names)
        .bind(&dto.sort_criteria)
        .bind(&dto.group_by)
        .bind(display_sums)
        .bind(show_hierarchies)
        .bind(include_subprojects)
        .bind(timeline_visible)
        .bind(&dto.timestamps)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, dto: UpdateQueryDto) -> Result<QueryRow, RepositoryError> {
        // Verify query exists
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Query {} not found", id)))?;

        // Validate name if provided
        if let Some(ref name) = dto.name {
            if name.trim().is_empty() {
                return Err(RepositoryError::Validation(
                    "Name can't be blank".to_string(),
                ));
            }
        }

        let new_name = dto.name.unwrap_or(existing.name);
        let new_filters = match dto.filters {
            Some(f) => f,
            None => existing.filters,
        };
        let new_column_names = match dto.column_names {
            Some(c) => c,
            None => existing.column_names,
        };
        let new_sort_criteria = match dto.sort_criteria {
            Some(s) => s,
            None => existing.sort_criteria,
        };
        let new_group_by = match dto.group_by {
            Some(g) => g,
            None => existing.group_by,
        };
        let new_display_sums = dto.display_sums.unwrap_or(existing.display_sums);
        let new_show_hierarchies = dto.show_hierarchies.unwrap_or(existing.show_hierarchies);
        let new_include_subprojects = dto.include_subprojects.unwrap_or(existing.include_subprojects);
        let new_timeline_visible = dto.timeline_visible.unwrap_or(existing.timeline_visible);
        let new_timestamps = match dto.timestamps {
            Some(t) => t,
            None => existing.timestamps,
        };

        let row = sqlx::query_as::<_, QueryRow>(
            r#"
            UPDATE queries
            SET name = $2, filters = $3, column_names = $4, sort_criteria = $5,
                group_by = $6, display_sums = $7, show_hierarchies = $8,
                include_subprojects = $9, timeline_visible = $10, timestamps = $11,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, project_id, user_id, name, filters, column_names, sort_criteria,
                      group_by, display_sums, show_hierarchies, include_subprojects,
                      timeline_visible, timestamps, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&new_name)
        .bind(&new_filters)
        .bind(&new_column_names)
        .bind(&new_sort_criteria)
        .bind(&new_group_by)
        .bind(new_display_sums)
        .bind(new_show_hierarchies)
        .bind(new_include_subprojects)
        .bind(new_timeline_visible)
        .bind(&new_timestamps)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if query exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Query {} not found",
                id
            )));
        }

        // Delete associated records first
        sqlx::query("DELETE FROM query_menu_items WHERE navigatable_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM views WHERE query_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Delete the query
        sqlx::query("DELETE FROM queries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_is_global() {
        let query = QueryRow {
            id: 1,
            project_id: None,
            user_id: 1,
            name: "Global Query".to_string(),
            filters: None,
            column_names: None,
            sort_criteria: None,
            group_by: None,
            display_sums: false,
            show_hierarchies: true,
            include_subprojects: true,
            timeline_visible: false,
            timestamps: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(query.is_global());

        let project_query = QueryRow {
            project_id: Some(1),
            ..query
        };

        assert!(!project_query.is_global());
    }
}
