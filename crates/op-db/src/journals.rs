//! Journal repository for change history tracking
//!
//! Maps to Rails: app/models/journal.rb
//! Table: journals (polymorphic journable_type, journable_id)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{FromRow, PgPool, Row};

use crate::repository::{Pagination, PaginatedResult, Repository, RepositoryError, RepositoryResult};

/// Valid cause types for journals
pub mod cause_type {
    pub const DEFAULT_ATTRIBUTE_WRITTEN: &str = "default_attribute_written";
    pub const PROGRESS_MODE_CHANGED_TO_STATUS_BASED: &str = "progress_mode_changed_to_status_based";
    pub const STATUS_CHANGED: &str = "status_changed";
    pub const SYSTEM_UPDATE: &str = "system_update";
    pub const TOTAL_PERCENT_COMPLETE_MODE_CHANGED: &str = "total_percent_complete_mode_changed_to_work_weighted_average";
    pub const WORK_PACKAGE_CHILDREN_CHANGED_TIMES: &str = "work_package_children_changed_times";
    pub const WORK_PACKAGE_PARENT_CHANGED_TIMES: &str = "work_package_parent_changed_times";
    pub const WORK_PACKAGE_PREDECESSOR_CHANGED_TIMES: &str = "work_package_predecessor_changed_times";
    pub const WORK_PACKAGE_RELATED_CHANGED_TIMES: &str = "work_package_related_changed_times";
    pub const WORK_PACKAGE_DUPLICATE_CLOSED: &str = "work_package_duplicate_closed";
    pub const WORKING_DAYS_CHANGED: &str = "working_days_changed";

    pub fn is_valid(t: &str) -> bool {
        matches!(
            t,
            DEFAULT_ATTRIBUTE_WRITTEN
                | PROGRESS_MODE_CHANGED_TO_STATUS_BASED
                | STATUS_CHANGED
                | SYSTEM_UPDATE
                | TOTAL_PERCENT_COMPLETE_MODE_CHANGED
                | WORK_PACKAGE_CHILDREN_CHANGED_TIMES
                | WORK_PACKAGE_PARENT_CHANGED_TIMES
                | WORK_PACKAGE_PREDECESSOR_CHANGED_TIMES
                | WORK_PACKAGE_RELATED_CHANGED_TIMES
                | WORK_PACKAGE_DUPLICATE_CLOSED
                | WORKING_DAYS_CHANGED
        )
    }
}

/// Common journable types
pub mod journable_type {
    pub const WORK_PACKAGE: &str = "WorkPackage";
    pub const WIKI_PAGE: &str = "WikiPage";
    pub const MEETING: &str = "Meeting";
    pub const PROJECT: &str = "Project";
    pub const NEWS: &str = "News";
    pub const MESSAGE: &str = "Message";
}

/// Journal row from database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct JournalRow {
    pub id: i64,
    pub journable_type: String,
    pub journable_id: i64,
    pub user_id: i64,
    pub notes: Option<String>,
    pub version: i32,
    pub data_type: String,
    pub data_id: i64,
    pub cause: JsonValue,
    pub restricted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JournalRow {
    /// Returns whether this is the initial journal version
    pub fn is_initial(&self) -> bool {
        self.version < 2
    }

    /// Returns the anchor number for HTML output
    pub fn anchor(&self) -> i32 {
        self.version - 1
    }

    /// Returns whether the journal has a cause
    pub fn has_cause(&self) -> bool {
        if let Some(cause_type) = self.cause.get("type") {
            if let Some(s) = cause_type.as_str() {
                return !s.is_empty();
            }
        }
        false
    }

    /// Returns the cause type if present
    pub fn cause_type(&self) -> Option<&str> {
        self.cause.get("type").and_then(|v: &JsonValue| v.as_str())
    }

    /// Returns whether the journal is internal/restricted
    pub fn is_internal(&self) -> bool {
        self.restricted
    }
}

