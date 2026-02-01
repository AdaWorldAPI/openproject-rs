//! API routes
//!
//! Mirrors: config/routes.rb API v3 section

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use serde::Serialize;

use crate::extractors::AppState;
use crate::handlers::{priorities, projects, queries, statuses, types, users, work_packages};

/// Create the complete API router
pub fn router() -> Router<AppState> {
    Router::new().nest("/api/v3", api_v3_router())
}

fn api_v3_router() -> Router<AppState> {
    Router::new()
        .route("/", get(api_root))
        .nest("/work_packages", work_packages_router())
        .nest("/projects", projects_router())
        .nest("/users", users_router())
        .nest("/queries", queries_router())
        .nest("/statuses", statuses_router())
        .nest("/types", types_router())
        .nest("/priorities", priorities_router())
}

fn work_packages_router() -> Router<AppState> {
    Router::new()
        .route("/", get(work_packages::list_work_packages))
        .route("/", post(work_packages::create_work_package))
        .route("/:id", get(work_packages::get_work_package))
        .route("/:id", patch(work_packages::update_work_package))
        .route("/:id", delete(work_packages::delete_work_package))
}

fn projects_router() -> Router<AppState> {
    Router::new()
        .route("/", get(projects::list_projects))
        .route("/", post(projects::create_project))
        .route("/:id", get(projects::get_project))
        .route("/:id", patch(projects::update_project))
        .route("/:id", delete(projects::delete_project))
        .route("/:id/types", get(types::list_project_types))
}

fn users_router() -> Router<AppState> {
    Router::new()
        .route("/", get(users::list_users))
        .route("/", post(users::create_user))
        .route("/me", get(users::get_me))
        .route("/:id", get(users::get_user))
        .route("/:id", patch(users::update_user))
        .route("/:id", delete(users::delete_user))
}

fn statuses_router() -> Router<AppState> {
    Router::new()
        .route("/", get(statuses::list_statuses))
        .route("/:id", get(statuses::get_status))
}

fn types_router() -> Router<AppState> {
    Router::new()
        .route("/", get(types::list_types))
        .route("/:id", get(types::get_type))
}

fn priorities_router() -> Router<AppState> {
    Router::new()
        .route("/", get(priorities::list_priorities))
        .route("/:id", get(priorities::get_priority))
}

fn queries_router() -> Router<AppState> {
    Router::new()
        .route("/", get(queries::list_queries))
        .route("/", post(queries::create_query))
        .route("/default", get(queries::get_default_query))
        .route("/form", get(queries::query_form))
        .route("/available_projects", get(queries::available_projects))
        .route("/:id", get(queries::get_query))
        .route("/:id", patch(queries::update_query))
        .route("/:id", delete(queries::delete_query))
        .route("/:id/star", post(queries::star_query))
        .route("/:id/star", delete(queries::unstar_query))
}

async fn api_root() -> axum::Json<ApiRoot> {
    axum::Json(ApiRoot {
        type_name: "Root".into(),
        instance_name: "OpenProject RS".into(),
        core_version: "15.0.0".into(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiRoot {
    #[serde(rename = "_type")]
    type_name: String,
    instance_name: String,
    core_version: String,
}
