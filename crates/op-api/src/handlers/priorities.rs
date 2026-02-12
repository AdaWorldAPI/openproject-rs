//! Priorities API handlers
//!
//! Mirrors: lib/api/v3/priorities/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{PriorityRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all priorities
///
/// GET /api/v3/priorities
pub async fn list_priorities(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = PriorityRepository::new(pool.clone());

    let rows = repo
        .find_all(pagination.page_size as i64, pagination.offset as i64)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let total = repo
        .count()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<PriorityResponse> = rows
        .into_iter()
        .map(|row| PriorityResponse::from_row(row))
        .collect();

    let collection = PriorityCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single priority
///
/// GET /api/v3/priorities/:id
pub async fn get_priority(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = PriorityRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Priority", id))?;

    Ok(HalResponse(PriorityResponse::from_row(row)))
}

/// Create a new priority (admin only)
///
/// POST /api/v3/priorities
pub async fn create_priority(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreatePriorityRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create priorities."));
    }

    let pool = state.pool()?;
    let repo = PriorityRepository::new(pool.clone());

    let create_dto = op_db::CreatePriorityDto {
        name: dto.name,
        position: dto.position,
        is_default: dto.is_default.unwrap_or(false),
        active: dto.active.unwrap_or(true),
        color_id: dto.color_id,
        project_id: None, // Priorities created via API are shared
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(PriorityResponse::from_row(row))))
}

/// Update a priority (admin only)
///
/// PATCH /api/v3/priorities/:id
pub async fn update_priority(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdatePriorityRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update priorities."));
    }

    let pool = state.pool()?;
    let repo = PriorityRepository::new(pool.clone());

    let update_dto = op_db::UpdatePriorityDto {
        name: dto.name,
        position: dto.position,
        is_default: dto.is_default,
        active: dto.active,
        color_id: dto.color_id,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Priority", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(PriorityResponse::from_row(row)))
}

/// Delete a priority (admin only)
///
/// DELETE /api/v3/priorities/:id
pub async fn delete_priority(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete priorities."));
    }

    let pool = state.pool()?;
    let repo = PriorityRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Priority", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePriorityRequest {
    pub name: String,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
    pub color_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePriorityRequest {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
    pub color_id: Option<i64>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PriorityCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<PriorityResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PriorityResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    position: i32,
    is_default: bool,
    is_active: bool,
    #[serde(rename = "_links")]
    links: PriorityLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PriorityLinks {
    #[serde(rename = "self")]
    self_link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Link>,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl PriorityResponse {
    fn from_row(row: op_db::PriorityRow) -> Self {
        let color_link = row.color_id.map(|cid| Link {
            href: format!("/api/v3/colors/{}", cid),
        });

        PriorityResponse {
            type_name: "Priority".into(),
            id: row.id,
            name: row.name,
            position: row.position,
            is_default: row.is_default,
            is_active: row.active,
            links: PriorityLinks {
                self_link: Link {
                    href: format!("/api/v3/priorities/{}", row.id),
                },
                color: color_link,
            },
        }
    }
}
