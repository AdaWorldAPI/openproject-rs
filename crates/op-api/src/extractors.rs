//! Axum extractors for API handlers

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Query},
    http::request::Parts,
};
use op_auth::permissions::CurrentUser;
use std::sync::Arc;

use crate::error::ApiError;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub api_version: String,
    pub base_url: String,
    pub require_authentication: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_version: "3".into(),
            base_url: "http://localhost:8080".into(),
            require_authentication: true,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Arc::new(AppConfig::default()),
        }
    }
}

/// Authenticated user extractor
pub struct AuthenticatedUser(pub CurrentUser);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Check Authorization header
        if let Some(auth) = parts.headers.get("authorization") {
            if let Ok(auth_str) = auth.to_str() {
                if auth_str.starts_with("Basic ") || auth_str.starts_with("Bearer ") {
                    // Mock user for now
                    return Ok(AuthenticatedUser(CurrentUser::new(
                        1,
                        "api_user",
                        "api@example.com",
                    )));
                }
            }
        }

        if !app_state.config.require_authentication {
            return Ok(AuthenticatedUser(CurrentUser::anonymous()));
        }

        Err(ApiError::unauthorized("Authentication required"))
    }
}

impl std::ops::Deref for AuthenticatedUser {
    type Target = CurrentUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Pagination parameters
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_page_size() -> usize {
    20
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page_size: 20,
            offset: 0,
        }
    }
}

pub struct Pagination(pub PaginationParams);

#[async_trait]
impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<PaginationParams>::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|_| Query(PaginationParams::default()));
        Ok(Pagination(params))
    }
}

impl std::ops::Deref for Pagination {
    type Target = PaginationParams;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// HAL+JSON response wrapper
pub struct HalResponse<T: serde::Serialize>(pub T);

impl<T: serde::Serialize> axum::response::IntoResponse for HalResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let json = serde_json::to_string(&self.0).unwrap_or_default();
        axum::response::Response::builder()
            .status(200)
            .header("content-type", "application/hal+json; charset=utf-8")
            .body(axum::body::Body::from(json))
            .unwrap()
    }
}
