//! Query API handlers
//!
//! Mirrors: app/controllers/api/v3/queries_controller.rb

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use serde::{Deserialize, Serialize};

use crate::error::ApiResult;
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// GET /api/v3/queries
pub async fn list_queries(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let collection = QueryCollection {
        type_name: "Collection".into(),
        total: 0,
        count: 0,
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements: vec![],
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/queries/default
pub async fn get_default_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(QueryResponse {
        type_name: "Query".into(),
        id: 0, // Special ID for default query
        name: "Default".into(),
        public: false,
        starred: false,
        sums: false,
        timeline_visible: false,
        timestamps: None,
    }))
}

/// GET /api/v3/queries/:id
pub async fn get_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(QueryResponse {
        type_name: "Query".into(),
        id,
        name: "Saved Query".into(),
        public: false,
        starred: false,
        sums: false,
        timeline_visible: false,
        timestamps: None,
    }))
}

/// POST /api/v3/queries
pub async fn create_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Json(dto): Json<CreateQueryDto>,
) -> ApiResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        HalResponse(QueryResponse {
            type_name: "Query".into(),
            id: 1,
            name: dto.name,
            public: dto.public,
            starred: dto.starred,
            sums: dto.sums,
            timeline_visible: dto.timeline_visible,
            timestamps: None,
        }),
    ))
}

/// PATCH /api/v3/queries/:id
pub async fn update_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateQueryDto>,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(QueryResponse {
        type_name: "Query".into(),
        id,
        name: dto.name.unwrap_or_else(|| "Updated Query".into()),
        public: dto.public.unwrap_or(false),
        starred: dto.starred.unwrap_or(false),
        sums: dto.sums.unwrap_or(false),
        timeline_visible: dto.timeline_visible.unwrap_or(false),
        timestamps: None,
    }))
}

/// DELETE /api/v3/queries/:id
pub async fn delete_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v3/queries/:id/star
pub async fn star_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(QueryResponse {
        type_name: "Query".into(),
        id,
        name: "Starred Query".into(),
        public: false,
        starred: true,
        sums: false,
        timeline_visible: false,
        timestamps: None,
    }))
}

/// DELETE /api/v3/queries/:id/star
pub async fn unstar_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(QueryResponse {
        type_name: "Query".into(),
        id,
        name: "Unstarred Query".into(),
        public: false,
        starred: false,
        sums: false,
        timeline_visible: false,
        timestamps: None,
    }))
}

/// GET /api/v3/queries/available_projects
pub async fn available_projects(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(AvailableProjectsResponse {
        type_name: "Collection".into(),
        total: 0,
        elements: vec![],
    }))
}

/// GET /api/v3/queries/form
pub async fn query_form(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    Ok(HalResponse(QueryFormResponse {
        type_name: "Form".into(),
        payload: QueryFormPayload {
            type_name: "Query".into(),
        },
    }))
}

// DTOs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<QueryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    public: bool,
    starred: bool,
    sums: bool,
    timeline_visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamps: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQueryDto {
    pub name: String,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub sums: bool,
    #[serde(default)]
    pub timeline_visible: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQueryDto {
    pub name: Option<String>,
    pub public: Option<bool>,
    pub starred: Option<bool>,
    pub sums: Option<bool>,
    pub timeline_visible: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AvailableProjectsResponse {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<ProjectStub>,
}

#[derive(Debug, Serialize)]
struct ProjectStub {
    id: Id,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryFormResponse {
    #[serde(rename = "_type")]
    type_name: String,
    #[serde(rename = "_embedded")]
    payload: QueryFormPayload,
}

#[derive(Debug, Serialize)]
struct QueryFormPayload {
    #[serde(rename = "_type")]
    type_name: String,
}
