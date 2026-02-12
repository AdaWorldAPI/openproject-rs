//! Relations API handlers
//!
//! Mirrors: lib/api/v3/relations/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{relation_type, RelationRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all relations
///
/// GET /api/v3/relations
pub async fn list_relations(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<RelationFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RelationRepository::new(pool.clone());

    let (rows, total) = if let Some(work_package_id) = filters.work_package_id {
        let result = repo
            .find_by_work_package(
                work_package_id,
                op_db::Pagination {
                    limit: pagination.page_size as i64,
                    offset: pagination.offset as i64,
                },
            )
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (result.items, result.total)
    } else if let Some(relation_type) = &filters.relation_type {
        let result = repo
            .find_by_type(
                relation_type,
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

    let elements: Vec<RelationResponse> = rows
        .into_iter()
        .map(|row| RelationResponse::from_row(row))
        .collect();

    let collection = RelationCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single relation
///
/// GET /api/v3/relations/:id
pub async fn get_relation(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RelationRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Relation", id))?;

    Ok(HalResponse(RelationResponse::from_row(row)))
}

/// Create a new relation
///
/// POST /api/v3/relations
pub async fn create_relation(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(dto): Json<CreateRelationRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RelationRepository::new(pool.clone());

    let create_dto = op_db::CreateRelationDto {
        from_id: dto.from_id,
        to_id: dto.to_id,
        relation_type: dto.relation_type,
        lag: dto.lag,
        description: dto.description,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok((StatusCode::CREATED, HalResponse(RelationResponse::from_row(row))))
}

/// Update a relation
///
/// PATCH /api/v3/relations/:id
pub async fn update_relation(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateRelationRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RelationRepository::new(pool.clone());

    let update_dto = op_db::UpdateRelationDto {
        lag: dto.lag,
        description: dto.description,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Relation", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(RelationResponse::from_row(row)))
}

/// Delete a relation
///
/// DELETE /api/v3/relations/:id
pub async fn delete_relation(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RelationRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Relation", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// List relations for a specific work package
///
/// GET /api/v3/work_packages/:work_package_id/relations
pub async fn list_work_package_relations(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = RelationRepository::new(pool.clone());

    let result = repo
        .find_by_work_package(
            work_package_id,
            op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<RelationResponse> = result
        .items
        .into_iter()
        .map(|row| RelationResponse::from_row(row))
        .collect();

    let collection = RelationCollection {
        type_name: "Collection".into(),
        total: result.total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

// Query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationFilters {
    pub work_package_id: Option<i64>,
    pub relation_type: Option<String>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRelationRequest {
    pub from_id: i64,
    pub to_id: i64,
    pub relation_type: String,
    pub lag: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRelationRequest {
    pub lag: Option<i32>,
    pub description: Option<Option<String>>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelationCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<RelationResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelationResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    relation_type: String,
    reverse_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lag: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: RelationLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelationLinks {
    #[serde(rename = "self")]
    self_link: Link,
    from: Link,
    to: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl RelationResponse {
    fn from_row(row: op_db::RelationRow) -> Self {
        let id = row.id;
        let from_id = row.from_id;
        let to_id = row.to_id;
        let reverse_type = relation_type::symmetric_type(&row.relation_type).to_string();
        let name = format_relation_name(&row.relation_type);

        RelationResponse {
            type_name: "Relation".into(),
            id,
            name,
            relation_type: row.relation_type,
            reverse_type,
            lag: row.lag,
            description: row.description,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            links: RelationLinks {
                self_link: Link {
                    href: format!("/api/v3/relations/{}", id),
                },
                from: Link {
                    href: format!("/api/v3/work_packages/{}", from_id),
                },
                to: Link {
                    href: format!("/api/v3/work_packages/{}", to_id),
                },
            },
        }
    }
}

fn format_relation_name(relation_type: &str) -> String {
    match relation_type {
        "relates" => "Relates to".to_string(),
        "precedes" => "Precedes".to_string(),
        "follows" => "Follows".to_string(),
        "blocks" => "Blocks".to_string(),
        "blocked" => "Blocked by".to_string(),
        "duplicates" => "Duplicates".to_string(),
        "duplicated" => "Duplicated by".to_string(),
        "includes" => "Includes".to_string(),
        "partof" => "Part of".to_string(),
        "requires" => "Requires".to_string(),
        "required" => "Required by".to_string(),
        _ => relation_type.to_string(),
    }
}
