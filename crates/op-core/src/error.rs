//! Core error types for OpenProject RS
//!
//! Maps to Ruby's error handling patterns and contract validation errors.

use std::collections::HashMap;
use thiserror::Error;

/// Core error type for all OpenProject operations
#[derive(Error, Debug)]
pub enum OpError {
    #[error("Not found: {entity} with {field}={value}")]
    NotFound {
        entity: &'static str,
        field: &'static str,
        value: String,
    },

    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Forbidden: {message}")]
    Forbidden { message: String },

    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Contract violation: {0}")]
    Contract(#[from] ContractError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Rate limited: retry after {retry_after_seconds}s")]
    RateLimited { retry_after_seconds: u64 },

    #[error("Conflict: {message}")]
    Conflict { message: String },
}

/// Validation errors collection (mirrors Rails ActiveModel::Errors)
#[derive(Error, Debug, Default, Clone)]
#[error("Validation errors: {errors:?}")]
pub struct ValidationErrors {
    /// Field-specific errors: field_name -> Vec<error_messages>
    pub errors: HashMap<String, Vec<String>>,
    /// Base errors not tied to a specific field
    pub base_errors: Vec<String>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors
            .entry(field.into())
            .or_default()
            .push(message.into());
    }

    pub fn add_base(&mut self, message: impl Into<String>) {
        self.base_errors.push(message.into());
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.base_errors.is_empty()
    }

    /// Check if there are errors for a specific field
    pub fn has_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    /// Get errors for a specific field
    pub fn get(&self, field: &str) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    pub fn merge(&mut self, other: ValidationErrors) {
        for (field, messages) in other.errors {
            self.errors.entry(field).or_default().extend(messages);
        }
        self.base_errors.extend(other.base_errors);
    }

    pub fn full_messages(&self) -> Vec<String> {
        let mut messages = self.base_errors.clone();
        for (field, field_messages) in &self.errors {
            for msg in field_messages {
                messages.push(format!("{} {}", field, msg));
            }
        }
        messages
    }
}

/// Contract validation error (mirrors OpenProject's Contract errors)
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Attribute {attribute} is invalid: {message}")]
    AttributeInvalid { attribute: String, message: String },

    #[error("Attribute {attribute} is not writable")]
    AttributeNotWritable { attribute: String },

    #[error("Base contract error: {message}")]
    Base { message: String },

    #[error("Multiple contract errors")]
    Multiple { errors: ValidationErrors },
}

impl From<ContractError> for ValidationErrors {
    fn from(err: ContractError) -> Self {
        let mut errors = ValidationErrors::new();
        match err {
            ContractError::AttributeInvalid { attribute, message } => {
                errors.add(attribute, message);
            }
            ContractError::AttributeNotWritable { attribute } => {
                errors.add(attribute, "is not writable");
            }
            ContractError::Base { message } => {
                errors.add_base(message);
            }
            ContractError::Multiple { errors: e } => {
                return e;
            }
        }
        errors
    }
}

/// HTTP status code mapping for errors
impl OpError {
    pub fn status_code(&self) -> u16 {
        match self {
            OpError::NotFound { .. } => 404,
            OpError::Unauthorized { .. } => 401,
            OpError::Forbidden { .. } => 403,
            OpError::Validation(_) | OpError::Contract(_) => 422,
            OpError::RateLimited { .. } => 429,
            OpError::Conflict { .. } => 409,
            OpError::Database(_) | OpError::Internal(_) => 500,
            OpError::Config(_) => 500,
            OpError::ExternalService { .. } => 502,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            OpError::NotFound { .. } => "not_found",
            OpError::Unauthorized { .. } => "unauthorized",
            OpError::Forbidden { .. } => "forbidden",
            OpError::Validation(_) => "validation_failed",
            OpError::Contract(_) => "contract_violated",
            OpError::Database(_) => "database_error",
            OpError::Internal(_) => "internal_error",
            OpError::Config(_) => "configuration_error",
            OpError::ExternalService { .. } => "external_service_error",
            OpError::RateLimited { .. } => "rate_limited",
            OpError::Conflict { .. } => "conflict",
        }
    }
}
