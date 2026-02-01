//! # op-journals
//!
//! Journal/audit logging for OpenProject RS.
//!
//! Mirrors: app/models/journal.rb and related
//!
//! Journals track all changes to journable entities (work packages, etc.)
//! providing a complete audit trail and activity feed.

pub mod journal;
pub mod journal_data;
pub mod journal_service;

pub use journal::{Journal, JournalType, JournalVersion};
pub use journal_data::{JournalData, JournalDiff, JournalDetails};
pub use journal_service::{JournalService, JournalEvent};
