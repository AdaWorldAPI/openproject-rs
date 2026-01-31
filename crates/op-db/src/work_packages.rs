//! Work Package repository
//!
//! Database operations for work packages.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool, Row};

use crate::repository::{Pagination, PaginatedResult, Repository, RepositoryError, RepositoryResult};

/// Work package database entity
#[derive(Debug, Clone, FromRow)]
pub struct WorkPackageRow {
    pub id: i64,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: i64,
    pub type_id: i64,
    pub status_id: i64,
    pub priority_id: Option<i64>,
    pub author_id: i64,
    pub assigned_to_id: Option<i64>,
    pub responsible_id: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: i32,
    pub parent_id: Option<i64>,
    pub version_id: Option<i64>,
    pub category_id: Option<i64>,
    pub lock_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a work package
#[derive(Debug, Clone)]
pub struct CreateWorkPackageDto {
    pub subject: String,
    pub description: Option<String>,
    pub project_id: i64,
    pub type_id: i64,
    pub status_id: i64,
    pub priority_id: Option<i64>,
    pub author_id: i64,
    pub assigned_to_id: Option<i64>,
    pub responsible_id: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: i32,
    pub parent_id: Option<i64>,
    pub version_id: Option<i64>,
    pub category_id: Option<i64>,
}

/// DTO for updating a work package
#[derive(Debug, Clone, Default)]
pub struct UpdateWorkPackageDto {
    pub subject: Option<String>,
    pub description: Option<String>,
    pub type_id: Option<i64>,
    pub status_id: Option<i64>,
    pub priority_id: Option<i64>,
    pub assigned_to_id: Option<i64>,
    pub responsible_id: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: Option<i32>,
    pub parent_id: Option<i64>,
    pub version_id: Option<i64>,
    pub category_id: Option<i64>,
    pub lock_version: i32,
}

/// Work package repository implementation
pub struct WorkPackageRepository {
    pool: PgPool,
}

impl WorkPackageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find work packages by project ID
    pub async fn find_by_project(
        &self,
        project_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<WorkPackageRow>> {
        let items = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            SELECT id, subject, description, project_id, type_id, status_id,
                   priority_id, author_id, assigned_to_id, responsible_id,
                   start_date, due_date, estimated_hours, done_ratio,
                   parent_id, version_id, category_id, lock_version,
                   created_at, updated_at
            FROM work_packages
            WHERE project_id = $1
            ORDER BY id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(project_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM work_packages WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find work packages by status
    pub async fn find_by_status(
        &self,
        status_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<WorkPackageRow>> {
        let items = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            SELECT id, subject, description, project_id, type_id, status_id,
                   priority_id, author_id, assigned_to_id, responsible_id,
                   start_date, due_date, estimated_hours, done_ratio,
                   parent_id, version_id, category_id, lock_version,
                   created_at, updated_at
            FROM work_packages
            WHERE status_id = $1
            ORDER BY id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM work_packages WHERE status_id = $1",
        )
        .bind(status_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find work packages assigned to a user
    pub async fn find_by_assignee(
        &self,
        user_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<WorkPackageRow>> {
        let items = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            SELECT id, subject, description, project_id, type_id, status_id,
                   priority_id, author_id, assigned_to_id, responsible_id,
                   start_date, due_date, estimated_hours, done_ratio,
                   parent_id, version_id, category_id, lock_version,
                   created_at, updated_at
            FROM work_packages
            WHERE assigned_to_id = $1
            ORDER BY id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM work_packages WHERE assigned_to_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find children of a work package
    pub async fn find_children(&self, parent_id: Id) -> RepositoryResult<Vec<WorkPackageRow>> {
        let items = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            SELECT id, subject, description, project_id, type_id, status_id,
                   priority_id, author_id, assigned_to_id, responsible_id,
                   start_date, due_date, estimated_hours, done_ratio,
                   parent_id, version_id, category_id, lock_version,
                   created_at, updated_at
            FROM work_packages
            WHERE parent_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    /// Update the status of a work package
    pub async fn update_status(
        &self,
        id: Id,
        status_id: Id,
        lock_version: i32,
    ) -> RepositoryResult<WorkPackageRow> {
        let row = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            UPDATE work_packages
            SET status_id = $1, lock_version = lock_version + 1, updated_at = NOW()
            WHERE id = $2 AND lock_version = $3
            RETURNING id, subject, description, project_id, type_id, status_id,
                      priority_id, author_id, assigned_to_id, responsible_id,
                      start_date, due_date, estimated_hours, done_ratio,
                      parent_id, version_id, category_id, lock_version,
                      created_at, updated_at
            "#,
        )
        .bind(status_id)
        .bind(id)
        .bind(lock_version)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            RepositoryError::Conflict("Work package was modified by another user".to_string())
        })?;

        Ok(row)
    }
}

#[async_trait]
impl Repository<WorkPackageRow, CreateWorkPackageDto, UpdateWorkPackageDto>
    for WorkPackageRepository
{
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<WorkPackageRow>> {
        let row = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            SELECT id, subject, description, project_id, type_id, status_id,
                   priority_id, author_id, assigned_to_id, responsible_id,
                   start_date, due_date, estimated_hours, done_ratio,
                   parent_id, version_id, category_id, lock_version,
                   created_at, updated_at
            FROM work_packages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<WorkPackageRow>> {
        let rows = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            SELECT id, subject, description, project_id, type_id, status_id,
                   priority_id, author_id, assigned_to_id, responsible_id,
                   start_date, due_date, estimated_hours, done_ratio,
                   parent_id, version_id, category_id, lock_version,
                   created_at, updated_at
            FROM work_packages
            ORDER BY id DESC
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM work_packages")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateWorkPackageDto) -> RepositoryResult<WorkPackageRow> {
        let row = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            INSERT INTO work_packages (
                subject, description, project_id, type_id, status_id,
                priority_id, author_id, assigned_to_id, responsible_id,
                start_date, due_date, estimated_hours, done_ratio,
                parent_id, version_id, category_id, lock_version,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, 0, NOW(), NOW()
            )
            RETURNING id, subject, description, project_id, type_id, status_id,
                      priority_id, author_id, assigned_to_id, responsible_id,
                      start_date, due_date, estimated_hours, done_ratio,
                      parent_id, version_id, category_id, lock_version,
                      created_at, updated_at
            "#,
        )
        .bind(&dto.subject)
        .bind(&dto.description)
        .bind(dto.project_id)
        .bind(dto.type_id)
        .bind(dto.status_id)
        .bind(dto.priority_id)
        .bind(dto.author_id)
        .bind(dto.assigned_to_id)
        .bind(dto.responsible_id)
        .bind(dto.start_date)
        .bind(dto.due_date)
        .bind(dto.estimated_hours)
        .bind(dto.done_ratio)
        .bind(dto.parent_id)
        .bind(dto.version_id)
        .bind(dto.category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateWorkPackageDto) -> RepositoryResult<WorkPackageRow> {
        // Build dynamic update query
        let row = sqlx::query_as::<_, WorkPackageRow>(
            r#"
            UPDATE work_packages SET
                subject = COALESCE($1, subject),
                description = COALESCE($2, description),
                type_id = COALESCE($3, type_id),
                status_id = COALESCE($4, status_id),
                priority_id = COALESCE($5, priority_id),
                assigned_to_id = $6,
                responsible_id = $7,
                start_date = $8,
                due_date = $9,
                estimated_hours = $10,
                done_ratio = COALESCE($11, done_ratio),
                parent_id = $12,
                version_id = $13,
                category_id = $14,
                lock_version = lock_version + 1,
                updated_at = NOW()
            WHERE id = $15 AND lock_version = $16
            RETURNING id, subject, description, project_id, type_id, status_id,
                      priority_id, author_id, assigned_to_id, responsible_id,
                      start_date, due_date, estimated_hours, done_ratio,
                      parent_id, version_id, category_id, lock_version,
                      created_at, updated_at
            "#,
        )
        .bind(&dto.subject)
        .bind(&dto.description)
        .bind(dto.type_id)
        .bind(dto.status_id)
        .bind(dto.priority_id)
        .bind(dto.assigned_to_id)
        .bind(dto.responsible_id)
        .bind(dto.start_date)
        .bind(dto.due_date)
        .bind(dto.estimated_hours)
        .bind(dto.done_ratio)
        .bind(dto.parent_id)
        .bind(dto.version_id)
        .bind(dto.category_id)
        .bind(id)
        .bind(dto.lock_version)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            RepositoryError::Conflict("Work package was modified by another user".to_string())
        })?;

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM work_packages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Work package with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM work_packages WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
