//! Project API handlers

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

/// GET /api/v3/projects
pub async fn list_projects(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let collection = ProjectCollection {
        type_name: "Collection".into(),
        total: 0,
        count: 0,
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements: vec![],
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/projects/:id
pub async fn get_project(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(ProjectResponse {
        type_name: "Project".into(),
        id,
        name: "Sample Project".into(),
        identifier: "sample".into(),
        public: true,
        active: true,
    }))
}

/// POST /api/v3/projects
pub async fn create_project(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(dto): Json<CreateProjectDto>,
) -> ApiResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        HalResponse(ProjectResponse {
            type_name: "Project".into(),
            id: 1,
            name: dto.name,
            identifier: dto.identifier.unwrap_or_else(|| "new-project".into()),
            public: dto.public,
            active: true,
        }),
    ))
}

/// PATCH /api/v3/projects/:id
pub async fn update_project(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateProjectDto>,
) -> ApiResult<impl IntoResponse> {
    // Mock - would fetch and update in database
    Ok(HalResponse(ProjectResponse {
        type_name: "Project".into(),
        id,
        name: dto.name.unwrap_or_else(|| "Updated Project".into()),
        identifier: dto.identifier.unwrap_or_else(|| "updated".into()),
        public: dto.public.unwrap_or(false),
        active: dto.active.unwrap_or(true),
    }))
}

/// DELETE /api/v3/projects/:id
pub async fn delete_project(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    Ok(StatusCode::NO_CONTENT)
}

// DTOs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<ProjectResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    identifier: String,
    public: bool,
    active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectDto {
    pub name: String,
    pub identifier: Option<String>,
    #[serde(default)]
    pub public: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectDto {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub public: Option<bool>,
    pub active: Option<bool>,
    pub description: Option<String>,
}
