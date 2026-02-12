//! Roles API handlers
//!
//! Mirrors: lib/api/v3/roles/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{Repository, RoleRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all roles
///
/// GET /api/v3/roles
pub async fn list_roles(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RoleRepository::new(pool.clone());

    let rows = repo
        .find_all(pagination.page_size as i64, pagination.offset as i64)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let total = repo
        .count()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Fetch permissions for each role
    let mut elements = Vec::with_capacity(rows.len());
    for row in rows {
        let permissions = repo
            .get_permissions(row.id)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        elements.push(RoleResponse::from_row(row, permissions));
    }

    let collection = RoleCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single role
///
/// GET /api/v3/roles/:id
pub async fn get_role(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RoleRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Role", id))?;

    let permissions = repo
        .get_permissions(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok(HalResponse(RoleResponse::from_row(row, permissions)))
}

/// Create a new role (admin only)
///
/// POST /api/v3/roles
pub async fn create_role(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateRoleRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create roles."));
    }

    let pool = state.pool()?;
    let repo = RoleRepository::new(pool.clone());

    let create_dto = op_db::CreateRoleDto {
        name: dto.name,
        position: dto.position,
        role_type: "Role".to_string(),
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    // Set permissions if provided
    if let Some(permissions) = dto.permissions {
        repo.set_permissions(row.id, &permissions)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
    }

    let permissions = repo
        .get_permissions(row.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok((StatusCode::CREATED, HalResponse(RoleResponse::from_row(row, permissions))))
}

/// Update a role (admin only)
///
/// PATCH /api/v3/roles/:id
pub async fn update_role(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateRoleRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update roles."));
    }

    let pool = state.pool()?;
    let repo = RoleRepository::new(pool.clone());

    let update_dto = op_db::UpdateRoleDto {
        name: dto.name,
        position: dto.position,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Role", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    // Set permissions if provided
    if let Some(permissions) = dto.permissions {
        repo.set_permissions(id, &permissions)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
    }

    let permissions = repo
        .get_permissions(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok(HalResponse(RoleResponse::from_row(row, permissions)))
}

/// Delete a role (admin only)
///
/// DELETE /api/v3/roles/:id
pub async fn delete_role(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete roles."));
    }

    let pool = state.pool()?;
    let repo = RoleRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Role", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub name: String,
    pub position: Option<i32>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub position: Option<i32>,
    pub permissions: Option<Vec<String>>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoleCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<RoleResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoleResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    position: i32,
    builtin: bool,
    permissions: Vec<String>,
    #[serde(rename = "_links")]
    links: RoleLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoleLinks {
    #[serde(rename = "self")]
    self_link: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl RoleResponse {
    fn from_row(row: op_db::RoleRow, permissions: Vec<String>) -> Self {
        let id = row.id;
        let builtin = row.is_builtin();

        RoleResponse {
            type_name: "Role".into(),
            id,
            name: row.name,
            position: row.position,
            builtin,
            permissions,
            links: RoleLinks {
                self_link: Link {
                    href: format!("/api/v3/roles/{}", id),
                },
            },
        }
    }
}
