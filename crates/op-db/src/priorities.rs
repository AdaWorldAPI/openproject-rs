//! Priority repository
//!
//! Database operations for work package priorities.
//! Priorities are stored in the `enumerations` table with type = 'IssuePriority'.
//! Mirrors: app/models/issue_priority.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Repository, RepositoryError, RepositoryResult};

const PRIORITY_TYPE: &str = "IssuePriority";

/// Priority database entity
#[derive(Debug, Clone, FromRow)]
pub struct PriorityRow {
    pub id: i64,
    pub name: String,
    pub position: i32,
    pub is_default: bool,
    pub active: bool,
    pub color_id: Option<i64>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PriorityRow {
    /// Check if this is the default priority
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Check if this priority is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if this is a shared (not project-specific) priority
    pub fn is_shared(&self) -> bool {
        self.project_id.is_none()
    }
}

/// DTO for creating a priority
#[derive(Debug, Clone)]
pub struct CreatePriorityDto {
    pub name: String,
    pub position: Option<i32>,
    pub is_default: bool,
    pub active: bool,
    pub color_id: Option<i64>,
    pub project_id: Option<i64>,
}

/// DTO for updating a priority
#[derive(Debug, Clone, Default)]
pub struct UpdatePriorityDto {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
    pub color_id: Option<i64>,
}

/// Priority repository implementation
pub struct PriorityRepository {
    pool: PgPool,
}

impl PriorityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find the default priority
    pub async fn find_default(&self) -> RepositoryResult<Option<PriorityRow>> {
        let row = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND is_default = true
            LIMIT 1
            "#,
        )
        .bind(PRIORITY_TYPE)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find active priorities
    pub async fn find_active(&self) -> RepositoryResult<Vec<PriorityRow>> {
        let rows = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND active = true
            ORDER BY position ASC
            "#,
        )
        .bind(PRIORITY_TYPE)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find shared (not project-specific) priorities
    pub async fn find_shared(&self) -> RepositoryResult<Vec<PriorityRow>> {
        let rows = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND project_id IS NULL
            ORDER BY position ASC
            "#,
        )
        .bind(PRIORITY_TYPE)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find priorities for a specific project
    pub async fn find_by_project(&self, project_id: Id) -> RepositoryResult<Vec<PriorityRow>> {
        let rows = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND (project_id = $2 OR project_id IS NULL)
            ORDER BY position ASC
            "#,
        )
        .bind(PRIORITY_TYPE)
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find priority by name
    pub async fn find_by_name(&self, name: &str) -> RepositoryResult<Option<PriorityRow>> {
        let row = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id IS NULL
            "#,
        )
        .bind(PRIORITY_TYPE)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Check if name is unique within type and project scope
    pub async fn is_name_unique(
        &self,
        name: &str,
        project_id: Option<Id>,
        exclude_id: Option<Id>,
    ) -> RepositoryResult<bool> {
        let query = match (project_id, exclude_id) {
            (Some(pid), Some(eid)) => sqlx::query_scalar::<_, bool>(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id = $3 AND id != $4
                )
                "#,
            )
            .bind(PRIORITY_TYPE)
            .bind(name)
            .bind(pid)
            .bind(eid),
            (Some(pid), None) => sqlx::query_scalar::<_, bool>(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id = $3
                )
                "#,
            )
            .bind(PRIORITY_TYPE)
            .bind(name)
            .bind(pid),
            (None, Some(eid)) => sqlx::query_scalar::<_, bool>(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id IS NULL AND id != $3
                )
                "#,
            )
            .bind(PRIORITY_TYPE)
            .bind(name)
            .bind(eid),
            (None, None) => sqlx::query_scalar::<_, bool>(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM enumerations
                    WHERE type = $1 AND LOWER(name) = LOWER($2) AND project_id IS NULL
                )
                "#,
            )
            .bind(PRIORITY_TYPE)
            .bind(name),
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }

    /// Get max position for ordering
    async fn get_max_position(&self) -> RepositoryResult<i32> {
        let max_pos = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(position) FROM enumerations WHERE type = $1",
        )
        .bind(PRIORITY_TYPE)
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(max_pos)
    }

    /// Clear is_default flag on all other priorities
    async fn clear_default_except(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE enumerations SET is_default = false, updated_at = NOW() WHERE type = $1 AND id != $2",
        )
        .bind(PRIORITY_TYPE)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl Repository<PriorityRow, CreatePriorityDto, UpdatePriorityDto> for PriorityRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<PriorityRow>> {
        let row = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE id = $1 AND type = $2
            "#,
        )
        .bind(id)
        .bind(PRIORITY_TYPE)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<PriorityRow>> {
        let rows = sqlx::query_as::<_, PriorityRow>(
            r#"
            SELECT id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            FROM enumerations
            WHERE type = $1
            ORDER BY position ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(PRIORITY_TYPE)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM enumerations WHERE type = $1",
        )
        .bind(PRIORITY_TYPE)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreatePriorityDto) -> RepositoryResult<PriorityRow> {
        // Check name uniqueness
        if !self.is_name_unique(&dto.name, dto.project_id, None).await? {
            return Err(RepositoryError::Conflict(
                "Priority name has already been taken".to_string(),
            ));
        }

        let position = match dto.position {
            Some(pos) => pos,
            None => self.get_max_position().await? + 1,
        };

        let row = sqlx::query_as::<_, PriorityRow>(
            r#"
            INSERT INTO enumerations (
                type, name, position, is_default, active, color_id, project_id, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW(), NOW()
            )
            RETURNING id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            "#,
        )
        .bind(PRIORITY_TYPE)
        .bind(&dto.name)
        .bind(position)
        .bind(dto.is_default)
        .bind(dto.active)
        .bind(dto.color_id)
        .bind(dto.project_id)
        .fetch_one(&self.pool)
        .await?;

        // If this is the new default, clear default on others
        if dto.is_default {
            self.clear_default_except(row.id).await?;
        }

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdatePriorityDto) -> RepositoryResult<PriorityRow> {
        // Check name uniqueness if changing
        if let Some(ref name) = dto.name {
            let existing = self.find_by_id(id).await?;
            if let Some(existing) = existing {
                if !self.is_name_unique(name, existing.project_id, Some(id)).await? {
                    return Err(RepositoryError::Conflict(
                        "Priority name has already been taken".to_string(),
                    ));
                }
            }
        }

        let row = sqlx::query_as::<_, PriorityRow>(
            r#"
            UPDATE enumerations SET
                name = COALESCE($1, name),
                position = COALESCE($2, position),
                is_default = COALESCE($3, is_default),
                active = COALESCE($4, active),
                color_id = COALESCE($5, color_id),
                updated_at = NOW()
            WHERE id = $6 AND type = $7
            RETURNING id, name, position, is_default, active, color_id, project_id, parent_id, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(dto.position)
        .bind(dto.is_default)
        .bind(dto.active)
        .bind(dto.color_id)
        .bind(id)
        .bind(PRIORITY_TYPE)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("Priority with id {} not found", id)))?;

        // If this is now the default, clear default on others
        if dto.is_default == Some(true) {
            self.clear_default_except(id).await?;
        }

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        // Check if any work packages use this priority
        let has_work_packages = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM work_packages WHERE priority_id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if has_work_packages {
            return Err(RepositoryError::Conflict(
                "Cannot delete priority: work packages are using this priority".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM enumerations WHERE id = $1 AND type = $2")
            .bind(id)
            .bind(PRIORITY_TYPE)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Priority with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM enumerations WHERE id = $1 AND type = $2)",
        )
        .bind(id)
        .bind(PRIORITY_TYPE)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_is_default() {
        let priority = PriorityRow {
            id: 1,
            name: "Normal".to_string(),
            position: 1,
            is_default: true,
            active: true,
            color_id: None,
            project_id: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(priority.is_default());
        assert!(priority.is_active());
        assert!(priority.is_shared());
    }
}
