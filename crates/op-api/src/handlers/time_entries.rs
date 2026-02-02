//! Time Entry API handlers
//!
//! Mirrors: app/controllers/api/v3/time_entries_controller.rb

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use op_core::traits::Id;
use op_db::{Repository, TimeEntryRepository, Pagination as DbPagination};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// GET /api/v3/time_entries
pub async fn list_time_entries(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<TimeEntryFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TimeEntryRepository::new(pool.clone());

    let (rows, total) = if let Some(work_package_id) = filters.work_package_id {
        let result = repo
            .find_by_work_package(
                work_package_id,
                DbPagination {
                    limit: pagination.page_size as i64,
                    offset: pagination.offset as i64,
                },
            )
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        (result.items, result.total)
    } else if let Some(project_id) = filters.project_id {
        let result = repo
            .find_by_project(
                project_id,
                DbPagination {
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
                DbPagination {
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

    let elements: Vec<TimeEntryResponse> = rows
        .into_iter()
        .map(|row| TimeEntryResponse::from_row(row))
        .collect();

    let collection = TimeEntryCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/time_entries/:id
pub async fn get_time_entry(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TimeEntryRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("TimeEntry", id))?;

    Ok(HalResponse(TimeEntryResponse::from_row(row)))
}

/// POST /api/v3/time_entries
pub async fn create_time_entry(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateTimeEntryDto>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TimeEntryRepository::new(pool.clone());

    // Parse spent_on date
    let spent_on = dto
        .spent_on
        .as_ref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| ApiError::bad_request("Invalid date format for spent_on. Use YYYY-MM-DD"))?
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let create_dto = op_db::CreateTimeEntryDto {
        project_id: dto.project_id,
        user_id: dto.user_id.unwrap_or_else(|| user.id()),
        work_package_id: dto.work_package_id,
        hours: dto.hours,
        comments: dto.comments,
        activity_id: dto.activity_id,
        spent_on,
        logged_by_id: Some(user.id()),
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok((StatusCode::CREATED, HalResponse(TimeEntryResponse::from_row(row))))
}

/// PATCH /api/v3/time_entries/:id
pub async fn update_time_entry(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateTimeEntryDto>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TimeEntryRepository::new(pool.clone());

    // Parse spent_on date if provided
    let spent_on = dto
        .spent_on
        .as_ref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| ApiError::bad_request("Invalid date format for spent_on. Use YYYY-MM-DD"))?;

    let update_dto = op_db::UpdateTimeEntryDto {
        work_package_id: dto.work_package_id,
        hours: dto.hours,
        comments: dto.comments,
        activity_id: dto.activity_id,
        spent_on,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("TimeEntry", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(TimeEntryResponse::from_row(row)))
}

/// DELETE /api/v3/time_entries/:id
pub async fn delete_time_entry(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = TimeEntryRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("TimeEntry", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// Query filters
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntryFilters {
    pub work_package_id: Option<Id>,
    pub project_id: Option<Id>,
    pub user_id: Option<Id>,
}

// DTOs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TimeEntryCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<TimeEntryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TimeEntryResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    project_id: Id,
    user_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    work_package_id: Option<Id>,
    hours: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    comments: Option<String>,
    activity_id: Id,
    spent_on: String,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: TimeEntryLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TimeEntryLinks {
    #[serde(rename = "self")]
    self_link: Link,
    project: Link,
    user: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    work_package: Option<Link>,
    activity: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl TimeEntryResponse {
    fn from_row(row: op_db::TimeEntryRow) -> Self {
        Self {
            type_name: "TimeEntry".into(),
            id: row.id,
            project_id: row.project_id,
            user_id: row.user_id,
            work_package_id: row.work_package_id,
            hours: row.hours,
            comments: row.comments,
            activity_id: row.activity_id,
            spent_on: row.spent_on.to_string(),
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            links: TimeEntryLinks {
                self_link: Link {
                    href: format!("/api/v3/time_entries/{}", row.id),
                },
                project: Link {
                    href: format!("/api/v3/projects/{}", row.project_id),
                },
                user: Link {
                    href: format!("/api/v3/users/{}", row.user_id),
                },
                work_package: row.work_package_id.map(|id| Link {
                    href: format!("/api/v3/work_packages/{}", id),
                }),
                activity: Link {
                    href: format!("/api/v3/time_entries/activities/{}", row.activity_id),
                },
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimeEntryDto {
    pub project_id: Id,
    pub user_id: Option<Id>,
    pub work_package_id: Option<Id>,
    pub hours: f64,
    #[serde(default)]
    pub comments: Option<String>,
    pub activity_id: Id,
    pub spent_on: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTimeEntryDto {
    pub work_package_id: Option<Id>,
    pub hours: Option<f64>,
    pub comments: Option<String>,
    pub activity_id: Option<Id>,
    pub spent_on: Option<String>,
}
