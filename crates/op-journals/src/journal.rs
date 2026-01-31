//! Journal Model
//!
//! Mirrors: app/models/journal.rb

use chrono::{DateTime, Utc};
use op_core::traits::Id;
use serde::{Deserialize, Serialize};

/// Journal type (what kind of entity this journal belongs to)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalType {
    WorkPackage,
    Project,
    User,
    Wiki,
    Meeting,
    Budget,
    Document,
    TimeEntry,
    News,
    Message,
}

impl JournalType {
    /// Get the database type name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkPackage => "WorkPackage",
            Self::Project => "Project",
            Self::User => "User",
            Self::Wiki => "WikiContent",
            Self::Meeting => "Meeting",
            Self::Budget => "Budget",
            Self::Document => "Document",
            Self::TimeEntry => "TimeEntry",
            Self::News => "News",
            Self::Message => "Message",
        }
    }

    /// Parse from database type name
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "WorkPackage" => Some(Self::WorkPackage),
            "Project" => Some(Self::Project),
            "User" => Some(Self::User),
            "WikiContent" | "Wiki" => Some(Self::Wiki),
            "Meeting" => Some(Self::Meeting),
            "Budget" => Some(Self::Budget),
            "Document" => Some(Self::Document),
            "TimeEntry" => Some(Self::TimeEntry),
            "News" => Some(Self::News),
            "Message" => Some(Self::Message),
            _ => None,
        }
    }
}

/// Journal version (for detecting concurrent modifications)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JournalVersion(pub i32);

impl JournalVersion {
    pub fn new(version: i32) -> Self {
        Self(version)
    }

    pub fn initial() -> Self {
        Self(1)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<i32> for JournalVersion {
    fn from(v: i32) -> Self {
        Self(v)
    }
}

impl From<JournalVersion> for i32 {
    fn from(v: JournalVersion) -> Self {
        v.0
    }
}

/// A journal entry (audit record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    /// Journal ID
    pub id: Option<Id>,
    /// Type of the journable entity
    pub journable_type: JournalType,
    /// ID of the journable entity
    pub journable_id: Id,
    /// Version number of this journal entry
    pub version: JournalVersion,
    /// User who made the change
    pub user_id: Id,
    /// Notes/comments for this change
    pub notes: Option<String>,
    /// ID of the activity this journal belongs to
    pub activity_id: Option<Id>,
    /// Timestamp of the change
    pub created_at: DateTime<Utc>,
    /// Timestamp of last update (for editable notes)
    pub updated_at: DateTime<Utc>,
    /// Data stored separately (for space efficiency)
    pub data_id: Option<Id>,
    /// Cause of the journal entry
    pub cause: JournalCause,
}

/// What caused this journal entry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalCause {
    /// Type of cause
    pub cause_type: CauseType,
    /// Additional context
    pub context: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CauseType {
    /// Manual user action
    #[default]
    UserAction,
    /// System-initiated change
    SystemChange,
    /// Workflow/automation
    Workflow,
    /// Import from external system
    Import,
    /// API request
    Api,
    /// Bulk update
    BulkUpdate,
}

impl Journal {
    /// Create a new journal entry
    pub fn new(
        journable_type: JournalType,
        journable_id: Id,
        version: JournalVersion,
        user_id: Id,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            journable_type,
            journable_id,
            version,
            user_id,
            notes: None,
            activity_id: None,
            created_at: now,
            updated_at: now,
            data_id: None,
            cause: JournalCause::default(),
        }
    }

    /// Create the initial journal (creation of entity)
    pub fn initial(journable_type: JournalType, journable_id: Id, user_id: Id) -> Self {
        Self::new(journable_type, journable_id, JournalVersion::initial(), user_id)
    }

    /// Set notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set cause
    pub fn with_cause(mut self, cause_type: CauseType, context: Option<String>) -> Self {
        self.cause = JournalCause {
            cause_type,
            context,
        };
        self
    }

    /// Check if this is the initial journal (creation)
    pub fn is_initial(&self) -> bool {
        self.version.0 == 1
    }

    /// Check if this journal has notes
    pub fn has_notes(&self) -> bool {
        self.notes.as_ref().map_or(false, |n| !n.trim().is_empty())
    }
}

/// Builder for creating journal entries
pub struct JournalBuilder {
    journal: Journal,
}

impl JournalBuilder {
    /// Start building a journal for a work package
    pub fn work_package(id: Id, version: JournalVersion, user_id: Id) -> Self {
        Self {
            journal: Journal::new(JournalType::WorkPackage, id, version, user_id),
        }
    }

    /// Start building a journal for a project
    pub fn project(id: Id, version: JournalVersion, user_id: Id) -> Self {
        Self {
            journal: Journal::new(JournalType::Project, id, version, user_id),
        }
    }

    /// Add notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.journal.notes = Some(notes.into());
        self
    }

    /// Set activity ID
    pub fn activity(mut self, activity_id: Id) -> Self {
        self.journal.activity_id = Some(activity_id);
        self
    }

    /// Set cause
    pub fn cause(mut self, cause_type: CauseType) -> Self {
        self.journal.cause.cause_type = cause_type;
        self
    }

    /// Set cause context
    pub fn cause_context(mut self, context: impl Into<String>) -> Self {
        self.journal.cause.context = Some(context.into());
        self
    }

    /// Build the journal
    pub fn build(self) -> Journal {
        self.journal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_creation() {
        let journal = Journal::initial(JournalType::WorkPackage, 1, 10);
        assert!(journal.is_initial());
        assert_eq!(journal.journable_id, 1);
        assert_eq!(journal.user_id, 10);
        assert_eq!(journal.version.0, 1);
    }

    #[test]
    fn test_journal_with_notes() {
        let journal = Journal::initial(JournalType::WorkPackage, 1, 10)
            .with_notes("Initial creation");
        assert!(journal.has_notes());
        assert_eq!(journal.notes, Some("Initial creation".to_string()));
    }

    #[test]
    fn test_journal_builder() {
        let journal = JournalBuilder::work_package(1, 2.into(), 10)
            .notes("Updated status")
            .cause(CauseType::Api)
            .build();

        assert_eq!(journal.journable_type, JournalType::WorkPackage);
        assert_eq!(journal.version.0, 2);
        assert_eq!(journal.cause.cause_type, CauseType::Api);
    }

    #[test]
    fn test_journal_type_conversion() {
        assert_eq!(JournalType::WorkPackage.as_str(), "WorkPackage");
        assert_eq!(JournalType::from_str("WorkPackage"), Some(JournalType::WorkPackage));
        assert_eq!(JournalType::from_str("Unknown"), None);
    }

    #[test]
    fn test_journal_version() {
        let v1 = JournalVersion::initial();
        assert_eq!(v1.0, 1);

        let v2 = v1.next();
        assert_eq!(v2.0, 2);
    }
}
