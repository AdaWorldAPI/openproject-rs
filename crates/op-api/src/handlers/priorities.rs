//! Priorities API handlers
//!
//! Mirrors: lib/api/v3/priorities/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::error::ApiResult;
use crate::extractors::{AppState, AuthenticatedUser};

/// List all priorities
///
/// GET /api/v3/priorities
pub async fn list_priorities(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    let priorities = vec![
        priority_json(1, "Low", false, 1),
        priority_json(2, "Normal", true, 2),
        priority_json(3, "High", false, 3),
        priority_json(4, "Immediate", false, 4),
    ];

    let response = serde_json::json!({
        "_type": "Collection",
        "total": priorities.len(),
        "count": priorities.len(),
        "_embedded": {
            "elements": priorities
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get a single priority
///
/// GET /api/v3/priorities/:id
pub async fn get_priority(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let priority = match id {
        1 => priority_json(1, "Low", false, 1),
        2 => priority_json(2, "Normal", true, 2),
        3 => priority_json(3, "High", false, 3),
        4 => priority_json(4, "Immediate", false, 4),
        _ => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "_type": "Error",
                    "errorIdentifier": "urn:openproject-org:api:v3:errors:NotFound",
                    "message": format!("Priority with id {} not found.", id)
                })),
            ));
        }
    };

    Ok((StatusCode::OK, Json(priority)))
}

fn priority_json(id: i64, name: &str, is_default: bool, position: i32) -> serde_json::Value {
    serde_json::json!({
        "_type": "Priority",
        "id": id,
        "name": name,
        "isDefault": is_default,
        "isActive": true,
        "position": position,
        "_links": {
            "self": { "href": format!("/api/v3/priorities/{}", id) }
        }
    })
}
