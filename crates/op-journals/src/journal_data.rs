//! Journal Data Storage
//!
//! Stores the actual data changes for journals, separate from the journal
//! metadata for efficiency.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Journal data - stores complete state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalData {
    /// Type discriminator for deserialization
    #[serde(rename = "_type")]
    pub data_type: String,
    /// The actual data fields
    #[serde(flatten)]
    pub fields: HashMap<String, JsonValue>,
}

impl JournalData {
    /// Create new journal data
    pub fn new(data_type: impl Into<String>) -> Self {
        Self {
            data_type: data_type.into(),
            fields: HashMap::new(),
        }
    }

    /// Set a field value
    pub fn set(&mut self, key: impl Into<String>, value: impl Serialize) {
        self.fields.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(JsonValue::Null),
        );
    }

    /// Get a field value
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.fields.get(key)
    }

    /// Get a field value as a specific type
    pub fn get_as<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.fields.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Create work package journal data
    pub fn work_package() -> WorkPackageJournalDataBuilder {
        WorkPackageJournalDataBuilder::new()
    }
}

/// Builder for work package journal data
pub struct WorkPackageJournalDataBuilder {
    data: JournalData,
}

impl WorkPackageJournalDataBuilder {
    fn new() -> Self {
        Self {
            data: JournalData::new("WorkPackageJournal"),
        }
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.data.set("subject", subject.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.data.set("description", description.into());
        self
    }

    pub fn status_id(mut self, status_id: i64) -> Self {
        self.data.set("status_id", status_id);
        self
    }

    pub fn type_id(mut self, type_id: i64) -> Self {
        self.data.set("type_id", type_id);
        self
    }

    pub fn priority_id(mut self, priority_id: i64) -> Self {
        self.data.set("priority_id", priority_id);
        self
    }

    pub fn assigned_to_id(mut self, assigned_to_id: Option<i64>) -> Self {
        self.data.set("assigned_to_id", assigned_to_id);
        self
    }

    pub fn done_ratio(mut self, done_ratio: i32) -> Self {
        self.data.set("done_ratio", done_ratio);
        self
    }

    pub fn estimated_hours(mut self, hours: Option<f64>) -> Self {
        self.data.set("estimated_hours", hours);
        self
    }

    pub fn build(self) -> JournalData {
        self.data
    }
}

/// Journal diff - what changed between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalDiff {
    /// Field changes
    pub changes: Vec<JournalDetails>,
}

impl JournalDiff {
    /// Create a new diff
    pub fn new() -> Self {
        Self { changes: Vec::new() }
    }

    /// Add a change
    pub fn add(&mut self, detail: JournalDetails) {
        self.changes.push(detail);
    }

    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Get number of changes
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Compute diff between two journal data
    pub fn compute(old: &JournalData, new: &JournalData) -> Self {
        let mut diff = Self::new();

        // Find changed and added fields
        for (key, new_value) in &new.fields {
            match old.fields.get(key) {
                Some(old_value) if old_value != new_value => {
                    diff.add(JournalDetails::changed(
                        key.clone(),
                        old_value.clone(),
                        new_value.clone(),
                    ));
                }
                None => {
                    diff.add(JournalDetails::added(key.clone(), new_value.clone()));
                }
                _ => {}
            }
        }

        // Find removed fields
        for (key, old_value) in &old.fields {
            if !new.fields.contains_key(key) {
                diff.add(JournalDetails::removed(key.clone(), old_value.clone()));
            }
        }

        diff
    }
}

impl Default for JournalDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Details of a single change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalDetails {
    /// Property/field that changed
    pub property: String,
    /// Change type
    pub change_type: ChangeType,
    /// Old value (None for additions)
    pub old_value: Option<JsonValue>,
    /// New value (None for deletions)
    pub new_value: Option<JsonValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Changed,
    Added,
    Removed,
}

impl JournalDetails {
    /// Create a change detail
    pub fn changed(property: impl Into<String>, old: JsonValue, new: JsonValue) -> Self {
        Self {
            property: property.into(),
            change_type: ChangeType::Changed,
            old_value: Some(old),
            new_value: Some(new),
        }
    }

    /// Create an addition detail
    pub fn added(property: impl Into<String>, value: JsonValue) -> Self {
        Self {
            property: property.into(),
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(value),
        }
    }

    /// Create a removal detail
    pub fn removed(property: impl Into<String>, value: JsonValue) -> Self {
        Self {
            property: property.into(),
            change_type: ChangeType::Removed,
            old_value: Some(value),
            new_value: None,
        }
    }

    /// Format the change for display
    pub fn format_for_display(&self) -> String {
        match self.change_type {
            ChangeType::Changed => {
                format!(
                    "{}: {} → {}",
                    self.property,
                    self.old_value.as_ref().map_or("(empty)".to_string(), |v| v.to_string()),
                    self.new_value.as_ref().map_or("(empty)".to_string(), |v| v.to_string()),
                )
            }
            ChangeType::Added => {
                format!(
                    "{} set to {}",
                    self.property,
                    self.new_value.as_ref().map_or("(empty)".to_string(), |v| v.to_string()),
                )
            }
            ChangeType::Removed => {
                format!(
                    "{} removed (was {})",
                    self.property,
                    self.old_value.as_ref().map_or("(empty)".to_string(), |v| v.to_string()),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_data_creation() {
        let data = JournalData::work_package()
            .subject("Test work package")
            .status_id(1)
            .done_ratio(50)
            .build();

        assert_eq!(data.get_as::<String>("subject"), Some("Test work package".to_string()));
        assert_eq!(data.get_as::<i64>("status_id"), Some(1));
        assert_eq!(data.get_as::<i32>("done_ratio"), Some(50));
    }

    #[test]
    fn test_journal_diff_compute() {
        let old = JournalData::work_package()
            .subject("Old subject")
            .status_id(1)
            .build();

        let new = JournalData::work_package()
            .subject("New subject")
            .status_id(2)
            .priority_id(5)
            .build();

        let diff = JournalDiff::compute(&old, &new);

        assert!(!diff.is_empty());
        // Should have: subject changed, status_id changed, priority_id added
        let change_count = diff.len();
        assert!(change_count >= 2);
    }

    #[test]
    fn test_journal_details_display() {
        let detail = JournalDetails::changed(
            "status_id",
            serde_json::json!(1),
            serde_json::json!(2),
        );

        let display = detail.format_for_display();
        assert!(display.contains("status_id"));
        assert!(display.contains("1"));
        assert!(display.contains("2"));
    }

    #[test]
    fn test_empty_diff() {
        let data1 = JournalData::work_package()
            .subject("Same")
            .status_id(1)
            .build();

        let data2 = JournalData::work_package()
            .subject("Same")
            .status_id(1)
            .build();

        let diff = JournalDiff::compute(&data1, &data2);
        assert!(diff.is_empty());
    }
}
