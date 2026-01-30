//! Base contract system
//!
//! Mirrors: app/contracts/base_contract.rb

use op_core::error::ValidationErrors;
use op_core::traits::Id;

/// Result of contract validation
pub type ValidationResult = Result<(), ValidationErrors>;

/// Trait for user context in contracts
pub trait UserContext: Send + Sync {
    fn id(&self) -> Id;
    fn is_admin(&self) -> bool;
    fn is_anonymous(&self) -> bool;
    fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool;
    fn allowed_globally(&self, permission: &str) -> bool;
    fn allowed_for_work_package(&self, permission: &str, work_package_id: Id) -> bool {
        // Default implementation - override in concrete types
        let _ = (permission, work_package_id);
        false
    }
}

/// Base contract trait
pub trait Contract<T>: Send + Sync {
    /// Validate the entity
    fn validate(&self, entity: &T) -> ValidationResult;

    /// Check if an attribute is writable
    fn is_writable(&self, _attribute: &str) -> bool {
        true
    }
}

/// Change tracking for update contracts
#[derive(Debug, Default, Clone)]
pub struct ChangeTracker {
    changed_attributes: std::collections::HashSet<String>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_changed(&mut self, attribute: impl Into<String>) {
        self.changed_attributes.insert(attribute.into());
    }

    pub fn is_changed(&self, attribute: &str) -> bool {
        self.changed_attributes.contains(attribute)
    }

    pub fn changed_attributes(&self) -> &std::collections::HashSet<String> {
        &self.changed_attributes
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_tracker() {
        let mut tracker = ChangeTracker::new();
        assert!(!tracker.is_changed("subject"));

        tracker.mark_changed("subject");
        assert!(tracker.is_changed("subject"));
        assert!(!tracker.is_changed("description"));
    }
}
