//! Type repository
//!
//! Database operations for work package types.
//! Mirrors: app/models/type.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Repository, RepositoryError, RepositoryResult};

/// Type database entity
#[derive(Debug, Clone, FromRow)]
pub struct TypeRow {
    pub id: i64,
    pub name: String,
    pub position: i32,
    pub is_default: bool,
    pub is_in_roadmap: bool,
    pub is_milestone: bool,
    pub is_standard: bool,
    pub color_id: Option<i64>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TypeRow {
    /// Check if this is the default type
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Check if this is the standard (built-in) type
    pub fn is_standard(&self) -> bool {
        self.is_standard
    }

    /// Check if this type is shown in roadmaps
    pub fn is_in_roadmap(&self) -> bool {
        self.is_in_roadmap
    }

    /// Check if this type represents milestones
    pub fn is_milestone(&self) -> bool {
        self.is_milestone
    }
}

/// DTO for creating a type
#[derive(Debug, Clone)]
pub struct CreateTypeDto {
    pub name: String,
    pub position: Option<i32>,
    pub is_default: bool,
    pub is_in_roadmap: bool,
    pub is_milestone: bool,
    pub color_id: Option<i64>,
    pub description: Option<String>,
}

/// DTO for updating a type
#[derive(Debug, Clone, Default)]
pub struct UpdateTypeDto {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub is_in_roadmap: Option<bool>,
    pub is_milestone: Option<bool>,
    pub color_id: Option<i64>,
    pub description: Option<String>,
}

/// Type repository implementation
pub struct TypeRepository {
    pool: PgPool,
}

impl TypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find the standard (built-in) type
    pub async fn find_standard(&self) -> RepositoryResult<Option<TypeRow>> {
        let row = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE is_standard = true
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find default types
    pub async fn find_default(&self) -> RepositoryResult<Vec<TypeRow>> {
        let rows = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE is_default = true
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find milestone types
    pub async fn find_milestones(&self) -> RepositoryResult<Vec<TypeRow>> {
        let rows = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE is_milestone = true
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find types shown in roadmap
    pub async fn find_in_roadmap(&self) -> RepositoryResult<Vec<TypeRow>> {
        let rows = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE is_in_roadmap = true
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find types enabled for a project
    pub async fn find_by_project(&self, project_id: Id) -> RepositoryResult<Vec<TypeRow>> {
        let rows = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT t.id, t.name, t.position, t.is_default, t.is_in_roadmap, t.is_milestone,
                   t.is_standard, t.color_id, t.description, t.created_at, t.updated_at
            FROM types t
            INNER JOIN projects_types pt ON pt.type_id = t.id
            WHERE pt.project_id = $1
            ORDER BY t.position ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find type by name
    pub async fn find_by_name(&self, name: &str) -> RepositoryResult<Option<TypeRow>> {
        let row = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find non-standard types
    pub async fn find_without_standard(&self) -> RepositoryResult<Vec<TypeRow>> {
        let rows = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE is_standard = false
            ORDER BY position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Check if name is unique
    pub async fn is_name_unique(&self, name: &str, exclude_id: Option<Id>) -> RepositoryResult<bool> {
        let query = match exclude_id {
            Some(id) => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM types WHERE LOWER(name) = LOWER($1) AND id != $2)",
            )
            .bind(name)
            .bind(id),
            None => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM types WHERE LOWER(name) = LOWER($1))",
            )
            .bind(name),
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }

    /// Get max position for ordering
    async fn get_max_position(&self) -> RepositoryResult<i32> {
        let max_pos = sqlx::query_scalar::<_, Option<i32>>("SELECT MAX(position) FROM types")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok(max_pos)
    }

    /// Enable type for a project
    pub async fn enable_for_project(&self, type_id: Id, project_id: Id) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO projects_types (project_id, type_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(project_id)
        .bind(type_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Disable type for a project
    pub async fn disable_for_project(&self, type_id: Id, project_id: Id) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            DELETE FROM projects_types
            WHERE project_id = $1 AND type_id = $2
            "#,
        )
        .bind(project_id)
        .bind(type_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl Repository<TypeRow, CreateTypeDto, UpdateTypeDto> for TypeRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<TypeRow>> {
        let row = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<TypeRow>> {
        let rows = sqlx::query_as::<_, TypeRow>(
            r#"
            SELECT id, name, position, is_default, is_in_roadmap, is_milestone,
                   is_standard, color_id, description, created_at, updated_at
            FROM types
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM types")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateTypeDto) -> RepositoryResult<TypeRow> {
        // Check name uniqueness
        if !self.is_name_unique(&dto.name, None).await? {
            return Err(RepositoryError::Conflict(
                "Type name has already been taken".to_string(),
            ));
        }

        let position = match dto.position {
            Some(pos) => pos,
            None => self.get_max_position().await? + 1,
        };

        let row = sqlx::query_as::<_, TypeRow>(
            r#"
            INSERT INTO types (
                name, position, is_default, is_in_roadmap, is_milestone,
                is_standard, color_id, description, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, false, $6, $7, NOW(), NOW()
            )
            RETURNING id, name, position, is_default, is_in_roadmap, is_milestone,
                      is_standard, color_id, description, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(position)
        .bind(dto.is_default)
        .bind(dto.is_in_roadmap)
        .bind(dto.is_milestone)
        .bind(dto.color_id)
        .bind(&dto.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateTypeDto) -> RepositoryResult<TypeRow> {
        // Check name uniqueness if changing
        if let Some(ref name) = dto.name {
            if !self.is_name_unique(name, Some(id)).await? {
                return Err(RepositoryError::Conflict(
                    "Type name has already been taken".to_string(),
                ));
            }
        }

        let row = sqlx::query_as::<_, TypeRow>(
            r#"
            UPDATE types SET
                name = COALESCE($1, name),
                position = COALESCE($2, position),
                is_default = COALESCE($3, is_default),
                is_in_roadmap = COALESCE($4, is_in_roadmap),
                is_milestone = COALESCE($5, is_milestone),
                color_id = COALESCE($6, color_id),
                description = COALESCE($7, description),
                updated_at = NOW()
            WHERE id = $8
            RETURNING id, name, position, is_default, is_in_roadmap, is_milestone,
                      is_standard, color_id, description, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(dto.position)
        .bind(dto.is_default)
        .bind(dto.is_in_roadmap)
        .bind(dto.is_milestone)
        .bind(dto.color_id)
        .bind(&dto.description)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("Type with id {} not found", id)))?;

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        // Check if this is the standard type
        let is_standard = sqlx::query_scalar::<_, bool>(
            "SELECT is_standard FROM types WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if is_standard == Some(true) {
            return Err(RepositoryError::Conflict(
                "Cannot delete the standard type".to_string(),
            ));
        }

        // Check if any work packages use this type
        let has_work_packages = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM work_packages WHERE type_id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if has_work_packages {
            return Err(RepositoryError::Conflict(
                "Cannot delete type: work packages are using this type".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Type with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM types WHERE id = $1)")
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
    fn test_type_is_milestone() {
        let type_row = TypeRow {
            id: 1,
            name: "Milestone".to_string(),
            position: 1,
            is_default: false,
            is_in_roadmap: true,
            is_milestone: true,
            is_standard: false,
            color_id: None,
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(type_row.is_milestone());
        assert!(type_row.is_in_roadmap());
        assert!(!type_row.is_standard());
    }
}
