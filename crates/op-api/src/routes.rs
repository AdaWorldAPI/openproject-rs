//! API routes
//!
//! Mirrors: config/routes.rb API v3 section

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use serde::Serialize;

use crate::extractors::AppState;
use crate::handlers::{priorities, projects, queries, roles, statuses, time_entries, types, users, versions, work_packages};

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
        .nest("/roles", roles_router())
        .nest("/versions", versions_router())
        .nest("/time_entries", time_entries_router())
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
        .route("/:id/archive", post(projects::archive_project))
        .route("/:id/unarchive", post(projects::unarchive_project))
        .route("/:id/types", get(types::list_project_types))
        .route("/:id/versions", get(versions::list_project_versions))
}

fn users_router() -> Router<AppState> {
    Router::new()
        .route("/", get(users::list_users))
        .route("/", post(users::create_user))
        .route("/me", get(users::get_me))
        .route("/:id", get(users::get_user))
        .route("/:id", patch(users::update_user))
        .route("/:id", delete(users::delete_user))
        .route("/:id/lock", post(users::lock_user))
        .route("/:id/lock", delete(users::unlock_user))
}

fn statuses_router() -> Router<AppState> {
    Router::new()
        .route("/", get(statuses::list_statuses))
        .route("/", post(statuses::create_status))
        .route("/:id", get(statuses::get_status))
        .route("/:id", patch(statuses::update_status))
        .route("/:id", delete(statuses::delete_status))
}

fn types_router() -> Router<AppState> {
    Router::new()
        .route("/", get(types::list_types))
        .route("/", post(types::create_type))
        .route("/:id", get(types::get_type))
        .route("/:id", patch(types::update_type))
        .route("/:id", delete(types::delete_type))
}

fn priorities_router() -> Router<AppState> {
    Router::new()
        .route("/", get(priorities::list_priorities))
        .route("/", post(priorities::create_priority))
        .route("/:id", get(priorities::get_priority))
        .route("/:id", patch(priorities::update_priority))
        .route("/:id", delete(priorities::delete_priority))
}

fn roles_router() -> Router<AppState> {
    Router::new()
        .route("/", get(roles::list_roles))
        .route("/", post(roles::create_role))
        .route("/:id", get(roles::get_role))
        .route("/:id", patch(roles::update_role))
        .route("/:id", delete(roles::delete_role))
}

fn versions_router() -> Router<AppState> {
    Router::new()
        .route("/", get(versions::list_versions))
        .route("/", post(versions::create_version))
        .route("/:id", get(versions::get_version))
        .route("/:id", patch(versions::update_version))
        .route("/:id", delete(versions::delete_version))
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

fn time_entries_router() -> Router<AppState> {
    Router::new()
        .route("/", get(time_entries::list_time_entries))
        .route("/", post(time_entries::create_time_entry))
        .route("/:id", get(time_entries::get_time_entry))
        .route("/:id", patch(time_entries::update_time_entry))
        .route("/:id", delete(time_entries::delete_time_entry))
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
