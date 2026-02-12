//! Time Entry Activities API handlers
//!
//! Mirrors: lib/api/v3/time_entries/activities/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{ActivityRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all time entry activities
///
/// GET /api/v3/time_entries/activities
pub async fn list_activities(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<ActivityFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = ActivityRepository::new(pool.clone());

    let rows = if let Some(project_id) = filters.project_id {
        repo.find_by_project(project_id)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
    } else {
        repo.find_all(pagination.page_size as i64, pagination.offset as i64)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
    };

    let total = if filters.project_id.is_some() {
        rows.len() as i64
    } else {
        repo.count()
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
    };

    let elements: Vec<ActivityResponse> = rows
        .into_iter()
        .map(|row| ActivityResponse::from_row(row))
        .collect();

    let collection = ActivityCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single activity
///
/// GET /api/v3/time_entries/activities/:id
pub async fn get_activity(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = ActivityRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("TimeEntryActivity", id))?;

    Ok(HalResponse(ActivityResponse::from_row(row)))
}

/// Create a new activity (admin only)
///
/// POST /api/v3/time_entries/activities
pub async fn create_activity(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateActivityRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create activities."));
    }

    let pool = state.pool()?;
    let repo = ActivityRepository::new(pool.clone());

    let create_dto = op_db::CreateActivityDto {
        name: dto.name,
        position: dto.position,
        is_default: dto.is_default,
        active: dto.active,
        project_id: dto.project_id,
        parent_id: dto.parent_id,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(ActivityResponse::from_row(row))))
}

/// Update an activity (admin only)
///
/// PATCH /api/v3/time_entries/activities/:id
pub async fn update_activity(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateActivityRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update activities."));
    }

    let pool = state.pool()?;
    let repo = ActivityRepository::new(pool.clone());

    let update_dto = op_db::UpdateActivityDto {
        name: dto.name,
        position: dto.position,
        is_default: dto.is_default,
        active: dto.active,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("TimeEntryActivity", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(ActivityResponse::from_row(row)))
}

/// Delete an activity (admin only)
///
/// DELETE /api/v3/time_entries/activities/:id
pub async fn delete_activity(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete activities."));
    }

    let pool = state.pool()?;
    let repo = ActivityRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("TimeEntryActivity", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFilters {
    pub project_id: Option<i64>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivityRequest {
    pub name: String,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
    pub project_id: Option<i64>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateActivityRequest {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub active: Option<bool>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<ActivityResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    position: i32,
    is_default: bool,
    #[serde(rename = "_links")]
    links: ActivityLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityLinks {
    #[serde(rename = "self")]
    self_link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<Link>,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl ActivityResponse {
    fn from_row(row: op_db::ActivityRow) -> Self {
        let id = row.id;
        let project_link = row.project_id.map(|pid| Link {
            href: format!("/api/v3/projects/{}", pid),
        });

        ActivityResponse {
            type_name: "TimeEntryActivity".into(),
            id,
            name: row.name,
            position: row.position,
            is_default: row.is_default,
            links: ActivityLinks {
                self_link: Link {
                    href: format!("/api/v3/time_entries/activities/{}", id),
                },
                project: project_link,
            },
        }
    }
}
