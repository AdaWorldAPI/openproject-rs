//! Categories repository
//!
//! Mirrors: app/models/category.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Category row from database
#[derive(Debug, Clone, FromRow)]
pub struct CategoryRow {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub assigned_to_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a category
#[derive(Debug, Clone)]
pub struct CreateCategoryDto {
    pub project_id: i64,
    pub name: String,
    pub assigned_to_id: Option<i64>,
}

/// DTO for updating a category
#[derive(Debug, Clone, Default)]
pub struct UpdateCategoryDto {
    pub name: Option<String>,
    pub assigned_to_id: Option<i64>,
}

/// Category repository
pub struct CategoryRepository {
    pool: PgPool,
}

impl CategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all categories for a project
    pub async fn find_by_project(
        &self,
        project_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<CategoryRow>, RepositoryError> {
        let items = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, project_id, name, assigned_to_id, created_at, updated_at
            FROM categories
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
            "SELECT COUNT(*) FROM categories WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find all categories for a project (no pagination)
    pub async fn find_all_by_project(&self, project_id: i64) -> Result<Vec<CategoryRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, project_id, name, assigned_to_id, created_at, updated_at
            FROM categories
            WHERE project_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Check if category name is unique within a project
    async fn is_name_unique(
        &self,
        project_id: i64,
        name: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, RepositoryError> {
        let count = if let Some(id) = exclude_id {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM categories
                WHERE project_id = $1 AND LOWER(name) = LOWER($2) AND id != $3
                "#,
            )
            .bind(project_id)
            .bind(name)
            .bind(id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM categories
                WHERE project_id = $1 AND LOWER(name) = LOWER($2)
                "#,
            )
            .bind(project_id)
            .bind(name)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(count == 0)
    }
}

#[async_trait]
impl Repository<CategoryRow, CreateCategoryDto, UpdateCategoryDto> for CategoryRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<CategoryRow>, RepositoryError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, project_id, name, assigned_to_id, created_at, updated_at
            FROM categories
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<CategoryRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, project_id, name, assigned_to_id, created_at, updated_at
            FROM categories
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM categories")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM categories WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateCategoryDto) -> Result<CategoryRow, RepositoryError> {
        // Validate name
        if dto.name.trim().is_empty() {
            return Err(RepositoryError::Validation("Name can't be blank".to_string()));
        }

        // Check name uniqueness
        if !self.is_name_unique(dto.project_id, &dto.name, None).await? {
            return Err(RepositoryError::Conflict(
                "Name has already been taken".to_string(),
            ));
        }

        let row = sqlx::query_as::<_, CategoryRow>(
            r#"
            INSERT INTO categories (project_id, name, assigned_to_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING id, project_id, name, assigned_to_id, created_at, updated_at
            "#,
        )
        .bind(dto.project_id)
        .bind(&dto.name)
        .bind(dto.assigned_to_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, dto: UpdateCategoryDto) -> Result<CategoryRow, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Category {} not found", id)))?;

        let name = dto.name.unwrap_or(existing.name);
        let assigned_to_id = dto.assigned_to_id.or(existing.assigned_to_id);

        // Validate name
        if name.trim().is_empty() {
            return Err(RepositoryError::Validation("Name can't be blank".to_string()));
        }

        // Check name uniqueness
        if !self.is_name_unique(existing.project_id, &name, Some(id)).await? {
            return Err(RepositoryError::Conflict(
                "Name has already been taken".to_string(),
            ));
        }

        let row = sqlx::query_as::<_, CategoryRow>(
            r#"
            UPDATE categories
            SET name = $2, assigned_to_id = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, project_id, name, assigned_to_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&name)
        .bind(assigned_to_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if category exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Category {} not found",
                id
            )));
        }

        // Check if category is in use (has work packages)
        let wp_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM work_packages WHERE category_id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if wp_count > 0 {
            return Err(RepositoryError::Conflict(format!(
                "Cannot delete category with {} work packages",
                wp_count
            )));
        }

        sqlx::query("DELETE FROM categories WHERE id = $1")
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
    fn test_category_dto() {
        let dto = CreateCategoryDto {
            project_id: 1,
            name: "Bug".to_string(),
            assigned_to_id: Some(2),
        };

        assert_eq!(dto.project_id, 1);
        assert_eq!(dto.name, "Bug");
        assert_eq!(dto.assigned_to_id, Some(2));
    }
}
