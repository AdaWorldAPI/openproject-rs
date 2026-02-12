//! Versions API handlers
//!
//! Mirrors: lib/api/v3/versions/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use op_core::traits::Id;
use op_db::{Repository, VersionRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all versions
///
/// GET /api/v3/versions
pub async fn list_versions(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<VersionFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = VersionRepository::new(pool.clone());

    let (rows, total) = if let Some(project_id) = filters.project_id {
        let result = repo
            .find_by_project(
                project_id,
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

    let elements: Vec<VersionResponse> = rows
        .into_iter()
        .map(|row| VersionResponse::from_row(row))
        .collect();

    let collection = VersionCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single version
///
/// GET /api/v3/versions/:id
pub async fn get_version(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = VersionRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Version", id))?;

    Ok(HalResponse(VersionResponse::from_row(row)))
}

/// Create a new version
///
/// POST /api/v3/versions
pub async fn create_version(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateVersionRequest>,
) -> ApiResult<impl IntoResponse> {
    // Only admins or users with manage_versions permission can create versions
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create versions."));
    }

    let pool = state.pool()?;
    let repo = VersionRepository::new(pool.clone());

    let create_dto = op_db::CreateVersionDto {
        project_id: dto.project_id,
        name: dto.name,
        description: dto.description,
        effective_date: dto.effective_date,
        start_date: dto.start_date,
        status: dto.status,
        sharing: dto.sharing,
        wiki_page_title: dto.wiki_page_title,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(VersionResponse::from_row(row))))
}

/// Update a version
///
/// PATCH /api/v3/versions/:id
pub async fn update_version(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateVersionRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update versions."));
    }

    let pool = state.pool()?;
    let repo = VersionRepository::new(pool.clone());

    let update_dto = op_db::UpdateVersionDto {
        name: dto.name,
        description: dto.description,
        effective_date: dto.effective_date,
        start_date: dto.start_date,
        status: dto.status,
        sharing: dto.sharing,
        wiki_page_title: dto.wiki_page_title,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Version", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(VersionResponse::from_row(row)))
}

/// Delete a version
///
/// DELETE /api/v3/versions/:id
pub async fn delete_version(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete versions."));
    }

    let pool = state.pool()?;
    let repo = VersionRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Version", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// List versions for a specific project
///
/// GET /api/v3/projects/:project_id/versions
pub async fn list_project_versions(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = VersionRepository::new(pool.clone());

    let result = repo
        .find_by_project(
            project_id,
            op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<VersionResponse> = result
        .items
        .into_iter()
        .map(|row| VersionResponse::from_row(row))
        .collect();

    let collection = VersionCollection {
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
pub struct VersionFilters {
    pub project_id: Option<i64>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVersionRequest {
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub sharing: Option<String>,
    pub wiki_page_title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVersionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub sharing: Option<String>,
    pub wiki_page_title: Option<String>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<VersionResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<String>,
    status: String,
    sharing: String,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: VersionLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VersionLinks {
    #[serde(rename = "self")]
    self_link: Link,
    defining_project: Link,
    available_in_projects: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl VersionResponse {
    fn from_row(row: op_db::VersionRow) -> Self {
        let id = row.id;
        let project_id = row.project_id;

        VersionResponse {
            type_name: "Version".into(),
            id,
            name: row.name,
            description: row.description,
            start_date: row.start_date.map(|d| d.to_string()),
            end_date: row.effective_date.map(|d| d.to_string()),
            status: row.status,
            sharing: row.sharing,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            links: VersionLinks {
                self_link: Link {
                    href: format!("/api/v3/versions/{}", id),
                },
                defining_project: Link {
                    href: format!("/api/v3/projects/{}", project_id),
                },
                available_in_projects: Link {
                    href: format!("/api/v3/versions/{}/projects", id),
                },
            },
        }
    }
}
