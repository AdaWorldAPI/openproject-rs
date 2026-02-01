//! Project repository
//!
//! Database operations for projects.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Pagination, PaginatedResult, Repository, RepositoryError, RepositoryResult};

/// Project database entity
#[derive(Debug, Clone, FromRow)]
pub struct ProjectRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub identifier: String,
    pub public: bool,
    pub parent_id: Option<i64>,
    pub lft: i32,
    pub rgt: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProjectRow {
    /// Get the depth of the project in the tree
    pub fn depth(&self) -> i32 {
        // In nested set model, depth = (lft - 1) / 2 approximately
        // This is a simplified calculation
        0
    }

    /// Check if this project is a root project
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    /// Check if project has children
    pub fn has_children(&self) -> bool {
        self.rgt - self.lft > 1
    }
}

/// DTO for creating a project
#[derive(Debug, Clone)]
pub struct CreateProjectDto {
    pub name: String,
    pub description: Option<String>,
    pub identifier: String,
    pub public: bool,
    pub parent_id: Option<i64>,
    pub active: bool,
}

/// DTO for updating a project
#[derive(Debug, Clone, Default)]
pub struct UpdateProjectDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
    pub parent_id: Option<i64>,
    pub active: Option<bool>,
}

/// Project repository implementation
pub struct ProjectRepository {
    pool: PgPool,
}

impl ProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find a project by identifier
    pub async fn find_by_identifier(&self, identifier: &str) -> RepositoryResult<Option<ProjectRow>> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE identifier = $1
            "#,
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find root projects (no parent)
    pub async fn find_roots(
        &self,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<ProjectRow>> {
        let items = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE parent_id IS NULL
            ORDER BY lft ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects WHERE parent_id IS NULL")
                .fetch_one(&self.pool)
                .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find children of a project
    pub async fn find_children(&self, parent_id: Id) -> RepositoryResult<Vec<ProjectRow>> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE parent_id = $1
            ORDER BY lft ASC
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find all ancestors of a project
    pub async fn find_ancestors(&self, project: &ProjectRow) -> RepositoryResult<Vec<ProjectRow>> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE lft < $1 AND rgt > $2
            ORDER BY lft ASC
            "#,
        )
        .bind(project.lft)
        .bind(project.rgt)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find all descendants of a project
    pub async fn find_descendants(&self, project: &ProjectRow) -> RepositoryResult<Vec<ProjectRow>> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE lft > $1 AND rgt < $2
            ORDER BY lft ASC
            "#,
        )
        .bind(project.lft)
        .bind(project.rgt)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find active projects
    pub async fn find_active(
        &self,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<ProjectRow>> {
        let items = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE active = true
            ORDER BY lft ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects WHERE active = true")
                .fetch_one(&self.pool)
                .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find public projects
    pub async fn find_public(
        &self,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<ProjectRow>> {
        let items = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE public = true AND active = true
            ORDER BY lft ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM projects WHERE public = true AND active = true",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Check if identifier is unique
    pub async fn is_identifier_unique(
        &self,
        identifier: &str,
        exclude_id: Option<Id>,
    ) -> RepositoryResult<bool> {
        let query = match exclude_id {
            Some(id) => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM projects WHERE identifier = $1 AND id != $2)",
            )
            .bind(identifier)
            .bind(id),
            None => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM projects WHERE identifier = $1)",
            )
            .bind(identifier),
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }

    /// Archive a project (set active = false)
    pub async fn archive(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query("UPDATE projects SET active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Unarchive a project (set active = true)
    pub async fn unarchive(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query("UPDATE projects SET active = true, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get projects a user has access to
    pub async fn find_visible_to_user(
        &self,
        user_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<ProjectRow>> {
        let items = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT DISTINCT p.id, p.name, p.description, p.identifier, p.public, p.parent_id,
                   p.lft, p.rgt, p.active, p.created_at, p.updated_at
            FROM projects p
            LEFT JOIN members m ON m.project_id = p.id AND m.user_id = $1
            WHERE p.active = true AND (p.public = true OR m.id IS NOT NULL)
            ORDER BY p.lft ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT p.id)
            FROM projects p
            LEFT JOIN members m ON m.project_id = p.id AND m.user_id = $1
            WHERE p.active = true AND (p.public = true OR m.id IS NOT NULL)
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }
}

#[async_trait]
impl Repository<ProjectRow, CreateProjectDto, UpdateProjectDto> for ProjectRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<ProjectRow>> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<ProjectRow>> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT id, name, description, identifier, public, parent_id,
                   lft, rgt, active, created_at, updated_at
            FROM projects
            ORDER BY lft ASC
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateProjectDto) -> RepositoryResult<ProjectRow> {
        // For nested set, we need to calculate lft/rgt
        // This is a simplified version - real implementation needs proper nested set management
        let max_rgt = sqlx::query_scalar::<_, Option<i32>>("SELECT MAX(rgt) FROM projects")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        let lft = max_rgt + 1;
        let rgt = max_rgt + 2;

        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            INSERT INTO projects (
                name, description, identifier, public, parent_id,
                lft, rgt, active, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()
            )
            RETURNING id, name, description, identifier, public, parent_id,
                      lft, rgt, active, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.description)
        .bind(&dto.identifier)
        .bind(dto.public)
        .bind(dto.parent_id)
        .bind(lft)
        .bind(rgt)
        .bind(dto.active)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateProjectDto) -> RepositoryResult<ProjectRow> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            UPDATE projects SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                public = COALESCE($3, public),
                active = COALESCE($4, active),
                updated_at = NOW()
            WHERE id = $5
            RETURNING id, name, description, identifier, public, parent_id,
                      lft, rgt, active, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.description)
        .bind(dto.public)
        .bind(dto.active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("Project with id {} not found", id)))?;

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        // Check if project has children
        let has_children = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE parent_id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if has_children {
            return Err(RepositoryError::Conflict(
                "Cannot delete project with children".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Project with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}
