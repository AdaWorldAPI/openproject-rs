//! Categories API handlers
//!
//! Mirrors: lib/api/v3/categories/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{CategoryRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all categories
///
/// GET /api/v3/categories
pub async fn list_categories(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<CategoryFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = CategoryRepository::new(pool.clone());

    let (rows, total) = if let Some(project_id) = filters.project_id {
        let result = repo
            .find_by_project(
                project_id,
                op_db::Pagination {
                    limit: pagination.page_size as i64,
                    offset: pagination.offset as i64,
                },
            )
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

    let elements: Vec<CategoryResponse> = rows
        .into_iter()
        .map(|row| CategoryResponse::from_row(row))
        .collect();

    let collection = CategoryCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single category
///
/// GET /api/v3/categories/:id
pub async fn get_category(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = CategoryRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Category", id))?;

    Ok(HalResponse(CategoryResponse::from_row(row)))
}

/// List categories for a specific project
///
/// GET /api/v3/projects/:project_id/categories
pub async fn list_project_categories(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(project_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = CategoryRepository::new(pool.clone());

    let result = repo
        .find_by_project(
            project_id,
            op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<CategoryResponse> = result
        .items
        .into_iter()
        .map(|row| CategoryResponse::from_row(row))
        .collect();

    let collection = CategoryCollection {
        type_name: "Collection".into(),
        total: result.total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Create a new category
///
/// POST /api/v3/categories
pub async fn create_category(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateCategoryRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create categories."));
    }

    let pool = state.pool()?;
    let repo = CategoryRepository::new(pool.clone());

    let create_dto = op_db::CreateCategoryDto {
        project_id: dto.project_id,
        name: dto.name,
        assigned_to_id: dto.assigned_to_id,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(CategoryResponse::from_row(row))))
}

/// Update a category
///
/// PATCH /api/v3/categories/:id
pub async fn update_category(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateCategoryRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update categories."));
    }

    let pool = state.pool()?;
    let repo = CategoryRepository::new(pool.clone());

    let update_dto = op_db::UpdateCategoryDto {
        name: dto.name,
        assigned_to_id: dto.assigned_to_id,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Category", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(CategoryResponse::from_row(row)))
}

/// Delete a category
///
/// DELETE /api/v3/categories/:id
pub async fn delete_category(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete categories."));
    }

    let pool = state.pool()?;
    let repo = CategoryRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Category", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryFilters {
    pub project_id: Option<i64>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub project_id: i64,
    pub name: String,
    pub assigned_to_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub assigned_to_id: Option<i64>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<CategoryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    #[serde(rename = "_links")]
    links: CategoryLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryLinks {
    #[serde(rename = "self")]
    self_link: Link,
    project: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_assignee: Option<Link>,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl CategoryResponse {
    fn from_row(row: op_db::CategoryRow) -> Self {
        let id = row.id;
        let project_id = row.project_id;
        let default_assignee_link = row.assigned_to_id.map(|uid| Link {
            href: format!("/api/v3/users/{}", uid),
        });

        CategoryResponse {
            type_name: "Category".into(),
            id,
            name: row.name,
            links: CategoryLinks {
                self_link: Link {
                    href: format!("/api/v3/categories/{}", id),
                },
                project: Link {
                    href: format!("/api/v3/projects/{}", project_id),
                },
                default_assignee: default_assignee_link,
            },
        }
    }
}
