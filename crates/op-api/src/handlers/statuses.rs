//! Statuses API handlers
//!
//! Mirrors: lib/api/v3/statuses/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::error::ApiResult;
use crate::extractors::{AppState, AuthenticatedUser};

/// List all statuses
///
/// GET /api/v3/statuses
pub async fn list_statuses(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    let statuses = vec![
        status_json(1, "New", false, true, 1),
        status_json(2, "In progress", false, false, 2),
        status_json(3, "On hold", false, false, 3),
        status_json(4, "Rejected", true, false, 4),
        status_json(5, "Closed", true, false, 5),
    ];

    let response = serde_json::json!({
        "_type": "Collection",
        "total": statuses.len(),
        "count": statuses.len(),
        "_embedded": {
            "elements": statuses
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get a single status
///
/// GET /api/v3/statuses/:id
pub async fn get_status(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let status = match id {
        1 => status_json(1, "New", false, true, 1),
        2 => status_json(2, "In progress", false, false, 2),
        3 => status_json(3, "On hold", false, false, 3),
        4 => status_json(4, "Rejected", true, false, 4),
        5 => status_json(5, "Closed", true, false, 5),
        _ => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "_type": "Error",
                    "errorIdentifier": "urn:openproject-org:api:v3:errors:NotFound",
                    "message": format!("Status with id {} not found.", id)
                })),
            ));
        }
    };

    Ok((StatusCode::OK, Json(status)))
}

fn status_json(id: i64, name: &str, is_closed: bool, is_default: bool, position: i32) -> serde_json::Value {
    serde_json::json!({
        "_type": "Status",
        "id": id,
        "name": name,
        "isClosed": is_closed,
        "isDefault": is_default,
        "isReadonly": false,
        "position": position,
        "_links": {
            "self": { "href": format!("/api/v3/statuses/{}", id) }
        }
    })
}
