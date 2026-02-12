//! Time Entry Activities repository
//!
//! Mirrors: app/models/time_entry_activity.rb (subclass of Enumeration)
//!
//! Activities are stored in the `enumerations` table with type = 'TimeEntryActivity'

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Activity type constant for enumerations table
const ACTIVITY_TYPE: &str = "TimeEntryActivity";

/// Activity row from database
#[derive(Debug, Clone, FromRow)]
pub struct ActivityRow {
    pub id: i64,
    pub name: String,
    pub position: i32,
    pub is_default: bool,
    pub active: bool,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating an activity
#[derive(Debug, Clone)]
pub struct CreateActivityDto {
    pub name: String,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
}

/// DTO for updating an activity
#[derive(Debug, Clone, Default)]
pub struct UpdateActivityDto {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
}

/// Activity repository
pub struct ActivityRepository {
    pool: PgPool,
}

impl ActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all shared (global) activities
    pub async fn find_shared(&self) -> Result<Vec<ActivityRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND project_id IS NULL
            ORDER BY position ASC
            "#,
        )
        .bind(ACTIVITY_TYPE)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find activities for a specific project (includes shared activities)
    pub async fn find_by_project(&self, project_id: i64) -> Result<Vec<ActivityRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND (project_id IS NULL OR project_id = $2)
            ORDER BY position ASC
            "#,
        )
        .bind(ACTIVITY_TYPE)
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find active activities for a project
    pub async fn find_active_by_project(&self, project_id: i64) -> Result<Vec<ActivityRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND (project_id IS NULL OR project_id = $2) AND active = true
            ORDER BY position ASC
            "#,
        )
        .bind(ACTIVITY_TYPE)
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find the default activity
    pub async fn find_default(&self) -> Result<Option<ActivityRow>, RepositoryError> {
        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND is_default = true
            "#,
        )
        .bind(ACTIVITY_TYPE)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Check if name is unique within activities for a project scope
    async fn is_name_unique(
        &self,
        name: &str,
        project_id: Option<i64>,
        exclude_id: Option<i64>,
    ) -> Result<bool, RepositoryError> {
        let count = if let Some(pid) = project_id {
            if let Some(id) = exclude_id {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id = $3 AND id != $4
                    "#,
                )
                .bind(ACTIVITY_TYPE)
                .bind(name)
                .bind(pid)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            } else {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id = $3
                    "#,
                )
                .bind(ACTIVITY_TYPE)
                .bind(name)
                .bind(pid)
                .fetch_one(&self.pool)
                .await?
            }
        } else {
            if let Some(id) = exclude_id {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id IS NULL AND id != $3
                    "#,
                )
                .bind(ACTIVITY_TYPE)
                .bind(name)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
            } else {
                sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id IS NULL
                    "#,
                )
                .bind(ACTIVITY_TYPE)
                .bind(name)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(count == 0)
    }

    /// Get the next position for a new activity
    async fn next_position(&self) -> Result<i32, RepositoryError> {
        let max_pos = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(position) FROM enumerations WHERE type = $1",
        )
        .bind(ACTIVITY_TYPE)
        .fetch_one(&self.pool)
        .await?;

        Ok(max_pos.unwrap_or(0) + 1)
    }
}

#[async_trait]
impl Repository<ActivityRow, CreateActivityDto, UpdateActivityDto> for ActivityRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<ActivityRow>, RepositoryError> {
        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE id = $1 AND type = $2
            "#,
        )
        .bind(id)
        .bind(ACTIVITY_TYPE)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<ActivityRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1
            ORDER BY position ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(ACTIVITY_TYPE)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM enumerations WHERE type = $1",
        )
        .bind(ACTIVITY_TYPE)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM enumerations WHERE id = $1 AND type = $2",
        )
        .bind(id)
        .bind(ACTIVITY_TYPE)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateActivityDto) -> Result<ActivityRow, RepositoryError> {
        // Validate name
        if dto.name.trim().is_empty() {
            return Err(RepositoryError::Validation("Name can't be blank".to_string()));
        }

        // Check name uniqueness
        if !self.is_name_unique(&dto.name, dto.project_id, None).await? {
            return Err(RepositoryError::Conflict(
                "Name has already been taken".to_string(),
            ));
        }

        let position = dto.position.unwrap_or_else(|| 0);
        let position = if position == 0 {
            self.next_position().await?
        } else {
            position
        };

        let is_default = dto.is_default.unwrap_or(false);
        let active = dto.active.unwrap_or(true);

        // If setting as default, unset other defaults
        if is_default {
            sqlx::query("UPDATE enumerations SET is_default = false WHERE type = $1")
                .bind(ACTIVITY_TYPE)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
            INSERT INTO enumerations (type, name, position, is_default, active, project_id, parent_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            "#,
        )
        .bind(ACTIVITY_TYPE)
        .bind(&dto.name)
        .bind(position)
        .bind(is_default)
        .bind(active)
        .bind(dto.project_id)
        .bind(dto.parent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, dto: UpdateActivityDto) -> Result<ActivityRow, RepositoryError> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Activity {} not found", id)))?;

        let name = dto.name.unwrap_or(existing.name);
        let position = dto.position.unwrap_or(existing.position);
        let is_default = dto.is_default.unwrap_or(existing.is_default);
        let active = dto.active.unwrap_or(existing.active);

        // Validate name
        if name.trim().is_empty() {
            return Err(RepositoryError::Validation("Name can't be blank".to_string()));
        }

        // Check name uniqueness
        if !self.is_name_unique(&name, existing.project_id, Some(id)).await? {
            return Err(RepositoryError::Conflict(
                "Name has already been taken".to_string(),
            ));
        }

        // If setting as default, unset other defaults
        if is_default && !existing.is_default {
            sqlx::query("UPDATE enumerations SET is_default = false WHERE type = $1")
                .bind(ACTIVITY_TYPE)
                .execute(&self.pool)
                .await?;
        }

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
            UPDATE enumerations
            SET name = $2, position = $3, is_default = $4, active = $5, updated_at = NOW()
            WHERE id = $1 AND type = $6
            RETURNING id, name, position, is_default, active, project_id, parent_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&name)
        .bind(position)
        .bind(is_default)
        .bind(active)
        .bind(ACTIVITY_TYPE)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if activity exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Activity {} not found",
                id
            )));
        }

        // Check if activity is in use (has time entries)
        let in_use_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM time_entries WHERE activity_id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if in_use_count > 0 {
            return Err(RepositoryError::Conflict(format!(
                "Cannot delete activity with {} time entries",
                in_use_count
            )));
        }

        sqlx::query("DELETE FROM enumerations WHERE id = $1 AND type = $2")
            .bind(id)
            .bind(ACTIVITY_TYPE)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_constant() {
        assert_eq!(ACTIVITY_TYPE, "TimeEntryActivity");
    }
}
