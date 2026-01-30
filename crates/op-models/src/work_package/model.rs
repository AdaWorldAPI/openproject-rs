//! Work Package model

use op_core::traits::Id;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Work Package entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkPackage {
    pub id: Option<Id>,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: Id,
    pub type_id: Id,
    pub status_id: Id,
    pub priority_id: Option<Id>,
    pub author_id: Id,
    pub assigned_to_id: Option<Id>,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub done_ratio: i32,
    pub estimated_hours: Option<f64>,
    pub lock_version: i32,
}

impl WorkPackage {
    pub fn new(subject: impl Into<String>, project_id: Id, type_id: Id) -> Self {
        Self {
            subject: subject.into(),
            project_id,
            type_id,
            ..Default::default()
        }
    }
}
