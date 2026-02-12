//! Users API handlers
//!
//! Mirrors: lib/api/v3/users/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use op_core::traits::Id;
use op_db::{Repository, UserRepository};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};
use crate::extractors::{AppState, AuthenticatedUser, HalResponse, Pagination};

/// List users
///
/// GET /api/v3/users
pub async fn list_users(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    pagination: Pagination,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    // Only admins can list all users
    if !user.0.is_admin() {
        // Non-admins only see themselves
        let self_user = repo
            .find_by_id(user.id())
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

        let elements: Vec<UserResponse> = self_user
            .into_iter()
            .map(|row| UserResponse::from_row(row, true))
            .collect();

        let collection = UserCollection {
            type_name: "Collection".into(),
            total: elements.len(),
            count: elements.len(),
            page_size: pagination.page_size,
            offset: pagination.offset,
            elements,
        };
        return Ok(HalResponse(collection));
    }

    let rows = repo
        .find_all(pagination.page_size as i64, pagination.offset as i64)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let total = repo
        .count()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let elements: Vec<UserResponse> = rows
        .into_iter()
        .map(|row| UserResponse::from_row(row, true))
        .collect();

    let collection = UserCollection {
        type_name: "Collection".into(),
        total: total as usize,
        count: elements.len(),
        page_size: pagination.page_size,
        offset: pagination.offset,
        elements,
    };
    Ok(HalResponse(collection))
}

/// Get a single user
///
/// GET /api/v3/users/:id
pub async fn get_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    let is_self = user.id() == id;
    let is_admin = user.0.is_admin();

    // Non-admins can only view themselves
    if !is_self && !is_admin {
        return Err(ApiError::forbidden("You are not authorized to access this resource."));
    }

    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    let row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("User", id))?;

    Ok(HalResponse(UserResponse::from_row(row, is_self || is_admin)))
}

/// Get current user (me)
///
/// GET /api/v3/users/me
pub async fn get_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    let row = repo
        .find_by_id(user.id())
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("User", user.id()))?;

    Ok(HalResponse(UserResponse::from_row(row, true)))
}

/// Create a new user (admin only)
///
/// POST /api/v3/users
pub async fn create_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateUserRequest>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can create users."));
    }

    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    // Check login uniqueness
    let login_unique = repo
        .is_login_unique(&dto.login, None)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if !login_unique {
        return Err(ApiError::conflict("Login has already been taken"));
    }

    // Check email uniqueness
    let email_unique = repo
        .is_email_unique(&dto.email, None)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if !email_unique {
        return Err(ApiError::conflict("Email has already been taken"));
    }

    // Hash password if provided
    let (hashed_password, salt) = if let Some(ref password) = dto.password {
        let salt = generate_salt();
        let hashed = hash_password(password, &salt);
        (Some(hashed), Some(salt))
    } else {
        (None, None)
    };

    let status = match dto.status.as_deref() {
        Some("active") => op_db::user_status::ACTIVE,
        Some("locked") => op_db::user_status::LOCKED,
        Some("invited") => op_db::user_status::INVITED,
        Some("registered") => op_db::user_status::REGISTERED,
        _ => op_db::user_status::INVITED,
    };

    let create_dto = op_db::CreateUserDto {
        login: dto.login,
        firstname: dto.firstname,
        lastname: dto.lastname,
        mail: dto.email,
        admin: dto.admin.unwrap_or(false),
        status,
        language: dto.language,
        hashed_password,
        salt,
    };

    let row = repo
        .create(create_dto)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    Ok((StatusCode::CREATED, HalResponse(UserResponse::from_row(row, true))))
}

