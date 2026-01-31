//! Query Sort Orders
//!
//! Mirrors: app/models/queries/sorts/*
//!
//! Sort orders define how query results should be ordered.

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Ascending order (A-Z, 1-9, oldest first)
    #[default]
    Asc,
    /// Descending order (Z-A, 9-1, newest first)
    Desc,
}

impl SortDirection {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Some(Self::Asc),
            "desc" | "descending" => Some(Self::Desc),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    /// Get the opposite direction
    pub fn reverse(&self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

/// A single sort criterion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortCriterion {
    /// The attribute to sort by
    pub attribute: String,
    /// The sort direction
    pub direction: SortDirection,
}

impl SortCriterion {
    /// Create a new sort criterion
    pub fn new(attribute: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            attribute: attribute.into(),
            direction,
        }
    }

    /// Create ascending sort
    pub fn asc(attribute: impl Into<String>) -> Self {
        Self::new(attribute, SortDirection::Asc)
    }

    /// Create descending sort
    pub fn desc(attribute: impl Into<String>) -> Self {
        Self::new(attribute, SortDirection::Desc)
    }

    /// Reverse the sort direction
    pub fn reversed(mut self) -> Self {
        self.direction = self.direction.reverse();
        self
    }
}

/// Collection of sort criteria
#[derive(Debug, Clone, Default)]
pub struct SortOrder {
    criteria: Vec<SortCriterion>,
}

impl SortOrder {
    /// Create a new empty sort order
    pub fn new() -> Self {
        Self { criteria: vec![] }
    }

    /// Create with a single criterion
    pub fn by(attribute: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            criteria: vec![SortCriterion::new(attribute, direction)],
        }
    }

    /// Create with ascending sort on single attribute
    pub fn by_asc(attribute: impl Into<String>) -> Self {
        Self::by(attribute, SortDirection::Asc)
    }

    /// Create with descending sort on single attribute
    pub fn by_desc(attribute: impl Into<String>) -> Self {
        Self::by(attribute, SortDirection::Desc)
    }

    /// Add a sort criterion
    pub fn add(&mut self, criterion: SortCriterion) -> &mut Self {
        self.criteria.push(criterion);
        self
    }

    /// Add a sort criterion (builder pattern)
    pub fn then(mut self, criterion: SortCriterion) -> Self {
        self.criteria.push(criterion);
        self
    }

    /// Add ascending sort
    pub fn then_asc(self, attribute: impl Into<String>) -> Self {
        self.then(SortCriterion::asc(attribute))
    }

    /// Add descending sort
    pub fn then_desc(self, attribute: impl Into<String>) -> Self {
        self.then(SortCriterion::desc(attribute))
    }

    /// Get all sort criteria
    pub fn criteria(&self) -> &[SortCriterion] {
        &self.criteria
    }

    /// Check if any sort is defined
    pub fn is_empty(&self) -> bool {
        self.criteria.is_empty()
    }

    /// Get number of sort criteria
    pub fn len(&self) -> usize {
        self.criteria.len()
    }

    /// Get the primary (first) sort criterion
    pub fn primary(&self) -> Option<&SortCriterion> {
        self.criteria.first()
    }

    /// Check if sorting by a specific attribute
    pub fn sorts_by(&self, attribute: &str) -> bool {
        self.criteria.iter().any(|c| c.attribute == attribute)
    }

    /// Remove sort for a specific attribute
    pub fn remove_sort_for(&mut self, attribute: &str) {
        self.criteria.retain(|c| c.attribute != attribute);
    }

    /// Clear all sort criteria
    pub fn clear(&mut self) {
        self.criteria.clear();
    }
}

/// Common sort attributes for work packages
pub mod attributes {
    pub const ID: &str = "id";
    pub const SUBJECT: &str = "subject";
    pub const STATUS: &str = "status";
    pub const TYPE: &str = "type";
    pub const PRIORITY: &str = "priority";
    pub const ASSIGNED_TO: &str = "assigned_to";
    pub const AUTHOR: &str = "author";
    pub const PROJECT: &str = "project";
    pub const START_DATE: &str = "start_date";
    pub const DUE_DATE: &str = "due_date";
    pub const ESTIMATED_HOURS: &str = "estimated_hours";
    pub const DONE_RATIO: &str = "done_ratio";
    pub const CREATED_AT: &str = "created_at";
    pub const UPDATED_AT: &str = "updated_at";
    pub const VERSION: &str = "version";
    pub const CATEGORY: &str = "category";
    pub const PARENT: &str = "parent";
    pub const SPENT_HOURS: &str = "spent_hours";
    pub const REMAINING_HOURS: &str = "remaining_hours";
    pub const MANUAL_SORT: &str = "manual_sort";
}

/// Default sort order for work packages
pub fn default_work_package_sort() -> SortOrder {
    SortOrder::by_desc(attributes::ID)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_direction() {
        assert_eq!(SortDirection::from_str("asc"), Some(SortDirection::Asc));
        assert_eq!(SortDirection::from_str("DESC"), Some(SortDirection::Desc));
        assert_eq!(SortDirection::Asc.reverse(), SortDirection::Desc);
        assert_eq!(SortDirection::Desc.reverse(), SortDirection::Asc);
    }

    #[test]
    fn test_sort_criterion() {
        let criterion = SortCriterion::asc("created_at");
        assert_eq!(criterion.attribute, "created_at");
        assert_eq!(criterion.direction, SortDirection::Asc);

        let reversed = criterion.reversed();
        assert_eq!(reversed.direction, SortDirection::Desc);
    }

    #[test]
    fn test_sort_order() {
        let order = SortOrder::by_desc("updated_at")
            .then_asc("id");

        assert_eq!(order.len(), 2);
        assert!(order.sorts_by("updated_at"));
        assert!(order.sorts_by("id"));
        assert!(!order.sorts_by("subject"));

        let primary = order.primary().unwrap();
        assert_eq!(primary.attribute, "updated_at");
        assert_eq!(primary.direction, SortDirection::Desc);
    }

    #[test]
    fn test_empty_sort_order() {
        let order = SortOrder::new();
        assert!(order.is_empty());
        assert_eq!(order.len(), 0);
        assert!(order.primary().is_none());
    }

    #[test]
    fn test_remove_sort() {
        let mut order = SortOrder::by_desc("updated_at")
            .then_asc("id")
            .then_asc("subject");

        assert_eq!(order.len(), 3);
        order.remove_sort_for("id");
        assert_eq!(order.len(), 2);
        assert!(!order.sorts_by("id"));
    }
}
