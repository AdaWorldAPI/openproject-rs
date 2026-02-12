//! Attachments API handlers
//!
//! Mirrors: lib/api/v3/attachments/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{attachment_status, AttachmentRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all attachments
///
/// GET /api/v3/attachments
pub async fn list_attachments(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<AttachmentFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = AttachmentRepository::new(pool.clone());

    let (rows, total) = if let (Some(container_type), Some(container_id)) =
        (&filters.container_type, filters.container_id)
    {
        let result = repo
            .find_by_container(
                container_type,
                container_id,
                op_db::Pagination {
                    limit: pagination.page_size as i64,
                    offset: pagination.offset as i64,
                },
            )
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (result.items, result.total)
    } else if let Some(author_id) = filters.author_id {
        let result = repo
            .find_by_author(
                author_id,
                op_db::Pagination {
                    limit: pagination.page_size as i64,
                    offset: pagination.offset as i64,
                },
            )
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (result.items, result.total)
    } else {
        let rows = repo
            .find_all(pagination.page_size as i64, pagination.offset as i64)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        let total = repo
            .count()
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (rows, total)
    };

    let elements: Vec<AttachmentResponse> = rows
        .into_iter()
        .map(|row| AttachmentResponse::from_row(row))
        .collect();

    let collection = AttachmentCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single attachment
///
/// GET /api/v3/attachments/:id
pub async fn get_attachment(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = AttachmentRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Attachment", id))?;

    Ok(HalResponse(AttachmentResponse::from_row(row)))
}

/// Create a new attachment (metadata only, file upload handled separately)
///
/// POST /api/v3/attachments
pub async fn create_attachment(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateAttachmentRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = AttachmentRepository::new(pool.clone());

    let create_dto = op_db::CreateAttachmentDto {
        container_id: dto.container_id,
        container_type: dto.container_type,
        filename: dto.filename,
        disk_filename: dto.disk_filename,
        filesize: dto.filesize,
        content_type: dto.content_type,
        digest: dto.digest,
        author_id: user.0.id,
        description: dto.description,
        status: dto.status.and_then(|s| attachment_status::from_string(&s)),
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(AttachmentResponse::from_row(row))))
}

/// Update an attachment
///
/// PATCH /api/v3/attachments/:id
pub async fn update_attachment(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateAttachmentRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = AttachmentRepository::new(pool.clone());

    // Check if user is author or admin
    let existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Attachment", id))?;

    if existing.author_id != user.0.id && !user.0.is_admin() {
        return Err(ApiError::forbidden("You can only update your own attachments."));
    }

    let update_dto = op_db::UpdateAttachmentDto {
        container_id: dto.container_id,
        container_type: dto.container_type,
        description: dto.description,
        status: dto.status.and_then(|s| attachment_status::from_string(&s)),
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Attachment", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(AttachmentResponse::from_row(row)))
}

/// Delete an attachment
///
/// DELETE /api/v3/attachments/:id
pub async fn delete_attachment(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = AttachmentRepository::new(pool.clone());

    // Check if user is author or admin
    let existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Attachment", id))?;

    if existing.author_id != user.0.id && !user.0.is_admin() {
        return Err(ApiError::forbidden("You can only delete your own attachments."));
    }

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Attachment", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// List attachments for a work package
///
/// GET /api/v3/work_packages/:work_package_id/attachments
pub async fn list_work_package_attachments(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = AttachmentRepository::new(pool.clone());

    let result = repo
        .find_by_work_package(
            work_package_id,
            op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<AttachmentResponse> = result
        .items
        .into_iter()
        .map(|row| AttachmentResponse::from_row(row))
        .collect();

    let collection = AttachmentCollection {
        type_name: "Collection".into(),
        total: result.total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

// Query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentFilters {
    pub container_type: Option<String>,
    pub container_id: Option<i64>,
    pub author_id: Option<i64>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttachmentRequest {
    pub container_id: Option<i64>,
    pub container_type: Option<String>,
    pub filename: String,
    pub disk_filename: Option<String>,
    pub filesize: i64,
    pub content_type: String,
    pub digest: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttachmentRequest {
    pub container_id: Option<Option<i64>>,
    pub container_type: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<AttachmentResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    filesize: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<String>,
    downloads: i32,
    status: String,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: AttachmentLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AttachmentLinks {
    #[serde(rename = "self")]
    self_link: Link,
    author: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    container: Option<Link>,
    download_location: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl AttachmentResponse {
    fn from_row(row: op_db::AttachmentRow) -> Self {
        let id = row.id;
        let author_id = row.author_id;
        let status = row.status_name().to_string();

        let container_link = match (&row.container_type, row.container_id) {
            (Some(ct), Some(cid)) => {
                let href = match ct.as_str() {
                    "WorkPackage" => format!("/api/v3/work_packages/{}", cid),
                    "Project" => format!("/api/v3/projects/{}", cid),
                    "WikiPage" => format!("/api/v3/wiki_pages/{}", cid),
                    _ => format!("/api/v3/{}s/{}", ct.to_lowercase(), cid),
                };
                Some(Link { href })
            }
            _ => None,
        };

        AttachmentResponse {
            type_name: "Attachment".into(),
            id,
            filename: row.filename,
            filesize: row.filesize,
            content_type: row.content_type,
            description: row.description,
            digest: row.digest,
            downloads: row.downloads,
            status,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            links: AttachmentLinks {
                self_link: Link {
                    href: format!("/api/v3/attachments/{}", id),
                },
                author: Link {
                    href: format!("/api/v3/users/{}", author_id),
                },
                container: container_link,
                download_location: Link {
                    href: format!("/api/v3/attachments/{}/content", id),
                },
            },
        }
    }
}
