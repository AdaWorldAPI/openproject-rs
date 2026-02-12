//! Attachments repository
//!
//! Mirrors: app/models/attachment.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Attachment status enum
pub mod status {
    pub const UPLOADED: i32 = 0;
    pub const PREPARED: i32 = 1;
    pub const SCANNED: i32 = 2;
    pub const QUARANTINED: i32 = 3;
    pub const RESCAN: i32 = 4;

    pub fn to_string(status: i32) -> &'static str {
        match status {
            UPLOADED => "uploaded",
            PREPARED => "prepared",
            SCANNED => "scanned",
            QUARANTINED => "quarantined",
            RESCAN => "rescan",
            _ => "unknown",
        }
    }

    pub fn from_string(s: &str) -> Option<i32> {
        match s {
            "uploaded" => Some(UPLOADED),
            "prepared" => Some(PREPARED),
            "scanned" => Some(SCANNED),
            "quarantined" => Some(QUARANTINED),
            "rescan" => Some(RESCAN),
            _ => None,
        }
    }
}

/// Attachment row from database
#[derive(Debug, Clone, FromRow)]
pub struct AttachmentRow {
    pub id: i64,
    pub container_id: Option<i64>,
    pub container_type: Option<String>,
    pub filename: Option<String>,
    pub disk_filename: Option<String>,
    pub filesize: i64,
    pub content_type: Option<String>,
    pub digest: Option<String>,
    pub downloads: i32,
    pub author_id: i64,
    pub description: Option<String>,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AttachmentRow {
    /// Get status as string
    pub fn status_name(&self) -> &'static str {
        status::to_string(self.status)
    }

    /// Check if attachment is pending direct upload
    pub fn is_pending_direct_upload(&self) -> bool {
        self.digest.as_deref() == Some("") && self.downloads == -1
    }

    /// Check if attachment is ready
    pub fn is_ready(&self) -> bool {
        self.status == status::UPLOADED || self.status == status::SCANNED
    }

    /// Check if attachment is quarantined
    pub fn is_quarantined(&self) -> bool {
        self.status == status::QUARANTINED
    }

