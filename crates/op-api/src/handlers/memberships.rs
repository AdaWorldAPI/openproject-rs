//! Memberships API handlers
//!
//! Mirrors: lib/api/v3/memberships/*

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{MemberRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all memberships
///
/// GET /api/v3/memberships
pub async fn list_memberships(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<MembershipFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = MemberRepository::new(pool.clone());

    let (members, total) = if let Some(project_id) = filters.project_id {
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
    } else if let Some(user_id) = filters.principal_id {
        let members = repo
            .find_by_user(user_id)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        let total = members.len() as i64;
        (members, total)
    } else {
        let result = repo
            .find_all_with_roles(op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            })
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (result.items, result.total)
    };

    let elements: Vec<MembershipResponse> = members
        .into_iter()
        .map(|m| MembershipResponse::from_member_with_roles(m))
        .collect();

    let collection = MembershipCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single membership
///
/// GET /api/v3/memberships/:id
pub async fn get_membership(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = MemberRepository::new(pool.clone());

    let member_with_roles = repo
        .find_by_id_with_roles(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Membership", id))?;

    Ok(HalResponse(MembershipResponse::from_member_with_roles(member_with_roles)))
}

/// Create a new membership
///
/// POST /api/v3/memberships
pub async fn create_membership(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateMembershipRequest>,
) -> ApiResult<impl IntoResponse> {
    // Only admins can create memberships
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create memberships."));
    }

    let pool = state.pool()?;
    let repo = MemberRepository::new(pool.clone());

    let create_dto = op_db::CreateMemberDto {
        user_id: dto.principal_id,
        project_id: dto.project_id,
        role_ids: dto.role_ids,
        entity_type: None,
        entity_id: None,
    };

    let member = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    // Get the member with roles
    let member_with_roles = repo
        .find_by_id_with_roles(member.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::internal("Failed to retrieve created membership".to_string()))?;

    Ok((StatusCode::CREATED, HalResponse(MembershipResponse::from_member_with_roles(member_with_roles))))
}

/// Update a membership
///
/// PATCH /api/v3/memberships/:id
pub async fn update_membership(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateMembershipRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can update memberships."));
    }

    let pool = state.pool()?;
    let repo = MemberRepository::new(pool.clone());

    let update_dto = op_db::UpdateMemberDto {
        role_ids: dto.role_ids,
    };

    let member = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Membership", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    // Get the member with roles
    let member_with_roles = repo
        .find_by_id_with_roles(member.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::internal("Failed to retrieve updated membership".to_string()))?;

    Ok(HalResponse(MembershipResponse::from_member_with_roles(member_with_roles)))
}

/// Delete a membership
///
/// DELETE /api/v3/memberships/:id
pub async fn delete_membership(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete memberships."));
    }

    let pool = state.pool()?;
    let repo = MemberRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Membership", id),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipFilters {
    pub project_id: Option<i64>,
    pub principal_id: Option<i64>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMembershipRequest {
    pub principal_id: i64,
    pub project_id: Option<i64>,
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMembershipRequest {
    pub role_ids: Option<Vec<i64>>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MembershipCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<MembershipResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MembershipResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: MembershipLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MembershipLinks {
    #[serde(rename = "self")]
    self_link: Link,
    principal: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<Link>,
    roles: Vec<Link>,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl MembershipResponse {
    fn from_member_with_roles(member_with_roles: op_db::MemberWithRoles) -> Self {
        let member = member_with_roles.member;
        let role_ids = member_with_roles.role_ids;
        let id = member.id;
        let user_id = member.user_id;

        let project_link = member.project_id.map(|pid| Link {
            href: format!("/api/v3/projects/{}", pid),
        });

        let role_links: Vec<Link> = role_ids
            .iter()
            .map(|rid| Link {
                href: format!("/api/v3/roles/{}", rid),
            })
            .collect();

        MembershipResponse {
            type_name: "Membership".into(),
            id,
            created_at: member.created_at.to_rfc3339(),
            updated_at: member.updated_at.to_rfc3339(),
            links: MembershipLinks {
                self_link: Link {
                    href: format!("/api/v3/memberships/{}", id),
                },
                principal: Link {
                    href: format!("/api/v3/users/{}", user_id),
                },
                project: project_link,
                roles: role_links,
            },
        }
    }
}
