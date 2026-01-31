//! Journal Service
//!
//! Provides functionality for creating, storing, and querying journal entries.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use thiserror::Error;

use crate::journal::{CauseType, Journal, JournalType, JournalVersion};
use crate::journal_data::{JournalData, JournalDiff, JournalDetails};

/// Journal service errors
#[derive(Debug, Error)]
pub enum JournalError {
    #[error("Journal not found: {0}")]
    NotFound(Id),
    #[error("Version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: i32, actual: i32 },
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type JournalResult<T> = Result<T, JournalError>;

/// Event emitted when a journal is created
#[derive(Debug, Clone)]
pub struct JournalEvent {
    /// The journal that was created
    pub journal: Journal,
    /// The data at this point in time
    pub data: JournalData,
    /// The diff from the previous version (if not initial)
    pub diff: Option<JournalDiff>,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// Journal store trait for persistence
#[async_trait]
pub trait JournalStore: Send + Sync {
    /// Create a new journal entry
    async fn create(&self, journal: &Journal, data: &JournalData) -> JournalResult<Id>;

    /// Get a journal by ID
    async fn get(&self, id: Id) -> JournalResult<Option<(Journal, JournalData)>>;

    /// Get all journals for an entity
    async fn get_for_entity(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<Vec<Journal>>;

    /// Get the latest journal for an entity
    async fn get_latest(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<Option<(Journal, JournalData)>>;

    /// Get the current version for an entity
    async fn get_current_version(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<JournalVersion>;

    /// Get journal data by ID
    async fn get_data(&self, data_id: Id) -> JournalResult<Option<JournalData>>;

    /// Delete journals for an entity
    async fn delete_for_entity(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<usize>;
}

/// Journal service for managing journals
pub struct JournalService {
    store: Arc<dyn JournalStore>,
    event_handlers: Vec<Box<dyn Fn(&JournalEvent) + Send + Sync>>,
}

impl JournalService {
    /// Create a new journal service
    pub fn new(store: Arc<dyn JournalStore>) -> Self {
        Self {
            store,
            event_handlers: Vec::new(),
        }
    }

    /// Register an event handler
    pub fn on_journal_created<F>(&mut self, handler: F)
    where
        F: Fn(&JournalEvent) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    /// Record a journal entry for an entity creation
    pub async fn record_creation(
        &self,
        journable_type: JournalType,
        journable_id: Id,
        user_id: Id,
        data: JournalData,
        notes: Option<String>,
    ) -> JournalResult<Journal> {
        let mut journal = Journal::initial(journable_type, journable_id, user_id);

        if let Some(n) = notes {
            journal = journal.with_notes(n);
        }

        let id = self.store.create(&journal, &data).await?;
        journal.id = Some(id);

        // Emit event
        let event = JournalEvent {
            journal: journal.clone(),
            data,
            diff: None,
            timestamp: Utc::now(),
        };
        self.emit_event(&event);

        Ok(journal)
    }

    /// Record a journal entry for an entity update
    pub async fn record_update(
        &self,
        journable_type: JournalType,
        journable_id: Id,
        user_id: Id,
        new_data: JournalData,
        notes: Option<String>,
        cause: CauseType,
    ) -> JournalResult<Option<Journal>> {
        // Get the previous version
        let (prev_journal, prev_data) = match self.store.get_latest(journable_type, journable_id).await? {
            Some(data) => data,
            None => {
                // No previous version, treat as creation
                return self
                    .record_creation(journable_type, journable_id, user_id, new_data, notes)
                    .await
                    .map(Some);
            }
        };

        // Compute diff
        let diff = JournalDiff::compute(&prev_data, &new_data);

        // Only create a journal if there are changes or notes
        if diff.is_empty() && notes.is_none() {
            return Ok(None);
        }

        // Create new journal
        let new_version = prev_journal.version.next();
        let mut journal = Journal::new(journable_type, journable_id, new_version, user_id)
            .with_cause(cause, None);

        if let Some(n) = notes {
            journal = journal.with_notes(n);
        }

        let id = self.store.create(&journal, &new_data).await?;
        journal.id = Some(id);

        // Emit event
        let event = JournalEvent {
            journal: journal.clone(),
            data: new_data,
            diff: Some(diff),
            timestamp: Utc::now(),
        };
        self.emit_event(&event);

        Ok(Some(journal))
    }

    /// Get all journals for an entity
    pub async fn get_history(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<Vec<Journal>> {
        self.store.get_for_entity(journable_type, journable_id).await
    }

    /// Get a specific journal with its data
    pub async fn get_journal(&self, id: Id) -> JournalResult<Option<(Journal, JournalData)>> {
        self.store.get(id).await
    }

    /// Get the diff between two versions
    pub async fn get_diff(
        &self,
        journable_type: JournalType,
        journable_id: Id,
        from_version: JournalVersion,
        to_version: JournalVersion,
    ) -> JournalResult<JournalDiff> {
        let journals = self.store.get_for_entity(journable_type, journable_id).await?;

        let from_journal = journals
            .iter()
            .find(|j| j.version == from_version)
            .ok_or_else(|| JournalError::NotFound(from_version.0 as Id))?;

        let to_journal = journals
            .iter()
            .find(|j| j.version == to_version)
            .ok_or_else(|| JournalError::NotFound(to_version.0 as Id))?;

        let from_data = match from_journal.data_id {
            Some(id) => self.store.get_data(id).await?,
            None => None,
        }
        .ok_or_else(|| JournalError::InvalidData("Missing journal data".to_string()))?;

        let to_data = match to_journal.data_id {
            Some(id) => self.store.get_data(id).await?,
            None => None,
        }
        .ok_or_else(|| JournalError::InvalidData("Missing journal data".to_string()))?;

        Ok(JournalDiff::compute(&from_data, &to_data))
    }

    /// Delete all journals for an entity
    pub async fn delete_history(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<usize> {
        self.store.delete_for_entity(journable_type, journable_id).await
    }

    /// Emit a journal event to all handlers
    fn emit_event(&self, event: &JournalEvent) {
        for handler in &self.event_handlers {
            handler(event);
        }
    }
}

/// In-memory journal store for testing
pub struct MemoryJournalStore {
    journals: std::sync::RwLock<Vec<(Journal, JournalData)>>,
    next_id: std::sync::atomic::AtomicI64,
}

impl Default for MemoryJournalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryJournalStore {
    pub fn new() -> Self {
        Self {
            journals: std::sync::RwLock::new(Vec::new()),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }
}

#[async_trait]
impl JournalStore for MemoryJournalStore {
    async fn create(&self, journal: &Journal, data: &JournalData) -> JournalResult<Id> {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut journal = journal.clone();
        journal.id = Some(id);
        journal.data_id = Some(id);

        let mut journals = self.journals.write().map_err(|e| JournalError::Database(e.to_string()))?;
        journals.push((journal, data.clone()));

        Ok(id)
    }

    async fn get(&self, id: Id) -> JournalResult<Option<(Journal, JournalData)>> {
        let journals = self.journals.read().map_err(|e| JournalError::Database(e.to_string()))?;
        Ok(journals.iter().find(|(j, _)| j.id == Some(id)).cloned())
    }

    async fn get_for_entity(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<Vec<Journal>> {
        let journals = self.journals.read().map_err(|e| JournalError::Database(e.to_string()))?;
        Ok(journals
            .iter()
            .filter(|(j, _)| j.journable_type == journable_type && j.journable_id == journable_id)
            .map(|(j, _)| j.clone())
            .collect())
    }

    async fn get_latest(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<Option<(Journal, JournalData)>> {
        let journals = self.journals.read().map_err(|e| JournalError::Database(e.to_string()))?;
        Ok(journals
            .iter()
            .filter(|(j, _)| j.journable_type == journable_type && j.journable_id == journable_id)
            .max_by_key(|(j, _)| j.version.0)
            .cloned())
    }

    async fn get_current_version(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<JournalVersion> {
        let journals = self.journals.read().map_err(|e| JournalError::Database(e.to_string()))?;
        Ok(journals
            .iter()
            .filter(|(j, _)| j.journable_type == journable_type && j.journable_id == journable_id)
            .map(|(j, _)| j.version)
            .max()
            .unwrap_or(JournalVersion(0)))
    }

    async fn get_data(&self, data_id: Id) -> JournalResult<Option<JournalData>> {
        let journals = self.journals.read().map_err(|e| JournalError::Database(e.to_string()))?;
        Ok(journals.iter().find(|(j, _)| j.data_id == Some(data_id)).map(|(_, d)| d.clone()))
    }

    async fn delete_for_entity(
        &self,
        journable_type: JournalType,
        journable_id: Id,
    ) -> JournalResult<usize> {
        let mut journals = self.journals.write().map_err(|e| JournalError::Database(e.to_string()))?;
        let before = journals.len();
        journals.retain(|(j, _)| !(j.journable_type == journable_type && j.journable_id == journable_id));
        Ok(before - journals.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_creation() {
        let store = Arc::new(MemoryJournalStore::new());
        let service = JournalService::new(store);

        let data = JournalData::work_package()
            .subject("Test WP")
            .status_id(1)
            .build();

        let journal = service
            .record_creation(JournalType::WorkPackage, 1, 10, data, Some("Created".to_string()))
            .await
            .unwrap();

        assert!(journal.id.is_some());
        assert!(journal.is_initial());
        assert_eq!(journal.notes, Some("Created".to_string()));
    }

    #[tokio::test]
    async fn test_record_update() {
        let store = Arc::new(MemoryJournalStore::new());
        let service = JournalService::new(store);

        // Create initial
        let data1 = JournalData::work_package()
            .subject("Test WP")
            .status_id(1)
            .build();

        service
            .record_creation(JournalType::WorkPackage, 1, 10, data1, None)
            .await
            .unwrap();

        // Update
        let data2 = JournalData::work_package()
            .subject("Updated WP")
            .status_id(2)
            .build();

        let journal = service
            .record_update(
                JournalType::WorkPackage,
                1,
                10,
                data2,
                Some("Updated status".to_string()),
                CauseType::UserAction,
            )
            .await
            .unwrap();

        assert!(journal.is_some());
        let journal = journal.unwrap();
        assert_eq!(journal.version.0, 2);
    }

    #[tokio::test]
    async fn test_no_journal_for_no_changes() {
        let store = Arc::new(MemoryJournalStore::new());
        let service = JournalService::new(store);

        // Create initial
        let data = JournalData::work_package()
            .subject("Test WP")
            .status_id(1)
            .build();

        service
            .record_creation(JournalType::WorkPackage, 1, 10, data.clone(), None)
            .await
            .unwrap();

        // "Update" with same data and no notes
        let journal = service
            .record_update(
                JournalType::WorkPackage,
                1,
                10,
                data,
                None,
                CauseType::UserAction,
            )
            .await
            .unwrap();

        assert!(journal.is_none());
    }

    #[tokio::test]
    async fn test_get_history() {
        let store = Arc::new(MemoryJournalStore::new());
        let service = JournalService::new(store);

        // Create initial
        let data1 = JournalData::work_package().subject("V1").build();
        service
            .record_creation(JournalType::WorkPackage, 1, 10, data1, None)
            .await
            .unwrap();

        // Update twice
        let data2 = JournalData::work_package().subject("V2").build();
        service
            .record_update(JournalType::WorkPackage, 1, 10, data2, None, CauseType::UserAction)
            .await
            .unwrap();

        let data3 = JournalData::work_package().subject("V3").build();
        service
            .record_update(JournalType::WorkPackage, 1, 10, data3, None, CauseType::UserAction)
            .await
            .unwrap();

        let history = service.get_history(JournalType::WorkPackage, 1).await.unwrap();
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_event_handler() {
        let store = Arc::new(MemoryJournalStore::new());
        let mut service = JournalService::new(store);

        let event_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let event_count_clone = event_count.clone();

        service.on_journal_created(move |_| {
            event_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        let data = JournalData::work_package().subject("Test").build();
        service
            .record_creation(JournalType::WorkPackage, 1, 10, data, None)
            .await
            .unwrap();

        assert_eq!(event_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
