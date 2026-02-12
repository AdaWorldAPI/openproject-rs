//! Types API handlers
//!
//! Mirrors: lib/api/v3/types/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{Repository, TypeRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all types
///
/// GET /api/v3/types
pub async fn list_types(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TypeRepository::new(pool.clone());

    let rows = repo
        .find_all(pagination.page_size as i64, pagination.offset as i64)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let total = repo
        .count()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<TypeResponse> = rows
        .into_iter()
        .map(|row| TypeResponse::from_row(row))
        .collect();

    let collection = TypeCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single type
///
/// GET /api/v3/types/:id
pub async fn get_type(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TypeRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Type", id))?;

    Ok(HalResponse(TypeResponse::from_row(row)))
}

/// List types available in a project
///
/// GET /api/v3/projects/:project_id/types
pub async fn list_project_types(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TypeRepository::new(pool.clone());

    let rows = repo
        .find_by_project(project_id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<TypeResponse> = rows
        .into_iter()
        .map(|row| TypeResponse::from_row(row))
        .collect();

    let collection = TypeCollection {
        type_name: "Collection".into(),
        total: elements.len(),
        count: elements.len(),
        page_size: 100,
        offset: 0,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Create a new type (admin only)
///
/// POST /api/v3/types
pub async fn create_type(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateTypeRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create types."));
    }

    let pool = state.pool()?;
    let repo = TypeRepository::new(pool.clone());

    let create_dto = op_db::CreateTypeDto {
        name: dto.name,
        position: dto.position,
        is_default: dto.is_default.unwrap_or(false),
        is_in_roadmap: dto.is_in_roadmap.unwrap_or(false),
        is_milestone: dto.is_milestone.unwrap_or(false),
        color_id: dto.color_id,
        description: dto.description,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(TypeResponse::from_row(row))))
}

/// Update a type (admin only)
///
/// PATCH /api/v3/types/:id
pub async fn update_type(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateTypeRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update types."));
    }

    let pool = state.pool()?;
    let repo = TypeRepository::new(pool.clone());

    let update_dto = op_db::UpdateTypeDto {
        name: dto.name,
        position: dto.position,
        is_default: dto.is_default,
        is_in_roadmap: dto.is_in_roadmap,
        is_milestone: dto.is_milestone,
        color_id: dto.color_id,
        description: dto.description,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Type", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(TypeResponse::from_row(row)))
}

/// Delete a type (admin only)
///
/// DELETE /api/v3/types/:id
pub async fn delete_type(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete types."));
    }

    let pool = state.pool()?;
    let repo = TypeRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Type", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTypeRequest {
    pub name: String,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub is_in_roadmap: Option<bool>,
    pub is_milestone: Option<bool>,
    pub color_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTypeRequest {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub is_default: Option<bool>,
    pub is_in_roadmap: Option<bool>,
    pub is_milestone: Option<bool>,
    pub color_id: Option<i64>,
    pub description: Option<String>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TypeCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<TypeResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TypeResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    position: i32,
    is_default: bool,
    is_milestone: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "_links")]
    links: TypeLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TypeLinks {
    #[serde(rename = "self")]
    self_link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Link>,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl TypeResponse {
    fn from_row(row: op_db::TypeRow) -> Self {
        let color_link = row.color_id.map(|cid| Link {
            href: format!("/api/v3/colors/{}", cid),
        });

        TypeResponse {
            type_name: "Type".into(),
            id: row.id,
            name: row.name,
            position: row.position,
            is_default: row.is_default,
            is_milestone: row.is_milestone,
            description: row.description,
            links: TypeLinks {
                self_link: Link {
                    href: format!("/api/v3/types/{}", row.id),
                },
                color: color_link,
            },
        }
    }
}
