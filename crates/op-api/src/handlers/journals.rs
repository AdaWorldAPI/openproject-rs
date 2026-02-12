//! Journals API handlers
//!
//! Mirrors: lib/api/v3/activities/* (journals are exposed as "activities" in API v3)

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{JournalRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List all activities/journals
///
/// GET /api/v3/activities
pub async fn list_activities(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<ActivityFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = JournalRepository::new(pool.clone());

    let (journals, total) = if let Some(work_package_id) = filters.work_package_id {
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
    } else if let Some(user_id) = filters.user_id {
        let result = repo
            .find_by_user(
                user_id,
                op_db::Pagination {
                    limit: pagination.page_size as i64,
                    offset: pagination.offset as i64,
                },
            )
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (result.items, result.total)
    } else {
        let journals = repo
            .find_all(pagination.page_size as i64, pagination.offset as i64)
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        let total = repo
            .count()
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (journals, total)
    };

    let elements: Vec<ActivityResponse> = journals
        .into_iter()
        .map(|j| ActivityResponse::from_journal(j))
        .collect();

    let collection = ActivityCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Get a single activity/journal
///
/// GET /api/v3/activities/:id
pub async fn get_activity(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = JournalRepository::new(pool.clone());

    let journal = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Activity", id))?;

    Ok(HalResponse(ActivityResponse::from_journal(journal)))
}

/// Update an activity/journal (update notes)
///
/// PATCH /api/v3/activities/:id
pub async fn update_activity(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateActivityRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = JournalRepository::new(pool.clone());

    // Check if user is the author or admin
    let existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Activity", id))?;

    if existing.user_id != user.0.id && !user.0.is_admin() {
        return Err(ApiError::forbidden(
            "You can only update your own activities.",
        ));
    }

    let update_dto = op_db::UpdateJournalDto {
        notes: dto.comment.map(Some),
    };

    let journal = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Activity", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(ActivityResponse::from_journal(journal)))
}

/// List activities for a work package
///
/// GET /api/v3/work_packages/:work_package_id/activities
pub async fn list_work_package_activities(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = JournalRepository::new(pool.clone());

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

    let elements: Vec<ActivityResponse> = result
        .items
        .into_iter()
        .map(|j| ActivityResponse::from_journal(j))
        .collect();

    let collection = ActivityCollection {
        type_name: "Collection".into(),
        total: result.total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// List revisions (changing journals) for a work package
///
/// GET /api/v3/work_packages/:work_package_id/revisions
pub async fn list_work_package_revisions(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = JournalRepository::new(pool.clone());

    let result = repo
        .find_changing(
            op_db::journable_type::WORK_PACKAGE,
            work_package_id,
            op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<ActivityResponse> = result
        .items
        .into_iter()
        .map(|j| ActivityResponse::from_journal(j))
        .collect();

    let collection = ActivityCollection {
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
pub struct ActivityFilters {
    pub work_package_id: Option<i64>,
    pub user_id: Option<i64>,
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateActivityRequest {
    pub comment: Option<String>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<ActivityResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<CommentResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<DetailResponse>>,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: ActivityLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommentResponse {
    format: String,
    raw: String,
    html: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DetailResponse {
    format: String,
    raw: String,
    html: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityLinks {
    #[serde(rename = "self")]
    self_link: Link,
    work_package: Link,
    user: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl ActivityResponse {
    fn from_journal(journal: op_db::JournalRow) -> Self {
        let id = journal.id;
        let user_id = journal.user_id;
        let journable_id = journal.journable_id;

        let comment = journal.notes.as_ref().filter(|n| !n.is_empty()).map(|n| CommentResponse {
            format: "markdown".into(),
            raw: n.clone(),
            html: format!("<p>{}</p>", n), // Simplified HTML rendering
        });

        ActivityResponse {
            type_name: "Activity".into(),
            id,
            version: journal.version,
            comment,
            details: None, // Would need to compute diff from journal data
            created_at: journal.created_at.to_rfc3339(),
            updated_at: journal.updated_at.to_rfc3339(),
            links: ActivityLinks {
                self_link: Link {
                    href: format!("/api/v3/activities/{}", id),
                },
                work_package: Link {
                    href: format!("/api/v3/work_packages/{}", journable_id),
                },
                user: Link {
                    href: format!("/api/v3/users/{}", user_id),
                },
            },
        }
    }
}
