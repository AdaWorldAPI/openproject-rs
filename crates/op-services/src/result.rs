//! Service Result type
//!
//! Mirrors: app/services/service_result.rb

use op_core::error::ValidationErrors;
use std::fmt;

/// Represents the result of a service call
#[derive(Debug)]
pub struct ServiceResult<T> {
    /// Whether the service call was successful
    success: bool,
    /// The result of the service call
    result: Option<T>,
    /// Errors from the service call
    errors: ValidationErrors,
    /// Message for display
    message: Option<String>,
    /// Dependent results from nested service calls
    dependent_results: Vec<ServiceResult<T>>,
}

impl<T> ServiceResult<T> {
    /// Create a successful service result
    pub fn success(result: T) -> Self {
        Self {
            success: true,
            result: Some(result),
            errors: ValidationErrors::new(),
            message: None,
            dependent_results: Vec::new(),
        }
    }

    /// Create a successful service result with a message
    pub fn success_with_message(result: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            result: Some(result),
            errors: ValidationErrors::new(),
            message: Some(message.into()),
            dependent_results: Vec::new(),
        }
    }

    /// Create a failed service result
    pub fn failure(errors: ValidationErrors) -> Self {
        Self {
            success: false,
            result: None,
            errors,
            message: None,
            dependent_results: Vec::new(),
        }
    }

    /// Create a failed service result with a single error
    pub fn failure_with_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        let mut errors = ValidationErrors::new();
        errors.add(field, message);
        Self::failure(errors)
    }

    /// Create a failed service result with a base error
    pub fn failure_with_base_error(message: impl Into<String>) -> Self {
        let mut errors = ValidationErrors::new();
        errors.add_base(message);
        Self::failure(errors)
    }

    /// Create a failed service result with a message
    pub fn failure_with_message(errors: ValidationErrors, message: impl Into<String>) -> Self {
        Self {
            success: false,
            result: None,
            errors,
            message: Some(message.into()),
            dependent_results: Vec::new(),
        }
    }

    /// Check if the service call was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the service call failed
    pub fn is_failure(&self) -> bool {
        !self.success
    }

    /// Get the result (if successful)
    pub fn result(&self) -> Option<&T> {
        self.result.as_ref()
    }

    /// Take the result (consuming it)
    pub fn take_result(&mut self) -> Option<T> {
        self.result.take()
    }

    /// Unwrap the result, panicking if it was a failure
    pub fn unwrap(self) -> T {
        self.result.expect("called unwrap on a failed ServiceResult")
    }

    /// Unwrap the result or return a default value
    pub fn unwrap_or(self, default: T) -> T {
        self.result.unwrap_or(default)
    }

    /// Get the errors
    pub fn errors(&self) -> &ValidationErrors {
        &self.errors
    }

    /// Get mutable reference to errors
    pub fn errors_mut(&mut self) -> &mut ValidationErrors {
        &mut self.errors
    }

    /// Get the message
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Set the message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Get dependent results
    pub fn dependent_results(&self) -> &[ServiceResult<T>] {
        &self.dependent_results
    }

    /// Add a dependent result
    pub fn add_dependent(&mut self, result: ServiceResult<T>) {
        if result.is_failure() {
            self.success = false;
        }
        self.dependent_results.push(result);
    }

    /// Get all results (including dependent)
    pub fn all_results(&self) -> Vec<&T> {
        let mut results = Vec::new();
        if let Some(ref r) = self.result {
            results.push(r);
        }
        for dep in &self.dependent_results {
            results.extend(dep.all_results());
        }
        results
    }

    /// Get all errors (including dependent)
    pub fn all_errors(&self) -> Vec<&ValidationErrors> {
        let mut errors = vec![&self.errors];
        for dep in &self.dependent_results {
            errors.extend(dep.all_errors());
        }
        errors
    }

    /// Full error messages
    pub fn full_messages(&self) -> Vec<String> {
        self.errors.full_messages()
    }

    /// Execute closure if successful
    pub fn on_success<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if self.success {
            if let Some(ref result) = self.result {
                f(result);
            }
        }
        self
    }

    /// Execute closure if failed
    pub fn on_failure<F>(self, f: F) -> Self
    where
        F: FnOnce(&ValidationErrors),
    {
        if !self.success {
            f(&self.errors);
        }
        self
    }

    /// Map the result if successful
    pub fn map<U, F>(self, f: F) -> ServiceResult<U>
    where
        F: FnOnce(T) -> U,
    {
        if self.success {
            ServiceResult {
                success: true,
                result: self.result.map(f),
                errors: self.errors,
                message: self.message,
                dependent_results: Vec::new(),
            }
        } else {
            ServiceResult {
                success: false,
                result: None,
                errors: self.errors,
                message: self.message,
                dependent_results: Vec::new(),
            }
        }
    }

    /// Chain with another service call if successful
    pub fn and_then<U, F>(self, f: F) -> ServiceResult<U>
    where
        F: FnOnce(T) -> ServiceResult<U>,
    {
        if self.success {
            if let Some(result) = self.result {
                f(result)
            } else {
                ServiceResult::failure(self.errors)
            }
        } else {
            ServiceResult::failure(self.errors)
        }
    }
}

