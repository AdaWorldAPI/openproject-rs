//! Versions repository
//!
//! Mirrors: app/models/version.rb

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Version status constants
pub mod status {
    pub const OPEN: &str = "open";
    pub const LOCKED: &str = "locked";
    pub const CLOSED: &str = "closed";

    pub fn all() -> Vec<&'static str> {
        vec![OPEN, LOCKED, CLOSED]
    }

    pub fn is_valid(status: &str) -> bool {
        all().contains(&status)
    }
}

/// Version sharing constants
pub mod sharing {
    pub const NONE: &str = "none";
    pub const DESCENDANTS: &str = "descendants";
    pub const HIERARCHY: &str = "hierarchy";
    pub const TREE: &str = "tree";
    pub const SYSTEM: &str = "system";

    pub fn all() -> Vec<&'static str> {
        vec![NONE, DESCENDANTS, HIERARCHY, TREE, SYSTEM]
    }

    pub fn is_valid(sharing: &str) -> bool {
        all().contains(&sharing)
    }
}

/// Version row from database
#[derive(Debug, Clone, FromRow)]
pub struct VersionRow {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub status: String,
    pub sharing: String,
    pub wiki_page_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VersionRow {
    pub fn is_open(&self) -> bool {
        self.status == status::OPEN
    }

    pub fn is_locked(&self) -> bool {
        self.status == status::LOCKED
    }

    pub fn is_closed(&self) -> bool {
        self.status == status::CLOSED
    }

    pub fn is_systemwide(&self) -> bool {
        self.sharing == sharing::SYSTEM
    }
}

/// DTO for creating a version
#[derive(Debug, Clone)]
pub struct CreateVersionDto {
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub sharing: Option<String>,
    pub wiki_page_title: Option<String>,
}

/// DTO for updating a version
#[derive(Debug, Clone, Default)]
pub struct UpdateVersionDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub sharing: Option<String>,
    pub wiki_page_title: Option<String>,
}

/// Version repository
pub struct VersionRepository {
    pool: PgPool,
}

