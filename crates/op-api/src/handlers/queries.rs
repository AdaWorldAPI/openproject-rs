//! Query API handlers
//!
//! Mirrors: app/controllers/api/v3/queries_controller.rb

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{QueryRepository, Repository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// GET /api/v3/queries
pub async fn list_queries(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    pagination: Pagination,
    Query(filters): Query<QueryFilters>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    let result = repo
        .find_visible(
            user.0.id,
            filters.project_id,
            op_db::Pagination {
                limit: pagination.page_size as i64,
                offset: pagination.offset as i64,
            },
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<QueryResponse> = result
        .items
        .into_iter()
        .map(|qws| QueryResponse::from_query_with_starred(qws))
        .collect();

    let collection = QueryCollection {
        type_name: "Collection".into(),
        total: result.total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };
    Ok(HalResponse(collection))
}

/// GET /api/v3/queries/default
pub async fn get_default_query(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    // Return a default query configuration
    Ok(HalResponse(QueryResponse {
        type_name: "Query".into(),
        id: 0, // Special ID for default query
        name: "Default".into(),
        public: false,
        starred: false,
        sums: false,
        show_hierarchies: true,
        timeline_visible: false,
        include_subprojects: true,
        timestamps: None,
        links: QueryLinks::default_links(),
    }))
}

/// GET /api/v3/queries/:id
pub async fn get_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    let qws = repo
        .find_by_id_with_starred(id, user.0.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Query", id))?;

    Ok(HalResponse(QueryResponse::from_query_with_starred(qws)))
}

/// POST /api/v3/queries
pub async fn create_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateQueryRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    let create_dto = op_db::CreateQueryDto {
        project_id: dto.project_id,
        user_id: user.0.id,
        name: dto.name,
        filters: dto.filters,
        column_names: dto.column_names,
        sort_criteria: dto.sort_criteria,
        group_by: dto.group_by,
        display_sums: dto.sums,
        show_hierarchies: dto.show_hierarchies,
        include_subprojects: dto.include_subprojects,
        timeline_visible: dto.timeline_visible,
        timestamps: dto.timestamps,
    };

    let query = repo
        .create(create_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    let qws = op_db::QueryWithStarred {
        query,
        starred: false,
    };

    Ok((StatusCode::CREATED, HalResponse(QueryResponse::from_query_with_starred(qws))))
}

/// PATCH /api/v3/queries/:id
pub async fn update_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateQueryRequest>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    // Check ownership or admin
    let existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Query", id))?;

    if existing.user_id != user.0.id && !user.0.is_admin() {
        return Err(ApiError::forbidden("You can only update your own queries."));
    }

    let update_dto = op_db::UpdateQueryDto {
        name: dto.name,
        filters: dto.filters,
        column_names: dto.column_names,
        sort_criteria: dto.sort_criteria,
        group_by: dto.group_by,
        display_sums: dto.sums,
        show_hierarchies: dto.show_hierarchies,
        include_subprojects: dto.include_subprojects,
        timeline_visible: dto.timeline_visible,
        timestamps: dto.timestamps,
    };

    let query = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Query", id),
            op_db::RepositoryError::Validation(msg) => ApiError::bad_request(msg),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    let starred = repo
        .is_starred(id, user.0.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let qws = op_db::QueryWithStarred { query, starred };

    Ok(HalResponse(QueryResponse::from_query_with_starred(qws)))
}

/// DELETE /api/v3/queries/:id
pub async fn delete_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    // Check ownership or admin
    let existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Query", id))?;

    if existing.user_id != user.0.id && !user.0.is_admin() {
        return Err(ApiError::forbidden("You can only delete your own queries."));
    }

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Query", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v3/queries/:id/star
pub async fn star_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    repo.star(id, user.0.id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("Query", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    let qws = repo
        .find_by_id_with_starred(id, user.0.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Query", id))?;

    Ok(HalResponse(QueryResponse::from_query_with_starred(qws)))
}

/// DELETE /api/v3/queries/:id/star
pub async fn unstar_query(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = QueryRepository::new(pool.clone());

    repo.unstar(id, user.0.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let qws = repo
        .find_by_id_with_starred(id, user.0.id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Query", id))?;

    Ok(HalResponse(QueryResponse::from_query_with_starred(qws)))
}

/// GET /api/v3/queries/available_projects
pub async fn available_projects(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;

    // Get projects the user has access to
    let projects = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT DISTINCT p.id, p.name
        FROM projects p
        JOIN members m ON m.project_id = p.id
        WHERE m.user_id = $1 AND p.active = true
        ORDER BY p.name ASC
        "#,
    )
    .bind(user.0.id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<ProjectStub> = projects
        .into_iter()
        .map(|(id, name)| ProjectStub { id, name })
        .collect();

    Ok(HalResponse(AvailableProjectsResponse {
        type_name: "Collection".into(),
        total: elements.len(),
        elements,
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

// Query parameters
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilters {
    pub project_id: Option<i64>,
}

// Request DTOs
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQueryRequest {
    pub name: String,
    pub project_id: Option<i64>,
    pub filters: Option<String>,
    pub column_names: Option<String>,
    pub sort_criteria: Option<String>,
    pub group_by: Option<String>,
    #[serde(default)]
    pub sums: Option<bool>,
    pub show_hierarchies: Option<bool>,
    pub include_subprojects: Option<bool>,
    #[serde(default)]
    pub timeline_visible: Option<bool>,
    pub timestamps: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQueryRequest {
    pub name: Option<String>,
    pub filters: Option<Option<String>>,
    pub column_names: Option<Option<String>>,
    pub sort_criteria: Option<Option<String>>,
    pub group_by: Option<Option<String>>,
    pub sums: Option<bool>,
    pub show_hierarchies: Option<bool>,
    pub include_subprojects: Option<bool>,
    pub timeline_visible: Option<bool>,
    pub timestamps: Option<Option<String>>,
}

// Response DTOs
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
    show_hierarchies: bool,
    timeline_visible: bool,
    include_subprojects: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamps: Option<String>,
    #[serde(rename = "_links")]
    links: QueryLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryLinks {
    #[serde(rename = "self")]
    self_link: Link,
    user: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<Link>,
    star: Link,
    unstar: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl QueryLinks {
    fn default_links() -> Self {
        QueryLinks {
            self_link: Link {
                href: "/api/v3/queries/default".to_string(),
            },
            user: Link {
                href: "/api/v3/users/me".to_string(),
            },
            project: None,
            star: Link {
                href: "/api/v3/queries/default/star".to_string(),
            },
            unstar: Link {
                href: "/api/v3/queries/default/star".to_string(),
            },
        }
    }
}

impl QueryResponse {
    fn from_query_with_starred(qws: op_db::QueryWithStarred) -> Self {
        let query = qws.query;
        let id = query.id;
        let user_id = query.user_id;
        let project_id = query.project_id;

        let project_link = project_id.map(|pid| Link {
            href: format!("/api/v3/projects/{}", pid),
        });

        QueryResponse {
            type_name: "Query".into(),
            id,
            name: query.name,
            public: false, // Determined by views table, simplified for now
            starred: qws.starred,
            sums: query.display_sums,
            show_hierarchies: query.show_hierarchies,
            timeline_visible: query.timeline_visible,
            include_subprojects: query.include_subprojects,
            timestamps: query.timestamps,
            links: QueryLinks {
                self_link: Link {
                    href: format!("/api/v3/queries/{}", id),
                },
                user: Link {
                    href: format!("/api/v3/users/{}", user_id),
                },
                project: project_link,
                star: Link {
                    href: format!("/api/v3/queries/{}/star", id),
                },
                unstar: Link {
                    href: format!("/api/v3/queries/{}/star", id),
                },
            },
        }
    }
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