/// Work package journal data row
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkPackageJournalRow {
    pub id: i64,
    pub type_id: i64,
    pub project_id: i64,
    pub subject: String,
    pub description: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub category_id: Option<i64>,
    pub status_id: i64,
    pub assigned_to_id: Option<i64>,
    pub priority_id: i64,
    pub version_id: Option<i64>,
    pub author_id: i64,
    pub done_ratio: Option<i32>,
    pub estimated_hours: Option<f64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub parent_id: Option<i64>,
    pub responsible_id: Option<i64>,
    pub derived_estimated_hours: Option<f64>,
    pub schedule_manually: Option<bool>,
    pub duration: Option<i32>,
    pub ignore_non_working_days: bool,
    pub derived_remaining_hours: Option<f64>,
    pub derived_done_ratio: Option<i32>,
}

/// Journal with associated user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalWithUser {
    pub journal: JournalRow,
    pub user_login: String,
    pub user_firstname: String,
    pub user_lastname: String,
}

/// Journal with work package data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalWithWorkPackageData {
    pub journal: JournalRow,
    pub data: Option<WorkPackageJournalRow>,
}

/// DTO for creating a journal
#[derive(Debug, Clone)]
pub struct CreateJournalDto {
    pub journable_type: String,
    pub journable_id: i64,
    pub user_id: i64,
    pub notes: Option<String>,
    pub version: i32,
    pub data_type: String,
    pub data_id: i64,
    pub cause: Option<JsonValue>,
    pub restricted: Option<bool>,
}

/// DTO for updating a journal (mainly notes)
#[derive(Debug, Clone, Default)]
pub struct UpdateJournalDto {
    pub notes: Option<Option<String>>,
}

/// Journal repository
#[derive(Clone)]
pub struct JournalRepository {
    pool: PgPool,
}

