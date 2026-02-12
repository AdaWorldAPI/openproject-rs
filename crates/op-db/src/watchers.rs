//! Watchers repository
//!
//! Mirrors: app/models/watcher.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Watcher row from database
#[derive(Debug, Clone, FromRow)]
pub struct WatcherRow {
    pub id: i64,
    pub watchable_type: String,
    pub watchable_id: i64,
    pub user_id: i64,
}

/// Watcher with user info
#[derive(Debug, Clone)]
pub struct WatcherWithUser {
    pub watcher: WatcherRow,
    pub user_login: String,
    pub user_firstname: String,
    pub user_lastname: String,
    pub user_mail: Option<String>,
}

/// DTO for creating a watcher
#[derive(Debug, Clone)]
pub struct CreateWatcherDto {
    pub watchable_type: String,
    pub watchable_id: i64,
    pub user_id: i64,
}

/// DTO for updating a watcher (no-op, watchers are not updatable)
#[derive(Debug, Clone, Default)]
pub struct UpdateWatcherDto {}

/// Watcher repository
pub struct WatcherRepository {
    pool: PgPool,
}

impl WatcherRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find watchers for a watchable entity
    pub async fn find_by_watchable(
        &self,
        watchable_type: &str,
        watchable_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<WatcherWithUser>, RepositoryError> {
        let rows = sqlx::query_as::<_, WatcherWithUserRow>(
            r#"
            SELECT w.id, w.watchable_type, w.watchable_id, w.user_id,
                   u.login as user_login, u.firstname as user_firstname,
                   u.lastname as user_lastname, u.mail as user_mail
            FROM watchers w
            JOIN users u ON u.id = w.user_id
            WHERE w.watchable_type = $1 AND w.watchable_id = $2
            ORDER BY w.id DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(watchable_type)
        .bind(watchable_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM watchers WHERE watchable_type = $1 AND watchable_id = $2",
        )
        .bind(watchable_type)
        .bind(watchable_id)
        .fetch_one(&self.pool)
        .await?;

        let items = rows.into_iter().map(|r| r.into()).collect();

        Ok(PaginatedResult {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find watchers for a work package
    pub async fn find_by_work_package(
        &self,
        work_package_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<WatcherWithUser>, RepositoryError> {
        self.find_by_watchable("WorkPackage", work_package_id, pagination).await
    }

    /// Find what a user is watching
    pub async fn find_by_user(
        &self,
        user_id: i64,
        watchable_type: Option<&str>,
    ) -> Result<Vec<WatcherRow>, RepositoryError> {
        let rows = if let Some(wt) = watchable_type {
            sqlx::query_as::<_, WatcherRow>(
                r#"
                SELECT id, watchable_type, watchable_id, user_id
                FROM watchers
                WHERE user_id = $1 AND watchable_type = $2
                ORDER BY id DESC
                "#,
            )
            .bind(user_id)
            .bind(wt)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, WatcherRow>(
                r#"
                SELECT id, watchable_type, watchable_id, user_id
                FROM watchers
                WHERE user_id = $1
                ORDER BY id DESC
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    /// Check if user is watching an entity
    pub async fn is_watching(
        &self,
        user_id: i64,
        watchable_type: &str,
        watchable_id: i64,
    ) -> Result<bool, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM watchers
            WHERE user_id = $1 AND watchable_type = $2 AND watchable_id = $3
            "#,
        )
        .bind(user_id)
        .bind(watchable_type)
        .bind(watchable_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Find watcher by user and watchable
    pub async fn find_by_user_and_watchable(
        &self,
        user_id: i64,
        watchable_type: &str,
        watchable_id: i64,
    ) -> Result<Option<WatcherRow>, RepositoryError> {
        let row = sqlx::query_as::<_, WatcherRow>(
            r#"
            SELECT id, watchable_type, watchable_id, user_id
            FROM watchers
            WHERE user_id = $1 AND watchable_type = $2 AND watchable_id = $3
            "#,
        )
        .bind(user_id)
        .bind(watchable_type)
        .bind(watchable_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Delete by user and watchable (convenience method)
    pub async fn delete_by_user_and_watchable(
        &self,
        user_id: i64,
        watchable_type: &str,
        watchable_id: i64,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            "DELETE FROM watchers WHERE user_id = $1 AND watchable_type = $2 AND watchable_id = $3",
        )
        .bind(user_id)
        .bind(watchable_type)
        .bind(watchable_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Validate that user can watch the entity
    async fn validate_user_can_watch(&self, user_id: i64) -> Result<(), RepositoryError> {
        // Check user exists and is active
        let user_status = sqlx::query_scalar::<_, i32>(
            "SELECT status FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match user_status {
            None => Err(RepositoryError::Validation(format!(
                "User {} not found",
                user_id
            ))),
            Some(status) if status == 3 || status == 4 => Err(RepositoryError::Validation(
                "Locked or deleted users cannot watch entities".to_string(),
            )),
            _ => Ok(()),
        }
    }
}

/// Internal row type for joining watchers with users
#[derive(Debug, FromRow)]
struct WatcherWithUserRow {
    id: i64,
    watchable_type: String,
    watchable_id: i64,
    user_id: i64,
    user_login: String,
    user_firstname: String,
    user_lastname: String,
    user_mail: Option<String>,
}

impl From<WatcherWithUserRow> for WatcherWithUser {
    fn from(row: WatcherWithUserRow) -> Self {
        WatcherWithUser {
            watcher: WatcherRow {
                id: row.id,
                watchable_type: row.watchable_type,
                watchable_id: row.watchable_id,
                user_id: row.user_id,
            },
            user_login: row.user_login,
            user_firstname: row.user_firstname,
            user_lastname: row.user_lastname,
            user_mail: row.user_mail,
        }
    }
}

#[async_trait]
impl Repository<WatcherRow, CreateWatcherDto, UpdateWatcherDto> for WatcherRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<WatcherRow>, RepositoryError> {
        let row = sqlx::query_as::<_, WatcherRow>(
            r#"
            SELECT id, watchable_type, watchable_id, user_id
            FROM watchers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<WatcherRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, WatcherRow>(
            r#"
            SELECT id, watchable_type, watchable_id, user_id
            FROM watchers
            ORDER BY id DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM watchers")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM watchers WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateWatcherDto) -> Result<WatcherRow, RepositoryError> {
        // Validate user can watch
        self.validate_user_can_watch(dto.user_id).await?;

        // Check if already watching
        if self
            .is_watching(dto.user_id, &dto.watchable_type, dto.watchable_id)
            .await?
        {
            return Err(RepositoryError::Conflict(
                "User is already watching this entity".to_string(),
            ));
        }

        let row = sqlx::query_as::<_, WatcherRow>(
            r#"
            INSERT INTO watchers (watchable_type, watchable_id, user_id)
            VALUES ($1, $2, $3)
            RETURNING id, watchable_type, watchable_id, user_id
            "#,
        )
        .bind(&dto.watchable_type)
        .bind(dto.watchable_id)
        .bind(dto.user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, _dto: UpdateWatcherDto) -> Result<WatcherRow, RepositoryError> {
        // Watchers cannot be updated, just return the existing one
        self.find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Watcher {} not found", id)))
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if watcher exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Watcher {} not found",
                id
            )));
        }

        sqlx::query("DELETE FROM watchers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

impl WatcherWithUser {
    /// Get the watcher row
    pub fn watcher(&self) -> &WatcherRow {
        &self.watcher
    }

    /// Get full user name
    pub fn user_name(&self) -> String {
        format!("{} {}", self.user_firstname, self.user_lastname)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_with_user_name() {
        let watcher_with_user = WatcherWithUser {
            watcher: WatcherRow {
                id: 1,
                watchable_type: "WorkPackage".to_string(),
                watchable_id: 42,
                user_id: 2,
            },
            user_login: "jdoe".to_string(),
            user_firstname: "John".to_string(),
            user_lastname: "Doe".to_string(),
            user_mail: Some("john@example.com".to_string()),
        };

        assert_eq!(watcher_with_user.user_name(), "John Doe");
    }
}
