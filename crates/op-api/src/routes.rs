//! API routes
//!
//! Mirrors: config/routes.rb API v3 section

use axum::{
    routing::{delete, get, post},
    Router,
};
use serde::Serialize;

use crate::extractors::AppState;
use crate::handlers::{projects, work_packages};

/// Create the complete API router
pub fn router() -> Router<AppState> {
    Router::new().nest("/api/v3", api_v3_router())
}

fn api_v3_router() -> Router<AppState> {
    Router::new()
        .route("/", get(api_root))
        .nest("/work_packages", work_packages_router())
        .nest("/projects", projects_router())
}

fn work_packages_router() -> Router<AppState> {
    Router::new()
        .route("/", get(work_packages::list_work_packages))
        .route("/", post(work_packages::create_work_package))
        .route("/:id", get(work_packages::get_work_package))
        .route("/:id", delete(work_packages::delete_work_package))
}

fn projects_router() -> Router<AppState> {
    Router::new()
        .route("/", get(projects::list_projects))
        .route("/", post(projects::create_project))
        .route("/:id", get(projects::get_project))
        .route("/:id", delete(projects::delete_project))
}

async fn api_root() -> axum::Json<ApiRoot> {
    axum::Json(ApiRoot {
        type_name: "Root".into(),
        instance_name: "OpenProject RS".into(),
    })
}

#[derive(Serialize)]
struct ApiRoot {
    #[serde(rename = "_type")]
    type_name: String,
    #[serde(rename = "instanceName")]
    instance_name: String,
}