impl<T: Clone> Clone for ServiceResult<T> {
    fn clone(&self) -> Self {
        Self {
            success: self.success,
            result: self.result.clone(),
            errors: self.errors.clone(),
            message: self.message.clone(),
            dependent_results: self.dependent_results.clone(),
        }
    }
}

impl<T: Default> Default for ServiceResult<T> {
    fn default() -> Self {
        Self::success(T::default())
    }
}

impl<T> From<Result<T, ValidationErrors>> for ServiceResult<T> {
    fn from(result: Result<T, ValidationErrors>) -> Self {
        match result {
            Ok(value) => ServiceResult::success(value),
            Err(errors) => ServiceResult::failure(errors),
        }
    }
}

impl<T> From<ServiceResult<T>> for Result<T, ValidationErrors> {
    fn from(result: ServiceResult<T>) -> Self {
        if result.success {
            result.result.ok_or_else(|| {
                let mut errors = ValidationErrors::new();
                errors.add_base("Service succeeded but no result was returned");
                errors
            })
        } else {
            Err(result.errors)
        }
    }
}

impl<T: fmt::Display> fmt::Display for ServiceResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            if let Some(ref result) = self.result {
                write!(f, "Success: {}", result)
            } else {
                write!(f, "Success")
            }
        } else {
            write!(f, "Failure: {}", self.errors.full_messages().join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        let result = ServiceResult::success(42);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.result(), Some(&42));
    }

    #[test]
    fn test_failure_result() {
        let result: ServiceResult<i32> =
            ServiceResult::failure_with_error("field", "is invalid");
        assert!(!result.is_success());
        assert!(result.is_failure());
        assert!(result.result().is_none());
        assert!(result.errors().has_error("field"));
    }

    #[test]
    fn test_map_success() {
        let result = ServiceResult::success(42);
        let mapped = result.map(|n| n * 2);
        assert!(mapped.is_success());
        assert_eq!(mapped.result(), Some(&84));
    }

    #[test]
    fn test_map_failure() {
        let result: ServiceResult<i32> =
            ServiceResult::failure_with_error("field", "is invalid");
        let mapped = result.map(|n| n * 2);
        assert!(!mapped.is_success());
    }

    #[test]
    fn test_and_then_success() {
        let result = ServiceResult::success(42);
        let chained = result.and_then(|n| ServiceResult::success(n.to_string()));
        assert!(chained.is_success());
        assert_eq!(chained.result(), Some(&"42".to_string()));
    }

    #[test]
    fn test_and_then_failure() {
        let result: ServiceResult<i32> =
            ServiceResult::failure_with_error("field", "is invalid");
        let chained = result.and_then(|n| ServiceResult::success(n.to_string()));
        assert!(!chained.is_success());
    }

    #[test]
    fn test_on_success_callback() {
        let mut called = false;
        let result = ServiceResult::success(42);
        result.on_success(|_| called = true);
        assert!(called);
    }

    #[test]
    fn test_on_failure_callback() {
        let mut called = false;
        let result: ServiceResult<i32> =
            ServiceResult::failure_with_error("field", "is invalid");
        result.on_failure(|_| called = true);
        assert!(called);
    }

    #[test]
    fn test_dependent_results() {
        let mut result = ServiceResult::success(1);
        result.add_dependent(ServiceResult::success(2));
        result.add_dependent(ServiceResult::success(3));

        assert!(result.is_success());
        assert_eq!(result.dependent_results().len(), 2);
        assert_eq!(result.all_results().len(), 3);
    }

    #[test]
    fn test_dependent_failure_propagates() {
        let mut result = ServiceResult::success(1);
        result.add_dependent(ServiceResult::failure_with_error("x", "failed"));

        assert!(!result.is_success());
    }
}
