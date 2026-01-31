//! Users API handlers
//!
//! Mirrors: lib/api/v3/users/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extractors::{AppState, AuthenticatedUser, Pagination};

/// List users
///
/// GET /api/v3/users
pub async fn list_users(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let is_admin = user.0.is_admin();

    let response = serde_json::json!({
        "_type": "Collection",
        "total": if is_admin { 10 } else { 1 },
        "count": if is_admin { 10 } else { 1 },
        "pageSize": pagination.page_size,
        "offset": pagination.offset,
        "_embedded": {
            "elements": []
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get a single user
///
/// GET /api/v3/users/:id
pub async fn get_user(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let is_self = user.0.id() == id;
    let is_admin = user.0.is_admin();

    if !is_self && !is_admin {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "_type": "Error",
                "errorIdentifier": "urn:openproject-org:api:v3:errors:MissingPermission",
                "message": "You are not authorized to access this resource."
            })),
        ));
    }

    let response = serde_json::json!({
        "_type": "User",
        "id": id,
        "login": "user",
        "firstName": "Test",
        "lastName": "User",
        "email": "user@example.com",
        "admin": is_admin,
        "status": "active",
        "_links": {
            "self": { "href": format!("/api/v3/users/{}", id) }
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get current user (me)
///
/// GET /api/v3/users/me
pub async fn get_me(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    let id = user.0.id();

    let response = serde_json::json!({
        "_type": "User",
        "id": id,
        "login": user.0.login(),
        "firstName": "Current",
        "lastName": "User",
        "email": user.0.email(),
        "admin": user.0.is_admin(),
        "status": "active",
        "_links": {
            "self": { "href": format!("/api/v3/users/{}", id) }
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Create a new user (admin only)
///
/// POST /api/v3/users
pub async fn create_user(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Json(body): Json<CreateUserRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "_type": "Error",
                "errorIdentifier": "urn:openproject-org:api:v3:errors:MissingPermission",
                "message": "Only administrators can create users."
            })),
        ));
    }

    let response = serde_json::json!({
        "_type": "User",
        "id": 999,
        "login": body.login,
        "firstName": body.firstname,
        "lastName": body.lastname,
        "email": body.email,
        "admin": body.admin.unwrap_or(false),
        "status": "active",
        "_links": {
            "self": { "href": "/api/v3/users/999" }
        }
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update a user
///
/// PATCH /api/v3/users/:id
pub async fn update_user(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(body): Json<UpdateUserRequest>,
) -> ApiResult<impl IntoResponse> {
    let is_self = user.0.id() == id;
    let is_admin = user.0.is_admin();

    if !is_self && !is_admin {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "_type": "Error",
                "errorIdentifier": "urn:openproject-org:api:v3:errors:MissingPermission",
                "message": "You are not authorized to update this user."
            })),
        ));
    }

    let response = serde_json::json!({
        "_type": "User",
        "id": id,
        "login": "user",
        "firstName": body.firstname.unwrap_or_else(|| "Updated".to_string()),
        "lastName": body.lastname.unwrap_or_else(|| "User".to_string()),
        "email": body.email.unwrap_or_else(|| "user@example.com".to_string()),
        "admin": false,
        "status": "active",
        "_links": {
            "self": { "href": format!("/api/v3/users/{}", id) }
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Delete a user (admin only)
///
/// DELETE /api/v3/users/:id
pub async fn delete_user(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Ok((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "_type": "Error",
                "errorIdentifier": "urn:openproject-org:api:v3:errors:MissingPermission",
                "message": "Only administrators can delete users."
            })),
        ));
    }

    if user.0.id() == id {
        return Ok((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "_type": "Error",
                "errorIdentifier": "urn:openproject-org:api:v3:errors:Conflict",
                "message": "You cannot delete your own account."
            })),
        ));
    }

    Ok((StatusCode::NO_CONTENT, Json(serde_json::json!({}))))
}

// Request types

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: Option<String>,
    pub admin: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub admin: Option<bool>,
    pub status: Option<String>,
}