/// Update a user
///
/// PATCH /api/v3/users/:id
pub async fn update_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
    Json(dto): Json<UpdateUserRequest>,
) -> ApiResult<impl IntoResponse> {
    let is_self = user.id() == id;
    let is_admin = user.0.is_admin();

    if !is_self && !is_admin {
        return Err(ApiError::forbidden("You are not authorized to update this user."));
    }

    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    // Verify user exists
    if !repo.exists(id).await.map_err(|e| ApiError::internal(format!("Database error: {}", e)))? {
        return Err(ApiError::not_found("User", id));
    }

    // Check email uniqueness if changing
    if let Some(ref email) = dto.email {
        let email_unique = repo
            .is_email_unique(email, Some(id))
            .await
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

        if !email_unique {
            return Err(ApiError::conflict("Email has already been taken"));
        }
    }

    // Hash password if provided
    let (hashed_password, salt) = if let Some(ref password) = dto.password {
        let salt = generate_salt();
        let hashed = hash_password(password, &salt);
        (Some(hashed), Some(salt))
    } else {
        (None, None)
    };

    let status = dto.status.as_deref().map(|s| match s {
        "active" => op_db::user_status::ACTIVE,
        "locked" => op_db::user_status::LOCKED,
        "invited" => op_db::user_status::INVITED,
        "registered" => op_db::user_status::REGISTERED,
        _ => op_db::user_status::ACTIVE,
    });

    // Non-admins cannot change admin status
    let admin = if is_admin { dto.admin } else { None };

    let update_dto = op_db::UpdateUserDto {
        login: None, // Login cannot be changed
        firstname: dto.firstname,
        lastname: dto.lastname,
        mail: dto.email,
        admin,
        status,
        language: dto.language,
        hashed_password,
        salt,
    };

    let row = repo
        .update(id, update_dto)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("User", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(HalResponse(UserResponse::from_row(row, true)))
}

/// Delete a user (admin only)
///
/// DELETE /api/v3/users/:id
pub async fn delete_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can delete users."));
    }

    if user.id() == id {
        return Err(ApiError::conflict("You cannot delete your own account."));
    }

    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| match e {
            op_db::RepositoryError::NotFound(_) => ApiError::not_found("User", id),
            _ => ApiError::internal(format!("Database error: {}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Lock a user (admin only)
///
/// POST /api/v3/users/:id/lock
pub async fn lock_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can lock users."));
    }

    if user.id() == id {
        return Err(ApiError::conflict("You cannot lock your own account."));
    }

    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    // Verify user exists
    let _row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("User", id))?;

    repo.lock(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Return updated user
    let updated = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("User", id))?;

    Ok(HalResponse(UserResponse::from_row(updated, true)))
}

/// Unlock a user (admin only)
///
/// DELETE /api/v3/users/:id/lock
pub async fn unlock_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Id>,
) -> ApiResult<impl IntoResponse> {
    if !user.0.is_admin() {
        return Err(ApiError::forbidden("Only administrators can unlock users."));
    }

    let pool = state.pool()?;
    let repo = UserRepository::new(pool.clone());

    // Verify user exists
    let _row = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("User", id))?;

    repo.unlock(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Return updated user
    let updated = repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("User", id))?;

    Ok(HalResponse(UserResponse::from_row(updated, true)))
}

// Password hashing helpers (simplified - in production use bcrypt/argon2)
fn generate_salt() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

fn hash_password(password: &str, salt: &str) -> String {
    // Simplified hash - in production use proper password hashing
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    format!("{}{}", password, salt).hash(&mut hasher);
    format!("{:x}", hasher.finish())
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
    pub language: Option<String>,
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
    pub language: Option<String>,
}

// Response types
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserCollection {
    #[serde(rename = "_type")]
    type_name: String,
    total: usize,
    count: usize,
    page_size: usize,
    offset: usize,
    #[serde(rename = "_embedded")]
    elements: Vec<UserResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResponse {
    #[serde(rename = "_type")]
    type_name: String,
    id: Id,
    login: String,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    admin: bool,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    created_at: String,
    updated_at: String,
    #[serde(rename = "_links")]
    links: UserLinks,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserLinks {
    #[serde(rename = "self")]
    self_link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    lock: Option<Link>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unlock: Option<Link>,
    memberships: Link,
}

#[derive(Debug, Serialize)]
struct Link {
    href: String,
}

impl UserResponse {
    fn from_row(row: op_db::UserRow, include_email: bool) -> Self {
        let status_str = match row.status {
            1 => "active",
            2 => "registered",
            3 => "locked",
            4 => "invited",
            _ => "unknown",
        };

        let is_locked = row.status == 3;

        UserResponse {
            type_name: "User".into(),
            id: row.id,
            login: row.login,
            first_name: row.firstname.clone(),
            last_name: row.lastname.clone(),
            name: format!("{} {}", row.firstname, row.lastname),
            email: if include_email { Some(row.mail) } else { None },
            admin: row.admin,
            status: status_str.to_string(),
            language: row.language,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            links: UserLinks {
                self_link: Link {
                    href: format!("/api/v3/users/{}", row.id),
                },
                lock: if !is_locked {
                    Some(Link {
                        href: format!("/api/v3/users/{}/lock", row.id),
                    })
                } else {
                    None
                },
                unlock: if is_locked {
                    Some(Link {
                        href: format!("/api/v3/users/{}/lock", row.id),
                    })
                } else {
                    None
                },
                memberships: Link {
                    href: format!("/api/v3/memberships?filters=[{{\"principal\":{{\"operator\":\"=\",\"values\":[\"{}\"]}}}}]", row.id),
                },
            },
        }
    }
}
