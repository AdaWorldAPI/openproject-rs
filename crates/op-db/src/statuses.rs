//! Status repository
//!
//! Database operations for work package statuses.
//! Mirrors: app/models/status.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Repository, RepositoryError, RepositoryResult};

/// Status database entity
#[derive(Debug, Clone, FromRow)]
pub struct StatusRow {
    pub id: i64,
    pub name: String,
    pub is_closed: bool,
    pub is_default: bool,
    pub is_readonly: bool,
    pub position: i32,
    pub default_done_ratio: i32,
    pub color_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StatusRow {
    /// Check if this status means the work package is closed
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// Check if this is the default status for new work packages
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Check if work packages in this status are read-only
    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }
}

/// DTO for creating a status
#[derive(Debug, Clone)]
pub struct CreateStatusDto {
    pub name: String,
    pub is_closed: bool,
    pub is_default: bool,
    pub is_readonly: bool,
    pub position: Option<i32>,
    pub default_done_ratio: i32,
    pub color_id: Option<i64>,
}

/// DTO for updating a status
#[derive(Debug, Clone, Default)]
pub struct UpdateStatusDto {
    pub name: Option<String>,
    pub is_closed: Option<bool>,
    pub is_default: Option<bool>,
    pub is_readonly: Option<bool>,
    pub position: Option<i32>,
    pub default_done_ratio: Option<i32>,
    pub color_id: Option<i64>,
}

/// Status repository implementation
pub struct StatusRepository {
    pool: PgPool,
}

