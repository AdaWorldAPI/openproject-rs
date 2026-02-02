//! Time Entry repository
//!
//! Database operations for time entries (time tracking).
//!
//! Mirrors: app/models/time_entry.rb

use async_trait::async_trait;
use chrono::{Datelike, DateTime, NaiveDate, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Pagination, PaginatedResult, Repository, RepositoryError, RepositoryResult};

/// Time entry database entity
#[derive(Debug, Clone, FromRow)]
pub struct TimeEntryRow {
    pub id: i64,
    pub project_id: i64,
    pub user_id: i64,
    pub work_package_id: Option<i64>,
    pub hours: f64,
    pub comments: Option<String>,
    pub activity_id: i64,
    pub spent_on: NaiveDate,
    pub tyear: i32,
    pub tmonth: i32,
    pub tweek: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub overridden_costs: Option<f64>,
    pub costs: Option<f64>,
    pub rate_id: Option<i64>,
    pub logged_by_id: Option<i64>,
}

/// DTO for creating a time entry
#[derive(Debug, Clone)]
pub struct CreateTimeEntryDto {
    pub project_id: i64,
    pub user_id: i64,
    pub work_package_id: Option<i64>,
    pub hours: f64,
    pub comments: Option<String>,
    pub activity_id: i64,
    pub spent_on: NaiveDate,
    pub logged_by_id: Option<i64>,
}

/// DTO for updating a time entry
#[derive(Debug, Clone, Default)]
pub struct UpdateTimeEntryDto {
    pub work_package_id: Option<i64>,
    pub hours: Option<f64>,
    pub comments: Option<String>,
    pub activity_id: Option<i64>,
    pub spent_on: Option<NaiveDate>,
}

/// Time entry repository implementation
pub struct TimeEntryRepository {
    pool: PgPool,
}

