//! Watchers API handlers
//!
//! Mirrors: lib/api/v3/work_packages/watchers/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{Repository, WatcherRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List watchers for a work package
///
/// GET /api/v3/work_packages/:work_package_id/watchers
pub async fn list_work_package_watchers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WatcherRepository::new(pool.clone());

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

    let elements: Vec<WatcherResponse> = result
        .items
        .into_iter()
        .map(|w| WatcherResponse::from_watcher_with_user(w, work_package_id))
        .collect();

    let collection = WatcherCollection {
        type_name: "Collection".into(),
        total: result.total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };

    Ok(HalResponse(collection))
}

/// Add a watcher to a work package
///
/// POST /api/v3/work_packages/:work_package_id/watchers
pub async fn add_work_package_watcher(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
    Json(dto): Json<AddWatcherRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WatcherRepository::new(pool.clone());

    let create_dto = op_db::CreateWatcherDto {
        watchable_type: "WorkPackage".to_string(),
        watchable_id: work_package_id,
        user_id: dto.user_id,
    };

    let watcher = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            op_db::RepositoryError::Conflict(msg) => ApiError::conflict(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    // Get full watcher info
    let result = repo
        .find_by_work_package(
            work_package_id,
            op_db::Pagination { limit: 1000, offset: 0 },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let watcher_with_user = result
        .items
        .into_iter()
        .find(|w| w.watcher.id == watcher.id)
        .ok_or_else(|| ApiError::internal("Failed to retrieve created watcher".to_string()))?;

    Ok((
        StatusCode::CREATED,
        HalResponse(WatcherResponse::from_watcher_with_user(watcher_with_user, work_package_id)),
    ))
}

/// Remove a watcher from a work package
///
/// DELETE /api/v3/work_packages/:work_package_id/watchers/:user_id
pub async fn remove_work_package_watcher(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((work_package_id, user_id)): Path<(Id, Id)>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WatcherRepository::new(pool.clone());

    let deleted = repo
        .delete_by_user_and_watchable(user_id, "WorkPackage", work_package_id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if !deleted {
        return Err(ApiError::not_found("Watcher", user_id));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Check if current user is watching a work package
///
/// GET /api/v3/work_packages/:work_package_id/watching
pub async fn is_watching_work_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WatcherRepository::new(pool.clone());

    let is_watching = repo
        .is_watching(user.0.id, "WorkPackage", work_package_id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok(HalResponse(WatchingResponse { watching: is_watching }))
}

/// Watch a work package as the current user
///
/// POST /api/v3/work_packages/:work_package_id/watch
pub async fn watch_work_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WatcherRepository::new(pool.clone());

    let create_dto = op_db::CreateWatcherDto {
        watchable_type: "WorkPackage".to_string(),
        watchable_id: work_package_id,
        user_id: user.0.id,
    };

    match repo.create(create_dto).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(op_db::RepositoryError::Conflict(_)) => {
            // Already watching - treat as success (idempotent)
            Ok(StatusCode::NO_CONTENT)
        }
        Err(op_db::RepositoryError::Validation(msg)) => Err(ApiError::bad_request(msg)),
        Err(e) => Err(ApiError::internal(format!("Database error: {}", e))),
    }
}

/// Unwatch a work package as the current user
///
/// DELETE /api/v3/work_packages/:work_package_id/watch
pub async fn unwatch_work_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(work_package_id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = WatcherRepository::new(pool.clone());

    repo.delete_by_user_and_watchable(user.0.id, "WorkPackage", work_package_id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

// Request types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWatcherRequest {
    pub user_id: i64,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WatcherCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<WatcherResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WatcherResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    name: String,
    #[serde(rename = "_links")]
    links: WatcherLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WatcherLinks {
    #[serde(rename = "self")]
    self_link: Link,
    user: Link,
    watchable: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WatchingResponse {
    watching: bool,
}

impl WatcherResponse {
    fn from_watcher_with_user(w: op_db::WatcherWithUser, work_package_id: i64) -> Self {
        let id = w.watcher.id;
        let user_id = w.watcher.user_id;
        let user_name = w.user_name();

        WatcherResponse {
            type_name: "Watcher".into(),
            id,
            name: user_name.clone(),
            links: WatcherLinks {
                self_link: Link {
                    href: format!("/api/v3/work_packages/{}/watchers/{}", work_package_id, user_id),
                    title: None,
                },
                user: Link {
                    href: format!("/api/v3/users/{}", user_id),
                    title: Some(user_name),
                },
                watchable: Link {
                    href: format!("/api/v3/work_packages/{}", work_package_id),
                    title: None,
                },
            },
        }
    }
}