impl StatusRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find the default status for new work packages
    pub async fn find_default(&self) -> RepositoryResult<Option<StatusRow>> {
        let row = sqlx::query_as::<_, StatusRow>(
            r#"
            SELECT id, name, is_closed, is_default, is_readonly, position,
                   default_done_ratio, color_id, created_at, updated_at
            FROM statuses
            WHERE is_default = true
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find closed statuses
    pub async fn find_closed(&self) -> RepositoryResult<Vec<StatusRow>> {
        let rows = sqlx::query_as::<_, StatusRow>(
            r#"
            SELECT id, name, is_closed, is_default, is_readonly, position,
                   default_done_ratio, color_id, created_at, updated_at
            FROM statuses
            WHERE is_closed = true
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find open (not closed) statuses
    pub async fn find_open(&self) -> RepositoryResult<Vec<StatusRow>> {
        let rows = sqlx::query_as::<_, StatusRow>(
            r#"
            SELECT id, name, is_closed, is_default, is_readonly, position,
                   default_done_ratio, color_id, created_at, updated_at
            FROM statuses
            WHERE is_closed = false
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find status by name
    pub async fn find_by_name(&self, name: &str) -> RepositoryResult<Option<StatusRow>> {
        let row = sqlx::query_as::<_, StatusRow>(
            r#"
            SELECT id, name, is_closed, is_default, is_readonly, position,
                   default_done_ratio, color_id, created_at, updated_at
            FROM statuses
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Check if name is unique
    pub async fn is_name_unique(&self, name: &str, exclude_id: Option<Id>) -> RepositoryResult<bool> {
        let query = match exclude_id {
            Some(id) => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM statuses WHERE LOWER(name) = LOWER($1) AND id != $2)",
            )
            .bind(name)
            .bind(id),
            None => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM statuses WHERE LOWER(name) = LOWER($1))",
            )
            .bind(name),
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }

    /// Get max position for ordering
    async fn get_max_position(&self) -> RepositoryResult<i32> {
        let max_pos = sqlx::query_scalar::<_, Option<i32>>("SELECT MAX(position) FROM statuses")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok(max_pos)
    }

    /// Clear is_default flag on all other statuses
    async fn clear_default_except(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query("UPDATE statuses SET is_default = false, updated_at = NOW() WHERE id != $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Repository<StatusRow, CreateStatusDto, UpdateStatusDto> for StatusRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<StatusRow>> {
        let row = sqlx::query_as::<_, StatusRow>(
            r#"
            SELECT id, name, is_closed, is_default, is_readonly, position,
                   default_done_ratio, color_id, created_at, updated_at
            FROM statuses
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<StatusRow>> {
        let rows = sqlx::query_as::<_, StatusRow>(
            r#"
            SELECT id, name, is_closed, is_default, is_readonly, position,
                   default_done_ratio, color_id, created_at, updated_at
            FROM statuses
            ORDER BY position ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM statuses")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateStatusDto) -> RepositoryResult<StatusRow> {
        // Validate done ratio
        if dto.default_done_ratio < 0 || dto.default_done_ratio > 100 {
            return Err(RepositoryError::Validation(
                "default_done_ratio must be between 0 and 100".to_string(),
            ));
        }

        // Validate: default status cannot be readonly
        if dto.is_default && dto.is_readonly {
            return Err(RepositoryError::Validation(
                "Default status cannot be read-only".to_string(),
            ));
        }

        // Check name uniqueness
        if !self.is_name_unique(&dto.name, None).await? {
            return Err(RepositoryError::Conflict(
                "Status name has already been taken".to_string(),
            ));
        }

        let position = match dto.position {
            Some(pos) => pos,
            None => self.get_max_position().await? + 1,
        };

        let row = sqlx::query_as::<_, StatusRow>(
            r#"
            INSERT INTO statuses (
                name, is_closed, is_default, is_readonly, position,
                default_done_ratio, color_id, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW(), NOW()
            )
            RETURNING id, name, is_closed, is_default, is_readonly, position,
                      default_done_ratio, color_id, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(dto.is_closed)
        .bind(dto.is_default)
        .bind(dto.is_readonly)
        .bind(position)
        .bind(dto.default_done_ratio)
        .bind(dto.color_id)
        .fetch_one(&self.pool)
        .await?;

        // If this is the new default, clear default on others
        if dto.is_default {
            self.clear_default_except(row.id).await?;
        }

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateStatusDto) -> RepositoryResult<StatusRow> {
        // Validate done ratio if provided
        if let Some(ratio) = dto.default_done_ratio {
            if ratio < 0 || ratio > 100 {
                return Err(RepositoryError::Validation(
                    "default_done_ratio must be between 0 and 100".to_string(),
                ));
            }
        }

        // Check name uniqueness if changing
        if let Some(ref name) = dto.name {
            if !self.is_name_unique(name, Some(id)).await? {
                return Err(RepositoryError::Conflict(
                    "Status name has already been taken".to_string(),
                ));
            }
        }

        let row = sqlx::query_as::<_, StatusRow>(
            r#"
            UPDATE statuses SET
                name = COALESCE($1, name),
                is_closed = COALESCE($2, is_closed),
                is_default = COALESCE($3, is_default),
                is_readonly = COALESCE($4, is_readonly),
                position = COALESCE($5, position),
                default_done_ratio = COALESCE($6, default_done_ratio),
                color_id = COALESCE($7, color_id),
                updated_at = NOW()
            WHERE id = $8
            RETURNING id, name, is_closed, is_default, is_readonly, position,
                      default_done_ratio, color_id, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(dto.is_closed)
        .bind(dto.is_default)
        .bind(dto.is_readonly)
        .bind(dto.position)
        .bind(dto.default_done_ratio)
        .bind(dto.color_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("Status with id {} not found", id)))?;

        // If this is now the default, clear default on others
        if dto.is_default == Some(true) {
            self.clear_default_except(id).await?;
        }

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        // Check if any work packages use this status
        let has_work_packages = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM work_packages WHERE status_id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if has_work_packages {
            return Err(RepositoryError::Conflict(
                "Cannot delete status: work packages are using this status".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM statuses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Status with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM statuses WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_closed() {
        let status = StatusRow {
            id: 1,
            name: "Closed".to_string(),
            is_closed: true,
            is_default: false,
            is_readonly: false,
            position: 1,
            default_done_ratio: 100,
            color_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(status.is_closed());
    }

    #[test]
    fn test_status_is_default() {
        let status = StatusRow {
            id: 1,
            name: "New".to_string(),
            is_closed: false,
            is_default: true,
            is_readonly: false,
            position: 1,
            default_done_ratio: 0,
            color_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(status.is_default());
        assert!(!status.is_closed());
    }
}