impl JournalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find journals by journable (work package, wiki page, etc.)
    pub async fn find_by_journable(
        &self,
        journable_type: &str,
        journable_id: i64,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<JournalRow>> {
        let items = sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            WHERE journable_type = $1 AND journable_id = $2
            ORDER BY version ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(journable_type)
        .bind(journable_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journals WHERE journable_type = $1 AND journable_id = $2",
        )
        .bind(journable_type)
        .bind(journable_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find journals for a work package
    pub async fn find_by_work_package(
        &self,
        work_package_id: i64,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<JournalRow>> {
        self.find_by_journable(journable_type::WORK_PACKAGE, work_package_id, pagination)
            .await
    }

    /// Find journals for a wiki page
    pub async fn find_by_wiki_page(
        &self,
        wiki_page_id: i64,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<JournalRow>> {
        self.find_by_journable(journable_type::WIKI_PAGE, wiki_page_id, pagination)
            .await
    }

    /// Find journals by user
    pub async fn find_by_user(
        &self,
        user_id: i64,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<JournalRow>> {
        let items = sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journals WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find changing journals (version > 1)
    pub async fn find_changing(
        &self,
        journable_type: &str,
        journable_id: i64,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<JournalRow>> {
        let items = sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            WHERE journable_type = $1 AND journable_id = $2 AND version > 1
            ORDER BY version ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(journable_type)
        .bind(journable_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journals WHERE journable_type = $1 AND journable_id = $2 AND version > 1",
        )
        .bind(journable_type)
        .bind(journable_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find journal with user info
    pub async fn find_by_id_with_user(&self, id: i64) -> RepositoryResult<Option<JournalWithUser>> {
        let row = sqlx::query(
            r#"
            SELECT j.id, j.journable_type, j.journable_id, j.user_id, j.notes, j.version,
                   j.data_type, j.data_id, j.cause, j.restricted, j.created_at, j.updated_at,
                   u.login as user_login, u.firstname as user_firstname, u.lastname as user_lastname
            FROM journals j
            JOIN users u ON u.id = j.user_id
            WHERE j.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| JournalWithUser {
            journal: JournalRow {
                id: r.get("id"),
                journable_type: r.get("journable_type"),
                journable_id: r.get("journable_id"),
                user_id: r.get("user_id"),
                notes: r.get("notes"),
                version: r.get("version"),
                data_type: r.get("data_type"),
                data_id: r.get("data_id"),
                cause: r.get("cause"),
                restricted: r.get("restricted"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            },
            user_login: r.get("user_login"),
            user_firstname: r.get("user_firstname"),
            user_lastname: r.get("user_lastname"),
        }))
    }

    /// Get next version number for a journable
    pub async fn next_version(&self, journable_type: &str, journable_id: i64) -> RepositoryResult<i32> {
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version) FROM journals WHERE journable_type = $1 AND journable_id = $2",
        )
        .bind(journable_type)
        .bind(journable_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(max_version.unwrap_or(0) + 1)
    }

    /// Find predecessor journal
    pub async fn find_predecessor(
        &self,
        journable_type: &str,
        journable_id: i64,
        version: i32,
    ) -> RepositoryResult<Option<JournalRow>> {
        if version < 2 {
            return Ok(None);
        }

        Ok(sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            WHERE journable_type = $1 AND journable_id = $2 AND version < $3
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(journable_type)
        .bind(journable_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Find successor journal
    pub async fn find_successor(
        &self,
        journable_type: &str,
        journable_id: i64,
        version: i32,
    ) -> RepositoryResult<Option<JournalRow>> {
        Ok(sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            WHERE journable_type = $1 AND journable_id = $2 AND version > $3
            ORDER BY version ASC
            LIMIT 1
            "#,
        )
        .bind(journable_type)
        .bind(journable_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Find journals with work package data
    pub async fn find_work_package_journals_with_data(
        &self,
        work_package_id: i64,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<JournalWithWorkPackageData>> {
        let rows = sqlx::query(
            r#"
            SELECT j.id, j.journable_type, j.journable_id, j.user_id, j.notes, j.version,
                   j.data_type, j.data_id, j.cause, j.restricted, j.created_at, j.updated_at,
                   wpj.id as wpj_id, wpj.type_id, wpj.project_id, wpj.subject, wpj.description,
                   wpj.due_date, wpj.category_id, wpj.status_id, wpj.assigned_to_id,
                   wpj.priority_id, wpj.version_id, wpj.author_id, wpj.done_ratio,
                   wpj.estimated_hours, wpj.start_date, wpj.parent_id, wpj.responsible_id,
                   wpj.derived_estimated_hours, wpj.schedule_manually, wpj.duration,
                   wpj.ignore_non_working_days, wpj.derived_remaining_hours, wpj.derived_done_ratio
            FROM journals j
            LEFT JOIN work_package_journals wpj ON wpj.id = j.data_id AND j.data_type = 'Journal::WorkPackageJournal'
            WHERE j.journable_type = 'WorkPackage' AND j.journable_id = $1
            ORDER BY j.version ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(work_package_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<JournalWithWorkPackageData> = rows
            .into_iter()
            .map(|r| {
                let journal = JournalRow {
                    id: r.get("id"),
                    journable_type: r.get("journable_type"),
                    journable_id: r.get("journable_id"),
                    user_id: r.get("user_id"),
                    notes: r.get("notes"),
                    version: r.get("version"),
                    data_type: r.get("data_type"),
                    data_id: r.get("data_id"),
                    cause: r.get("cause"),
                    restricted: r.get("restricted"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                };

                let data: Option<i64> = r.get("wpj_id");
                let wp_data = data.map(|wpj_id| WorkPackageJournalRow {
                    id: wpj_id,
                    type_id: r.get("type_id"),
                    project_id: r.get("project_id"),
                    subject: r.get("subject"),
                    description: r.get("description"),
                    due_date: r.get("due_date"),
                    category_id: r.get("category_id"),
                    status_id: r.get("status_id"),
                    assigned_to_id: r.get("assigned_to_id"),
                    priority_id: r.get("priority_id"),
                    version_id: r.get("version_id"),
                    author_id: r.get("author_id"),
                    done_ratio: r.get("done_ratio"),
                    estimated_hours: r.get("estimated_hours"),
                    start_date: r.get("start_date"),
                    parent_id: r.get("parent_id"),
                    responsible_id: r.get("responsible_id"),
                    derived_estimated_hours: r.get("derived_estimated_hours"),
                    schedule_manually: r.get("schedule_manually"),
                    duration: r.get("duration"),
                    ignore_non_working_days: r.get("ignore_non_working_days"),
                    derived_remaining_hours: r.get("derived_remaining_hours"),
                    derived_done_ratio: r.get("derived_done_ratio"),
                });

                JournalWithWorkPackageData {
                    journal,
                    data: wp_data,
                }
            })
            .collect();

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journals WHERE journable_type = 'WorkPackage' AND journable_id = $1",
        )
        .bind(work_package_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }
}

#[async_trait]
impl Repository<JournalRow, CreateJournalDto, UpdateJournalDto> for JournalRepository {
    async fn find_by_id(&self, id: i64) -> RepositoryResult<Option<JournalRow>> {
        Ok(sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<JournalRow>> {
        Ok(sqlx::query_as::<_, JournalRow>(
            r#"
            SELECT id, journable_type, journable_id, user_id, notes, version,
                   data_type, data_id, cause, restricted, created_at, updated_at
            FROM journals
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?)
    }

    async fn create(&self, dto: CreateJournalDto) -> RepositoryResult<JournalRow> {
        let cause = dto.cause.unwrap_or_else(|| serde_json::json!({}));
        let restricted = dto.restricted.unwrap_or(false);

        sqlx::query_as::<_, JournalRow>(
            r#"
            INSERT INTO journals (journable_type, journable_id, user_id, notes, version,
                                  data_type, data_id, cause, restricted, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING id, journable_type, journable_id, user_id, notes, version,
                      data_type, data_id, cause, restricted, created_at, updated_at
            "#,
        )
        .bind(&dto.journable_type)
        .bind(dto.journable_id)
        .bind(dto.user_id)
        .bind(&dto.notes)
        .bind(dto.version)
        .bind(&dto.data_type)
        .bind(dto.data_id)
        .bind(&cause)
        .bind(restricted)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique constraint") {
                RepositoryError::Conflict("Journal version already exists".to_string())
            } else {
                RepositoryError::from(e)
            }
        })
    }

    async fn update(&self, id: i64, dto: UpdateJournalDto) -> RepositoryResult<JournalRow> {
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Journal with id {} not found", id)))?;

        let notes = dto.notes.unwrap_or(existing.notes);

        Ok(sqlx::query_as::<_, JournalRow>(
            r#"
            UPDATE journals
            SET notes = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, journable_type, journable_id, user_id, notes, version,
                      data_type, data_id, cause, restricted, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&notes)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM journals WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Journal with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn count(&self) -> RepositoryResult<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM journals")
            .fetch_one(&self.pool)
            .await?)
    }

    async fn exists(&self, id: i64) -> RepositoryResult<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journals WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cause_type_validation() {
        assert!(cause_type::is_valid(cause_type::STATUS_CHANGED));
        assert!(cause_type::is_valid(cause_type::SYSTEM_UPDATE));
        assert!(!cause_type::is_valid("invalid_cause"));
    }

    #[test]
    fn test_journal_row_methods() {
        let journal = JournalRow {
            id: 1,
            journable_type: "WorkPackage".to_string(),
            journable_id: 100,
            user_id: 1,
            notes: Some("Test note".to_string()),
            version: 1,
            data_type: "Journal::WorkPackageJournal".to_string(),
            data_id: 1,
            cause: serde_json::json!({"type": "status_changed"}),
            restricted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(journal.is_initial());
        assert_eq!(journal.anchor(), 0);
        assert!(journal.has_cause());
        assert_eq!(journal.cause_type(), Some("status_changed"));
        assert!(!journal.is_internal());
    }
}