impl TimeEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find time entries by project ID
    pub async fn find_by_project(
        &self,
        project_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<TimeEntryRow>> {
        let items = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            SELECT id, project_id, user_id, work_package_id, hours, comments,
                   activity_id, spent_on, tyear, tmonth, tweek,
                   created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            FROM time_entries
            WHERE project_id = $1
            ORDER BY spent_on DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(project_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM time_entries WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find time entries by user ID
    pub async fn find_by_user(
        &self,
        user_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<TimeEntryRow>> {
        let items = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            SELECT id, project_id, user_id, work_package_id, hours, comments,
                   activity_id, spent_on, tyear, tmonth, tweek,
                   created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            FROM time_entries
            WHERE user_id = $1
            ORDER BY spent_on DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM time_entries WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find time entries by work package ID
    pub async fn find_by_work_package(
        &self,
        work_package_id: Id,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<TimeEntryRow>> {
        let items = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            SELECT id, project_id, user_id, work_package_id, hours, comments,
                   activity_id, spent_on, tyear, tmonth, tweek,
                   created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            FROM time_entries
            WHERE work_package_id = $1
            ORDER BY spent_on DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(work_package_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM time_entries WHERE work_package_id = $1",
        )
        .bind(work_package_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find time entries for a date range
    pub async fn find_by_date_range(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        user_id: Option<Id>,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<TimeEntryRow>> {
        let (items, total) = if let Some(uid) = user_id {
            let items = sqlx::query_as::<_, TimeEntryRow>(
                r#"
                SELECT id, project_id, user_id, work_package_id, hours, comments,
                       activity_id, spent_on, tyear, tmonth, tweek,
                       created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
                FROM time_entries
                WHERE spent_on >= $1 AND spent_on <= $2 AND user_id = $3
                ORDER BY spent_on DESC, id DESC
                LIMIT $4 OFFSET $5
                "#,
            )
            .bind(from)
            .bind(to)
            .bind(uid)
            .bind(pagination.limit)
            .bind(pagination.offset)
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM time_entries WHERE spent_on >= $1 AND spent_on <= $2 AND user_id = $3",
            )
            .bind(from)
            .bind(to)
            .bind(uid)
            .fetch_one(&self.pool)
            .await?;

            (items, total)
        } else {
            let items = sqlx::query_as::<_, TimeEntryRow>(
                r#"
                SELECT id, project_id, user_id, work_package_id, hours, comments,
                       activity_id, spent_on, tyear, tmonth, tweek,
                       created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
                FROM time_entries
                WHERE spent_on >= $1 AND spent_on <= $2
                ORDER BY spent_on DESC, id DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(from)
            .bind(to)
            .bind(pagination.limit)
            .bind(pagination.offset)
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM time_entries WHERE spent_on >= $1 AND spent_on <= $2",
            )
            .bind(from)
            .bind(to)
            .fetch_one(&self.pool)
            .await?;

            (items, total)
        };

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Get total hours for a work package
    pub async fn total_hours_for_work_package(&self, work_package_id: Id) -> RepositoryResult<f64> {
        let total = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT SUM(hours) FROM time_entries WHERE work_package_id = $1",
        )
        .bind(work_package_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }

    /// Get total hours for a project
    pub async fn total_hours_for_project(&self, project_id: Id) -> RepositoryResult<f64> {
        let total = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT SUM(hours) FROM time_entries WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }

    /// Get total hours for a user in a date range
    pub async fn total_hours_for_user(
        &self,
        user_id: Id,
        from: NaiveDate,
        to: NaiveDate,
    ) -> RepositoryResult<f64> {
        let total = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT SUM(hours) FROM time_entries WHERE user_id = $1 AND spent_on >= $2 AND spent_on <= $3",
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }
}

#[async_trait]
impl Repository<TimeEntryRow, CreateTimeEntryDto, UpdateTimeEntryDto> for TimeEntryRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<TimeEntryRow>> {
        let row = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            SELECT id, project_id, user_id, work_package_id, hours, comments,
                   activity_id, spent_on, tyear, tmonth, tweek,
                   created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            FROM time_entries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<TimeEntryRow>> {
        let rows = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            SELECT id, project_id, user_id, work_package_id, hours, comments,
                   activity_id, spent_on, tyear, tmonth, tweek,
                   created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            FROM time_entries
            ORDER BY spent_on DESC, id DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM time_entries")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateTimeEntryDto) -> RepositoryResult<TimeEntryRow> {
        // Calculate year, month, week from spent_on
        let tyear = dto.spent_on.format("%Y").to_string().parse::<i32>().unwrap_or(0);
        let tmonth = dto.spent_on.format("%m").to_string().parse::<i32>().unwrap_or(0);
        let tweek = dto.spent_on.iso_week().week() as i32;

        let row = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            INSERT INTO time_entries (
                project_id, user_id, work_package_id, hours, comments,
                activity_id, spent_on, tyear, tmonth, tweek,
                logged_by_id, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW()
            )
            RETURNING id, project_id, user_id, work_package_id, hours, comments,
                      activity_id, spent_on, tyear, tmonth, tweek,
                      created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            "#,
        )
        .bind(dto.project_id)
        .bind(dto.user_id)
        .bind(dto.work_package_id)
        .bind(dto.hours)
        .bind(&dto.comments)
        .bind(dto.activity_id)
        .bind(dto.spent_on)
        .bind(tyear)
        .bind(tmonth)
        .bind(tweek)
        .bind(dto.logged_by_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateTimeEntryDto) -> RepositoryResult<TimeEntryRow> {
        // If spent_on changes, we need to recalculate year/month/week
        let (tyear, tmonth, tweek) = if let Some(spent_on) = dto.spent_on {
            let y = spent_on.format("%Y").to_string().parse::<i32>().unwrap_or(0);
            let m = spent_on.format("%m").to_string().parse::<i32>().unwrap_or(0);
            let w = spent_on.iso_week().week() as i32;
            (Some(y), Some(m), Some(w))
        } else {
            (None, None, None)
        };

        let row = sqlx::query_as::<_, TimeEntryRow>(
            r#"
            UPDATE time_entries SET
                work_package_id = COALESCE($1, work_package_id),
                hours = COALESCE($2, hours),
                comments = COALESCE($3, comments),
                activity_id = COALESCE($4, activity_id),
                spent_on = COALESCE($5, spent_on),
                tyear = COALESCE($6, tyear),
                tmonth = COALESCE($7, tmonth),
                tweek = COALESCE($8, tweek),
                updated_at = NOW()
            WHERE id = $9
            RETURNING id, project_id, user_id, work_package_id, hours, comments,
                      activity_id, spent_on, tyear, tmonth, tweek,
                      created_at, updated_at, overridden_costs, costs, rate_id, logged_by_id
            "#,
        )
        .bind(dto.work_package_id)
        .bind(dto.hours)
        .bind(&dto.comments)
        .bind(dto.activity_id)
        .bind(dto.spent_on)
        .bind(tyear)
        .bind(tmonth)
        .bind(tweek)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("Time entry with id {} not found", id)))?;

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM time_entries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Time entry with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM time_entries WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}
