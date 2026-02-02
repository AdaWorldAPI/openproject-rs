//! Work Package API handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{Repository, WorkPackageRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// GET /api/v3/work_packages
pub async fn list_work_packages(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WorkPackageRepository::new(pool.clone());

    let rows = repo
        .find_all(pagination.page_size as i64, pagination.offset as i64)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let total = repo
        .count()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<WorkPackageResponse> = rows
        .into_iter()
        .map(|row| WorkPackageResponse {
            type_name: "WorkPackage".into(),
            id: row.id,
            subject: row.subject,
            description: row.description,
            project_id: row.project_id,
            type_id: row.type_id,
            status_id: row.status_id,
            priority_id: row.priority_id,
            author_id: row.author_id,
            assigned_to_id: row.assigned_to_id,
            start_date: row.start_date.map(|d| d.to_string()),
            due_date: row.due_date.map(|d| d.to_string()),
            estimated_hours: row.estimated_hours,
            done_ratio: row.done_ratio,
            lock_version: row.lock_version,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        })
        .collect();

    let collection = WorkPackageCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/work_packages/:id
pub async fn get_work_package(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WorkPackageRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("WorkPackage", id))?;

    Ok(HalResponse(WorkPackageResponse {
        type_name: "WorkPackage".into(),
        id: row.id,
        subject: row.subject,
        description: row.description,
        project_id: row.project_id,
        type_id: row.type_id,
        status_id: row.status_id,
        priority_id: row.priority_id,
        author_id: row.author_id,
        assigned_to_id: row.assigned_to_id,
        start_date: row.start_date.map(|d| d.to_string()),
        due_date: row.due_date.map(|d| d.to_string()),
        estimated_hours: row.estimated_hours,
        done_ratio: row.done_ratio,
        lock_version: row.lock_version,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

/// POST /api/v3/work_packages
pub async fn create_work_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateWorkPackageDto>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WorkPackageRepository::new(pool.clone());

    let create_dto = op_db::CreateWorkPackageDto {
        subject: dto.subject,
        description: dto.description,
        project_id: dto.project_id.unwrap_or(1),
        type_id: dto.type_id.unwrap_or(1),
        status_id: dto.status_id.unwrap_or(1),
        priority_id: dto.priority_id,
        author_id: user.id(),
        assigned_to_id: dto.assigned_to_id,
        responsible_id: None,
        start_date: None,
        due_date: None,
        estimated_hours: dto.estimated_hours,
        done_ratio: 0,
        parent_id: dto.parent_id,
        version_id: None,
        category_id: None,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        HalResponse(WorkPackageResponse {
            type_name: "WorkPackage".into(),
            id: row.id,
            subject: row.subject,
            description: row.description,
            project_id: row.project_id,
            type_id: row.type_id,
            status_id: row.status_id,
            priority_id: row.priority_id,
            author_id: row.author_id,
            assigned_to_id: row.assigned_to_id,
            start_date: row.start_date.map(|d| d.to_string()),
            due_date: row.due_date.map(|d| d.to_string()),
            estimated_hours: row.estimated_hours,
            done_ratio: row.done_ratio,
            lock_version: row.lock_version,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }),
    ))
}

/// PATCH /api/v3/work_packages/:id
pub async fn update_work_package(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateWorkPackageDto>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WorkPackageRepository::new(pool.clone());

    let update_dto = op_db::UpdateWorkPackageDto {
        subject: dto.subject,
        description: dto.description,
        type_id: dto.type_id,
        status_id: dto.status_id,
        priority_id: dto.priority_id,
        assigned_to_id: dto.assigned_to_id,
        responsible_id: None,
        start_date: None,
        due_date: None,
        estimated_hours: dto.estimated_hours,
        done_ratio: dto.done_ratio,
        parent_id: None,
        version_id: None,
        category_id: None,
        lock_version: dto.lock_version,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(&msg),
            op_db::RepositoryError::NotFound(msg) => ApiError::not_found("WorkPackage", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(WorkPackageResponse {
        type_name: "WorkPackage".into(),
        id: row.id,
        subject: row.subject,
        description: row.description,
        project_id: row.project_id,
        type_id: row.type_id,
        status_id: row.status_id,
        priority_id: row.priority_id,
        author_id: row.author_id,
        assigned_to_id: row.assigned_to_id,
        start_date: row.start_date.map(|d| d.to_string()),
        due_date: row.due_date.map(|d| d.to_string()),
        estimated_hours: row.estimated_hours,
        done_ratio: row.done_ratio,
        lock_version: row.lock_version,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }))
}

/// DELETE /api/v3/work_packages/:id
pub async fn delete_work_package(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WorkPackageRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("WorkPackage", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// DTOs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkPackageCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<WorkPackageResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkPackageResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    project_id: Id,
    type_id: Id,
    status_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority_id: Option<Id>,
    author_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned_to_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_hours: Option<f64>,
    done_ratio: i32,
    lock_version: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkPackageDto {
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    pub project_id: Option<Id>,
    pub type_id: Option<Id>,
    pub status_id: Option<Id>,
    pub priority_id: Option<Id>,
    pub assigned_to_id: Option<Id>,
    pub parent_id: Option<Id>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkPackageDto {
    pub subject: Option<String>,
    pub description: Option<String>,
    pub type_id: Option<Id>,
    pub status_id: Option<Id>,
    pub priority_id: Option<Id>,
    pub assigned_to_id: Option<Id>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: Option<i32>,
    #[serde(default)]
    pub lock_version: i32,
}
