//! Project API handlers
//!
//! Mirrors: lib/api/v3/projects/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{ProjectRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// GET /api/v3/projects
pub async fn list_projects(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<ProjectFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    let (rows, total) = if filters.active_only.unwrap_or(false) {
        let result = repo
            .find_active(op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            })
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

    let elements: Vec<ProjectResponse> = rows
        .into_iter()
        .map(|row| ProjectResponse::from_row(row))
        .collect();

    let collection = ProjectCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/projects/:id
pub async fn get_project(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Project", id))?;

    Ok(HalResponse(ProjectResponse::from_row(row)))
}

/// POST /api/v3/projects
pub async fn create_project(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateProjectDto>,
) -> ApiResult<impl IntoResponse> {
    // Only admins can create projects
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create projects."));
    }

    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    // Check identifier uniqueness
    let is_unique = repo
        .is_identifier_unique(&dto.identifier, None)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if !is_unique {
        return Err(ApiError::conflict("Identifier has already been taken"));
    }

    let create_dto = op_db::CreateProjectDto {
        name: dto.name,
        description: dto.description,
        identifier: dto.identifier,
        public: dto.public.unwrap_or(false),
        parent_id: dto.parent_id,
        active: dto.active.unwrap_or(true),
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok((StatusCode::CREATED, HalResponse(ProjectResponse::from_row(row))))
}

/// PATCH /api/v3/projects/:id
pub async fn update_project(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateProjectDto>,
) -> ApiResult<impl IntoResponse> {
    // Only admins can update projects (simplified permission check)
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update projects."));
    }

    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    // Verify project exists
    if !repo.exists(id).await.map_err(|e| ApiError::internal(format!("Database error: {}", e)))? {
        return Err(ApiError::not_found("Project", id));
    }

    let update_dto = op_db::UpdateProjectDto {
        name: dto.name,
        description: dto.description,
        public: dto.public,
        parent_id: None, // Parent change not allowed via simple update
        active: dto.active,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Project", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(ProjectResponse::from_row(row)))
}

/// DELETE /api/v3/projects/:id
pub async fn delete_project(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    // Only admins can delete projects
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete projects."));
    }

    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Project", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v3/projects/:id/archive
pub async fn archive_project(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can archive projects."));
    }

    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    // Verify project exists
    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Project", id))?;

    repo.archive(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Return updated project
    let updated = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Project", id))?;

    Ok(HalResponse(ProjectResponse::from_row(updated)))
}

/// POST /api/v3/projects/:id/unarchive
pub async fn unarchive_project(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can unarchive projects."));
    }

    let pool = state.pool()?;
    let repo = ProjectRepository::new(pool.clone());

    // Verify project exists
    let _row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Project", id))?;

    repo.unarchive(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Return updated project
    let updated = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Project", id))?;

    Ok(HalResponse(ProjectResponse::from_row(updated)))
}

// Query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFilters {
    pub active_only: Option<bool>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    public: bool,
    active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<Id>,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: ProjectLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLinks {
    #[serde(rename = "self")]
    self_link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<Link>,
    work_packages: Link,
    categories: Link,
    versions: Link,
    memberships: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl ProjectResponse {
    fn from_row(row: op_db::ProjectRow) -> Self {
        let parent_link = row.parent_id.map(|pid| Link {
            href: format!("/api/v3/projects/{}", pid),
        });

        ProjectResponse {
            type_name: "Project".into(),
            id: row.id,
            name: row.name,
            identifier: row.identifier.clone(),
            description: row.description,
            public: row.public,
            active: row.active,
            parent_id: row.parent_id,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            links: ProjectLinks {
                self_link: Link {
                    href: format!("/api/v3/projects/{}", row.id),
                },
                parent: parent_link,
                work_packages: Link {
                    href: format!("/api/v3/projects/{}/work_packages", row.id),
                },
                categories: Link {
                    href: format!("/api/v3/projects/{}/categories", row.id),
                },
                versions: Link {
                    href: format!("/api/v3/projects/{}/versions", row.id),
                },
                memberships: Link {
                    href: format!("/api/v3/memberships?filters=[{{\"project\":{{\"operator\":\"=\",\"values\":[\"{}\"]}}}}]", row.id),
                },
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectDto {
    pub name: String,
    pub identifier: String,
    #[serde(default)]
    pub description: Option<String>,
    pub public: Option<bool>,
    pub active: Option<bool>,
    pub parent_id: Option<Id>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
    pub active: Option<bool>,
}
