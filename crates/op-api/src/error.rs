//! API error handling
//!
//! Provides HTTP error types with HAL+JSON responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use op_core::error::ValidationErrors;
use serde::Serialize;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    NotFound { resource: &'static str, id: String },
    Validation(ValidationErrors),
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl ApiError {
    pub fn not_found(resource: &'static str, id: impl std::fmt::Display) -> Self {
        ApiError::NotFound { resource, id: id.to_string() }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ApiError::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        ApiError::Forbidden(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        ApiError::Conflict(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal(msg.into())
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
struct HalError {
    #[serde(rename = "_type")]
    type_name: String,
    #[serde(rename = "errorIdentifier")]
    error_identifier: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error = match &self {
            ApiError::NotFound { resource, id } => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:NotFound".into(),
                message: format!("{} with id {} not found", resource, id),
            },
            ApiError::Validation(errors) => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:PropertyConstraintViolation".into(),
                message: errors.full_messages().join(", "),
            },
            ApiError::Unauthorized(msg) => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:Unauthenticated".into(),
                message: msg.clone(),
            },
            ApiError::Forbidden(msg) => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:MissingPermission".into(),
                message: msg.clone(),
            },
            ApiError::BadRequest(msg) => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:InvalidRequestBody".into(),
                message: msg.clone(),
            },
            ApiError::Conflict(msg) => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:UpdateConflict".into(),
                message: msg.clone(),
            },
            ApiError::Internal(msg) => HalError {
                type_name: "Error".into(),
                error_identifier: "urn:openproject-org:api:v3:errors:InternalError".into(),
                message: msg.clone(),
            },
        };

        (status, Json(error)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
