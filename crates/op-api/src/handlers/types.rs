//! Types API handlers
//!
//! Mirrors: lib/api/v3/types/*

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::error::ApiResult;
use crate::extractors::{AppState, AuthenticatedUser};

/// List all types
///
/// GET /api/v3/types
pub async fn list_types(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
) -> ApiResult<impl IntoResponse> {
    let types = vec![
        type_json(1, "Task", false, 1),
        type_json(2, "Milestone", true, 2),
        type_json(3, "Phase", false, 3),
        type_json(4, "Feature", false, 4),
        type_json(5, "Epic", false, 5),
        type_json(6, "User story", false, 6),
        type_json(7, "Bug", false, 7),
    ];

    let response = serde_json::json!({
        "_type": "Collection",
        "total": types.len(),
        "count": types.len(),
        "_embedded": {
            "elements": types
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get a single type
///
/// GET /api/v3/types/:id
pub async fn get_type(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let type_rep = match id {
        1 => type_json(1, "Task", false, 1),
        2 => type_json(2, "Milestone", true, 2),
        3 => type_json(3, "Phase", false, 3),
        4 => type_json(4, "Feature", false, 4),
        5 => type_json(5, "Epic", false, 5),
        6 => type_json(6, "User story", false, 6),
        7 => type_json(7, "Bug", false, 7),
        _ => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "_type": "Error",
                    "errorIdentifier": "urn:openproject-org:api:v3:errors:NotFound",
                    "message": format!("Type with id {} not found.", id)
                })),
            ));
        }
    };

    Ok((StatusCode::OK, Json(type_rep)))
}

/// List types available in a project
///
/// GET /api/v3/projects/:project_id/types
pub async fn list_project_types(
    State(_state): State<AppState>,
    _user: AuthenticatedUser,
    Path(_project_id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let types = vec![
        type_json(1, "Task", false, 1),
        type_json(2, "Milestone", true, 2),
        type_json(4, "Feature", false, 4),
        type_json(7, "Bug", false, 7),
    ];

    let response = serde_json::json!({
        "_type": "Collection",
        "total": types.len(),
        "count": types.len(),
        "_embedded": {
            "elements": types
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

fn type_json(id: i64, name: &str, is_milestone: bool, position: i32) -> serde_json::Value {
    serde_json::json!({
        "_type": "Type",
        "id": id,
        "name": name,
        "isMilestone": is_milestone,
        "position": position,
        "_links": {
            "self": { "href": format!("/api/v3/types/{}", id) }
        }
    })
}
