//! Result type aliases and service result pattern
//!
//! Mirrors OpenProject's ServiceResult pattern from Ruby.

use crate::error::{OpError, ValidationErrors};

/// Standard Result type for OpenProject operations
pub type OpResult<T> = Result<T, OpError>;

/// Service result pattern - mirrors Ruby's ServiceResult
///
/// In OpenProject Ruby:
/// ```ruby
/// ServiceResult.success(result: work_package)
/// ServiceResult.failure(errors: contract.errors)
/// ```
#[derive(Debug)]
pub struct ServiceResult<T> {
    /// Whether the operation succeeded
    pub success: bool,
    /// The result value (if successful)
    pub result: Option<T>,
    /// Errors (if failed)
    pub errors: ValidationErrors,
    /// Additional state/context
    pub state: ServiceState,
    /// Dependent results from sub-operations
    pub dependent_results: Vec<ServiceResult<()>>,
}

/// Additional state that can be passed through service results
#[derive(Debug, Default, Clone)]
pub struct ServiceState {
    /// Contract that was used for validation
    pub contract_class: Option<String>,
    /// Original attributes before transformation
    pub original_attributes: Option<serde_json::Value>,
    /// Custom state entries
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl<T> ServiceResult<T> {
    /// Create a successful result
    pub fn success(result: T) -> Self {
        Self {
            success: true,
            result: Some(result),
            errors: ValidationErrors::new(),
            state: ServiceState::default(),
            dependent_results: vec![],
        }
    }

    /// Create a successful result with no value
    pub fn success_empty() -> ServiceResult<()> {
        ServiceResult {
            success: true,
            result: Some(()),
            errors: ValidationErrors::new(),
            state: ServiceState::default(),
            dependent_results: vec![],
        }
    }

    /// Create a failed result with errors
    pub fn failure(errors: ValidationErrors) -> Self {
        Self {
            success: false,
            result: None,
            errors,
            state: ServiceState::default(),
            dependent_results: vec![],
        }
    }

    /// Create a failed result with a single error message
    pub fn failure_with_message(message: impl Into<String>) -> Self {
        let mut errors = ValidationErrors::new();
        errors.add_base(message);
        Self::failure(errors)
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the result is a failure
    pub fn is_failure(&self) -> bool {
        !self.success
    }

    /// Get the result value, panicking if not successful
    pub fn unwrap(self) -> T {
        self.result.expect("Called unwrap on a failed ServiceResult")
    }

    /// Get the result value or a default
    pub fn unwrap_or(self, default: T) -> T {
        self.result.unwrap_or(default)
    }

    /// Map the result value
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ServiceResult<U> {
        ServiceResult {
            success: self.success,
            result: self.result.map(f),
            errors: self.errors,
            state: self.state,
            dependent_results: self.dependent_results,
        }
    }

    /// Chain another service call
    pub fn and_then<U, F: FnOnce(T) -> ServiceResult<U>>(self, f: F) -> ServiceResult<U> {
        if self.success {
            if let Some(result) = self.result {
                return f(result);
            }
        }
        ServiceResult {
            success: false,
            result: None,
            errors: self.errors,
            state: self.state,
            dependent_results: self.dependent_results,
        }
    }

    /// Add state to the result
    pub fn with_state(mut self, state: ServiceState) -> Self {
        self.state = state;
        self
    }

    /// Add a dependent result
    pub fn with_dependent(mut self, dependent: ServiceResult<()>) -> Self {
        self.dependent_results.push(dependent);
        self
    }

    /// Merge errors from another result
    pub fn merge_errors(&mut self, other: &ServiceResult<impl std::fmt::Debug>) {
        self.errors.merge(other.errors.clone());
    }

    /// Convert to standard Result
    pub fn into_result(self) -> OpResult<T> {
        if self.success {
            self.result.ok_or_else(|| {
                OpError::Internal("ServiceResult success but no result value".into())
            })
        } else {
            Err(OpError::Validation(self.errors))
        }
    }
}

impl<T> From<OpResult<T>> for ServiceResult<T> {
    fn from(result: OpResult<T>) -> Self {
        match result {
            Ok(value) => ServiceResult::success(value),
            Err(OpError::Validation(errors)) => ServiceResult::failure(errors),
            Err(e) => ServiceResult::failure_with_message(e.to_string()),
        }
    }
}

impl<T> From<ServiceResult<T>> for OpResult<T> {
    fn from(result: ServiceResult<T>) -> Self {
        result.into_result()
    }
}