    /// Check if this is an image
    pub fn is_image(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

    /// Check if this is a PDF
    pub fn is_pdf(&self) -> bool {
        self.content_type.as_deref() == Some("application/pdf")
    }

    /// Get file extension
    pub fn extension(&self) -> Option<String> {
        self.filename.as_ref().and_then(|f| {
            f.rfind('.').map(|i| f[i..].to_string())
        })
    }
}

/// DTO for creating an attachment
#[derive(Debug, Clone)]
pub struct CreateAttachmentDto {
    pub container_id: Option<i64>,
    pub container_type: Option<String>,
    pub filename: String,
    pub disk_filename: Option<String>,
    pub filesize: i64,
    pub content_type: String,
    pub digest: Option<String>,
    pub author_id: i64,
    pub description: Option<String>,
    pub status: Option<i32>,
}

/// DTO for updating an attachment
#[derive(Debug, Clone, Default)]
pub struct UpdateAttachmentDto {
    pub container_id: Option<Option<i64>>,
    pub container_type: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub status: Option<i32>,
}

/// Attachment repository
pub struct AttachmentRepository {
    pool: PgPool,
}

impl AttachmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find attachments for a container
    pub async fn find_by_container(
        &self,
        container_type: &str,
        container_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<AttachmentRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, container_id, container_type, filename, disk_filename,
                   filesize, content_type, digest, downloads, author_id,
                   description, status, created_at, updated_at
            FROM attachments
            WHERE container_type = $1 AND container_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(container_type)
        .bind(container_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM attachments WHERE container_type = $1 AND container_id = $2",
        )
        .bind(container_type)
        .bind(container_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find attachments for a work package
    pub async fn find_by_work_package(
        &self,
        work_package_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<AttachmentRow>, RepositoryError> {
        self.find_by_container("WorkPackage", work_package_id, pagination).await
    }

    /// Find attachments for a project
    pub async fn find_by_project(
        &self,
        project_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<AttachmentRow>, RepositoryError> {
        self.find_by_container("Project", project_id, pagination).await
    }

    /// Find attachments by author
    pub async fn find_by_author(
        &self,
        author_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<AttachmentRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, container_id, container_type, filename, disk_filename,
                   filesize, content_type, digest, downloads, author_id,
                   description, status, created_at, updated_at
            FROM attachments
            WHERE author_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(author_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM attachments WHERE author_id = $1",
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find pending direct upload attachments
    pub async fn find_pending_direct_uploads(&self) -> Result<Vec<AttachmentRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, container_id, container_type, filename, disk_filename,
                   filesize, content_type, digest, downloads, author_id,
                   description, status, created_at, updated_at
            FROM attachments
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(status::PREPARED)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Increment download count
    pub async fn increment_downloads(&self, id: i64) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE attachments SET downloads = downloads + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Find orphaned attachments (no container)
    pub async fn find_orphaned(&self) -> Result<Vec<AttachmentRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, container_id, container_type, filename, disk_filename,
                   filesize, content_type, digest, downloads, author_id,
                   description, status, created_at, updated_at
            FROM attachments
            WHERE container_id IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

#[async_trait]
impl Repository<AttachmentRow, CreateAttachmentDto, UpdateAttachmentDto> for AttachmentRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<AttachmentRow>, RepositoryError> {
        let row = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, container_id, container_type, filename, disk_filename,
                   filesize, content_type, digest, downloads, author_id,
                   description, status, created_at, updated_at
            FROM attachments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<AttachmentRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, AttachmentRow>(
            r#"
            SELECT id, container_id, container_type, filename, disk_filename,
                   filesize, content_type, digest, downloads, author_id,
                   description, status, created_at, updated_at
            FROM attachments
            ORDER BY created_at DESC
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM attachments")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM attachments WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateAttachmentDto) -> Result<AttachmentRow, RepositoryError> {
        // Validate required fields
        if dto.filename.is_empty() {
            return Err(RepositoryError::Validation(
                "Filename can't be blank".to_string(),
            ));
        }

        if dto.content_type.is_empty() {
            return Err(RepositoryError::Validation(
                "Content type can't be blank".to_string(),
            ));
        }

        let status = dto.status.unwrap_or(status::UPLOADED);

        let row = sqlx::query_as::<_, AttachmentRow>(
            r#"
            INSERT INTO attachments (
                container_id, container_type, filename, disk_filename,
                filesize, content_type, digest, downloads, author_id,
                description, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8, $9, $10, NOW(), NOW())
            RETURNING id, container_id, container_type, filename, disk_filename,
                      filesize, content_type, digest, downloads, author_id,
                      description, status, created_at, updated_at
            "#,
        )
        .bind(dto.container_id)
        .bind(&dto.container_type)
        .bind(&dto.filename)
        .bind(&dto.disk_filename)
        .bind(dto.filesize)
        .bind(&dto.content_type)
        .bind(&dto.digest)
        .bind(dto.author_id)
        .bind(&dto.description)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, dto: UpdateAttachmentDto) -> Result<AttachmentRow, RepositoryError> {
        // Verify attachment exists
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Attachment {} not found", id)))?;

        let new_container_id = match dto.container_id {
            Some(cid) => cid,
            None => existing.container_id,
        };

        let new_container_type = match dto.container_type {
            Some(ct) => ct,
            None => existing.container_type,
        };

        let new_description = match dto.description {
            Some(d) => d,
            None => existing.description,
        };

        let new_status = dto.status.unwrap_or(existing.status);

        let row = sqlx::query_as::<_, AttachmentRow>(
            r#"
            UPDATE attachments
            SET container_id = $2, container_type = $3, description = $4,
                status = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING id, container_id, container_type, filename, disk_filename,
                      filesize, content_type, digest, downloads, author_id,
                      description, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_container_id)
        .bind(&new_container_type)
        .bind(&new_description)
        .bind(new_status)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if attachment exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Attachment {} not found",
                id
            )));
        }

        sqlx::query("DELETE FROM attachments WHERE id = $1")
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
    fn test_status_conversion() {
        assert_eq!(status::to_string(status::UPLOADED), "uploaded");
        assert_eq!(status::to_string(status::QUARANTINED), "quarantined");
        assert_eq!(status::from_string("uploaded"), Some(status::UPLOADED));
        assert_eq!(status::from_string("invalid"), None);
    }

    #[test]
    fn test_attachment_is_image() {
        let mut attachment = AttachmentRow {
            id: 1,
            container_id: Some(1),
            container_type: Some("WorkPackage".to_string()),
            filename: Some("test.png".to_string()),
            disk_filename: None,
            filesize: 1000,
            content_type: Some("image/png".to_string()),
            digest: Some("abc123".to_string()),
            downloads: 0,
            author_id: 1,
            description: None,
            status: status::UPLOADED,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(attachment.is_image());

        attachment.content_type = Some("application/pdf".to_string());
        assert!(!attachment.is_image());
        assert!(attachment.is_pdf());
    }
}