impl VersionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all versions for a project
    pub async fn find_by_project(
        &self,
        project_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<VersionRow>, RepositoryError> {
        let items = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT id, project_id, name, description, effective_date, start_date,
                   status, sharing, wiki_page_title, created_at, updated_at
            FROM versions
            WHERE project_id = $1
            ORDER BY effective_date ASC NULLS LAST, name ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(project_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM versions WHERE project_id = $1",
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

    /// Find open versions for a project
    pub async fn find_open_by_project(
        &self,
        project_id: i64,
    ) -> Result<Vec<VersionRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT id, project_id, name, description, effective_date, start_date,
                   status, sharing, wiki_page_title, created_at, updated_at
            FROM versions
            WHERE project_id = $1 AND status = 'open'
            ORDER BY effective_date ASC NULLS LAST, name ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find systemwide versions
    pub async fn find_systemwide(&self) -> Result<Vec<VersionRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT id, project_id, name, description, effective_date, start_date,
                   status, sharing, wiki_page_title, created_at, updated_at
            FROM versions
            WHERE sharing = 'system'
            ORDER BY effective_date ASC NULLS LAST, name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find versions shared with a project (including hierarchy sharing)
    pub async fn find_shared_with_project(
        &self,
        project_id: i64,
    ) -> Result<Vec<VersionRow>, RepositoryError> {
        // This is a simplified version - full implementation would need project hierarchy
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT v.id, v.project_id, v.name, v.description, v.effective_date, v.start_date,
                   v.status, v.sharing, v.wiki_page_title, v.created_at, v.updated_at
            FROM versions v
            WHERE v.project_id = $1
               OR v.sharing = 'system'
               OR (v.sharing IN ('descendants', 'hierarchy', 'tree')
                   AND v.project_id IN (
                       SELECT parent_id FROM projects WHERE id = $1 AND parent_id IS NOT NULL
                   ))
            ORDER BY v.effective_date ASC NULLS LAST, v.name ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Check if a version name is unique within a project
    pub async fn is_name_unique(
        &self,
        project_id: i64,
        name: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool, RepositoryError> {
        let count = if let Some(id) = exclude_id {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM versions
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
                SELECT COUNT(*) FROM versions
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

    fn validate_dates(
        start_date: Option<NaiveDate>,
        effective_date: Option<NaiveDate>,
    ) -> Result<(), RepositoryError> {
        if let (Some(start), Some(end)) = (start_date, effective_date) {
            if end < start {
                return Err(RepositoryError::Validation(
                    "Effective date must be greater than or equal to start date".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Repository<VersionRow, CreateVersionDto, UpdateVersionDto> for VersionRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<VersionRow>, RepositoryError> {
        let row = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT id, project_id, name, description, effective_date, start_date,
                   status, sharing, wiki_page_title, created_at, updated_at
            FROM versions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<VersionRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT id, project_id, name, description, effective_date, start_date,
                   status, sharing, wiki_page_title, created_at, updated_at
            FROM versions
            ORDER BY effective_date ASC NULLS LAST, name ASC
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM versions")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM versions WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateVersionDto) -> Result<VersionRow, RepositoryError> {
        // Validate name
        if dto.name.trim().is_empty() {
            return Err(RepositoryError::Validation("Name can't be blank".to_string()));
        }

        // Validate status
        let version_status = dto.status.as_deref().unwrap_or(status::OPEN);
        if !status::is_valid(version_status) {
            return Err(RepositoryError::Validation(format!(
                "Status must be one of: {}",
                status::all().join(", ")
            )));
        }

        // Validate sharing
        let version_sharing = dto.sharing.as_deref().unwrap_or(sharing::NONE);
        if !sharing::is_valid(version_sharing) {
            return Err(RepositoryError::Validation(format!(
                "Sharing must be one of: {}",
                sharing::all().join(", ")
            )));
        }

        // Validate dates
        Self::validate_dates(dto.start_date, dto.effective_date)?;

        // Check name uniqueness
        if !self.is_name_unique(dto.project_id, &dto.name, None).await? {
            return Err(RepositoryError::Conflict(
                "Name has already been taken".to_string(),
            ));
        }

        let row = sqlx::query_as::<_, VersionRow>(
            r#"
            INSERT INTO versions (project_id, name, description, effective_date, start_date,
                                  status, sharing, wiki_page_title, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING id, project_id, name, description, effective_date, start_date,
                      status, sharing, wiki_page_title, created_at, updated_at
            "#,
        )
        .bind(dto.project_id)
        .bind(&dto.name)
        .bind(&dto.description)
        .bind(dto.effective_date)
        .bind(dto.start_date)
        .bind(version_status)
        .bind(version_sharing)
        .bind(&dto.wiki_page_title)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(
        &self,
        id: i64,
        dto: UpdateVersionDto,
    ) -> Result<VersionRow, RepositoryError> {
        // Find existing
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Version {} not found", id)))?;

        let name = dto.name.unwrap_or(existing.name);
        let description = dto.description.or(existing.description);
        let effective_date = dto.effective_date.or(existing.effective_date);
        let start_date = dto.start_date.or(existing.start_date);
        let version_status = dto.status.unwrap_or(existing.status);
        let version_sharing = dto.sharing.unwrap_or(existing.sharing);
        let wiki_page_title = dto.wiki_page_title.or(existing.wiki_page_title);

        // Validate name
        if name.trim().is_empty() {
            return Err(RepositoryError::Validation("Name can't be blank".to_string()));
        }

        // Validate status
        if !status::is_valid(&version_status) {
            return Err(RepositoryError::Validation(format!(
                "Status must be one of: {}",
                status::all().join(", ")
            )));
        }

        // Validate sharing
        if !sharing::is_valid(&version_sharing) {
            return Err(RepositoryError::Validation(format!(
                "Sharing must be one of: {}",
                sharing::all().join(", ")
            )));
        }

        // Validate dates
        Self::validate_dates(start_date, effective_date)?;

        // Check name uniqueness
        if !self.is_name_unique(existing.project_id, &name, Some(id)).await? {
            return Err(RepositoryError::Conflict(
                "Name has already been taken".to_string(),
            ));
        }

        let row = sqlx::query_as::<_, VersionRow>(
            r#"
            UPDATE versions
            SET name = $2, description = $3, effective_date = $4, start_date = $5,
                status = $6, sharing = $7, wiki_page_title = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, project_id, name, description, effective_date, start_date,
                      status, sharing, wiki_page_title, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&name)
        .bind(&description)
        .bind(effective_date)
        .bind(start_date)
        .bind(&version_status)
        .bind(&version_sharing)
        .bind(&wiki_page_title)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if version exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Version {} not found",
                id
            )));
        }

        // Check if version has work packages
        let wp_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM work_packages WHERE version_id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        if wp_count > 0 {
            return Err(RepositoryError::Conflict(format!(
                "Cannot delete version with {} work packages",
                wp_count
            )));
        }

        sqlx::query("DELETE FROM versions WHERE id = $1")
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
    fn test_status_constants() {
        assert_eq!(status::OPEN, "open");
        assert_eq!(status::LOCKED, "locked");
        assert_eq!(status::CLOSED, "closed");
        assert!(status::is_valid("open"));
        assert!(status::is_valid("locked"));
        assert!(status::is_valid("closed"));
        assert!(!status::is_valid("unknown"));
    }

    #[test]
    fn test_sharing_constants() {
        assert_eq!(sharing::NONE, "none");
        assert_eq!(sharing::SYSTEM, "system");
        assert!(sharing::is_valid("none"));
        assert!(sharing::is_valid("descendants"));
        assert!(sharing::is_valid("hierarchy"));
        assert!(sharing::is_valid("tree"));
        assert!(sharing::is_valid("system"));
        assert!(!sharing::is_valid("unknown"));
    }
}
