//! Work Package API handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// GET /api/v3/work_packages
pub async fn list_work_packages(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    // Mock response
    let collection = WorkPackageCollection {
        type_name: "Collection".into(),
        total: 0,
        count: 0,
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements: vec![],
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/work_packages/:id
pub async fn get_work_package(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    // Mock - would fetch from database
    Ok(HalResponse(WorkPackageResponse {
        type_name: "WorkPackage".into(),
        id,
        subject: "Sample Work Package".into(),
        lock_version: 1,
    }))
}

/// POST /api/v3/work_packages
pub async fn create_work_package(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(dto): Json<CreateWorkPackageDto>,
) -> ApiResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        HalResponse(WorkPackageResponse {
            type_name: "WorkPackage".into(),
            id: 1,
            subject: dto.subject,
            lock_version: 0,
        }),
    ))
}

/// PATCH /api/v3/work_packages/:id
pub async fn update_work_package(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateWorkPackageDto>,
) -> ApiResult<impl IntoResponse> {
    // Mock - would fetch and update in database
    Ok(HalResponse(WorkPackageResponse {
        type_name: "WorkPackage".into(),
        id,
        subject: dto.subject.unwrap_or_else(|| "Updated Work Package".into()),
        lock_version: dto.lock_version + 1,
    }))
}

/// DELETE /api/v3/work_packages/:id
pub async fn delete_work_package(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
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
    lock_version: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkPackageDto {
    pub subject: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkPackageDto {
    pub subject: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub lock_version: i32,
}
