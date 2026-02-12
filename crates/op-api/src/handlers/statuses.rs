//! Statuses API handlers
//!
//! Mirrors: lib/api/v3/statuses/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{Repository, StatusRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all statuses
///
/// GET /api/v3/statuses
pub async fn list_statuses(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = StatusRepository::new(pool.clone());

    let rows = repo
        .find_all(pagination.page_size as i64, pagination.offset as i64)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let total = repo
        .count()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<StatusResponse> = rows
        .into_iter()
        .map(|row| StatusResponse::from_row(row))
        .collect();

    let collection = StatusCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single status
///
/// GET /api/v3/statuses/:id
pub async fn get_status(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = StatusRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Status", id))?;

    Ok(HalResponse(StatusResponse::from_row(row)))
}

/// Create a new status (admin only)
///
/// POST /api/v3/statuses
pub async fn create_status(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateStatusRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create statuses."));
    }

    let pool = state.pool()?;
    let repo = StatusRepository::new(pool.clone());

    let create_dto = op_db::CreateStatusDto {
        name: dto.name,
        is_closed: dto.is_closed.unwrap_or(false),
        is_default: dto.is_default.unwrap_or(false),
        is_readonly: dto.is_readonly.unwrap_or(false),
        position: dto.position,
        default_done_ratio: dto.default_done_ratio.unwrap_or(0),
        color_id: dto.color_id,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(StatusResponse::from_row(row))))
}

/// Update a status (admin only)
///
/// PATCH /api/v3/statuses/:id
pub async fn update_status(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateStatusRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update statuses."));
    }

    let pool = state.pool()?;
    let repo = StatusRepository::new(pool.clone());

    let update_dto = op_db::UpdateStatusDto {
        name: dto.name,
        is_closed: dto.is_closed,
        is_default: dto.is_default,
        is_readonly: dto.is_readonly,
        position: dto.position,
        default_done_ratio: dto.default_done_ratio,
        color_id: dto.color_id,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Status", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(StatusResponse::from_row(row)))
}

/// Delete a status (admin only)
///
/// DELETE /api/v3/statuses/:id
pub async fn delete_status(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete statuses."));
    }

    let pool = state.pool()?;
    let repo = StatusRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Status", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStatusRequest {
    pub name: String,
    pub is_closed: Option<bool>,
    pub is_default: Option<bool>,
    pub is_readonly: Option<bool>,
    pub position: Option<i32>,
    pub default_done_ratio: Option<i32>,
    pub color_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub name: Option<String>,
    pub is_closed: Option<bool>,
    pub is_default: Option<bool>,
    pub is_readonly: Option<bool>,
    pub position: Option<i32>,
    pub default_done_ratio: Option<i32>,
    pub color_id: Option<i64>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<StatusResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    is_closed: bool,
    is_default: bool,
    is_readonly: bool,
    position: i32,
    default_done_ratio: i32,
    #[serde(rename = "_links")]
    links: StatusLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusLinks {
    #[serde(rename = "self")]
    self_link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Link>,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl StatusResponse {
    fn from_row(row: op_db::StatusRow) -> Self {
        let color_link = row.color_id.map(|cid| Link {
            href: format!("/api/v3/colors/{}", cid),
        });

        StatusResponse {
            type_name: "Status".into(),
            id: row.id,
            name: row.name,
            is_closed: row.is_closed,
            is_default: row.is_default,
            is_readonly: row.is_readonly,
            position: row.position,
            default_done_ratio: row.default_done_ratio,
            links: StatusLinks {
                self_link: Link {
                    href: format!("/api/v3/statuses/{}", row.id),
                },
                color: color_link,
            },
        }
    }
}
