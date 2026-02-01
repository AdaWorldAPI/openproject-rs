//! User repository
//!
//! Database operations for users.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use op_core::traits::Id;
use sqlx::{FromRow, PgPool};

use crate::repository::{Pagination, PaginatedResult, Repository, RepositoryError, RepositoryResult};

/// User database entity
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub admin: bool,
    pub status: i32,
    pub language: Option<String>,
    pub hashed_password: Option<String>,
    pub salt: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_on: Option<DateTime<Utc>>,
}

impl UserRow {
    /// Check if the user is active
    pub fn is_active(&self) -> bool {
        self.status == 1
    }

    /// Check if the user is locked
    pub fn is_locked(&self) -> bool {
        self.status == 3
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.firstname, self.lastname)
    }
}

/// DTO for creating a user
#[derive(Debug, Clone)]
pub struct CreateUserDto {
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub admin: bool,
    pub status: i32,
    pub language: Option<String>,
    pub hashed_password: Option<String>,
    pub salt: Option<String>,
}

/// DTO for updating a user
#[derive(Debug, Clone, Default)]
pub struct UpdateUserDto {
    pub login: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
    pub admin: Option<bool>,
    pub status: Option<i32>,
    pub language: Option<String>,
    pub hashed_password: Option<String>,
    pub salt: Option<String>,
}

/// User status constants
pub mod status {
    pub const REGISTERED: i32 = 2;
    pub const ACTIVE: i32 = 1;
    pub const LOCKED: i32 = 3;
    pub const INVITED: i32 = 4;
}

/// User repository implementation
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find a user by login
    pub async fn find_by_login(&self, login: &str) -> RepositoryResult<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, login, firstname, lastname, mail, admin, status,
                   language, hashed_password, salt, created_at, updated_at, last_login_on
            FROM users
            WHERE login = $1
            "#,
        )
        .bind(login)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find a user by email
    pub async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, login, firstname, lastname, mail, admin, status,
                   language, hashed_password, salt, created_at, updated_at, last_login_on
            FROM users
            WHERE mail = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find active users
    pub async fn find_active(
        &self,
        pagination: Pagination,
    ) -> RepositoryResult<PaginatedResult<UserRow>> {
        let items = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, login, firstname, lastname, mail, admin, status,
                   language, hashed_password, salt, created_at, updated_at, last_login_on
            FROM users
            WHERE status = $1
            ORDER BY login ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status::ACTIVE)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE status = $1")
                .bind(status::ACTIVE)
                .fetch_one(&self.pool)
                .await?;

        Ok(PaginatedResult::new(items, total, pagination))
    }

    /// Find admins
    pub async fn find_admins(&self) -> RepositoryResult<Vec<UserRow>> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, login, firstname, lastname, mail, admin, status,
                   language, hashed_password, salt, created_at, updated_at, last_login_on
            FROM users
            WHERE admin = true AND status = $1
            ORDER BY login ASC
            "#,
        )
        .bind(status::ACTIVE)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Update last login timestamp
    pub async fn update_last_login(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query("UPDATE users SET last_login_on = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update password
    pub async fn update_password(
        &self,
        id: Id,
        hashed_password: &str,
        salt: &str,
    ) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE users SET hashed_password = $1, salt = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(hashed_password)
        .bind(salt)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Lock a user
    pub async fn lock(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query("UPDATE users SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status::LOCKED)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Unlock a user
    pub async fn unlock(&self, id: Id) -> RepositoryResult<()> {
        sqlx::query("UPDATE users SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status::ACTIVE)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Check if login is unique
    pub async fn is_login_unique(&self, login: &str, exclude_id: Option<Id>) -> RepositoryResult<bool> {
        let query = match exclude_id {
            Some(id) => {
                sqlx::query_scalar::<_, bool>(
                    "SELECT NOT EXISTS(SELECT 1 FROM users WHERE login = $1 AND id != $2)",
                )
                .bind(login)
                .bind(id)
            }
            None => sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM users WHERE login = $1)",
            )
            .bind(login),
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }

    /// Check if email is unique
    pub async fn is_email_unique(&self, email: &str, exclude_id: Option<Id>) -> RepositoryResult<bool> {
        let query = match exclude_id {
            Some(id) => {
                sqlx::query_scalar::<_, bool>(
                    "SELECT NOT EXISTS(SELECT 1 FROM users WHERE mail = $1 AND id != $2)",
                )
                .bind(email)
                .bind(id)
            }
            None => {
                sqlx::query_scalar::<_, bool>(
                    "SELECT NOT EXISTS(SELECT 1 FROM users WHERE mail = $1)",
                )
                .bind(email)
            }
        };

        let unique = query.fetch_one(&self.pool).await?;
        Ok(unique)
    }
}

#[async_trait]
impl Repository<UserRow, CreateUserDto, UpdateUserDto> for UserRepository {
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, login, firstname, lastname, mail, admin, status,
                   language, hashed_password, salt, created_at, updated_at, last_login_on
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<UserRow>> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, login, firstname, lastname, mail, admin, status,
                   language, hashed_password, salt, created_at, updated_at, last_login_on
            FROM users
            ORDER BY login ASC
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn create(&self, dto: CreateUserDto) -> RepositoryResult<UserRow> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (
                login, firstname, lastname, mail, admin, status,
                language, hashed_password, salt, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW()
            )
            RETURNING id, login, firstname, lastname, mail, admin, status,
                      language, hashed_password, salt, created_at, updated_at, last_login_on
            "#,
        )
        .bind(&dto.login)
        .bind(&dto.firstname)
        .bind(&dto.lastname)
        .bind(&dto.mail)
        .bind(dto.admin)
        .bind(dto.status)
        .bind(&dto.language)
        .bind(&dto.hashed_password)
        .bind(&dto.salt)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Id, dto: UpdateUserDto) -> RepositoryResult<UserRow> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users SET
                login = COALESCE($1, login),
                firstname = COALESCE($2, firstname),
                lastname = COALESCE($3, lastname),
                mail = COALESCE($4, mail),
                admin = COALESCE($5, admin),
                status = COALESCE($6, status),
                language = COALESCE($7, language),
                hashed_password = COALESCE($8, hashed_password),
                salt = COALESCE($9, salt),
                updated_at = NOW()
            WHERE id = $10
            RETURNING id, login, firstname, lastname, mail, admin, status,
                      language, hashed_password, salt, created_at, updated_at, last_login_on
            "#,
        )
        .bind(&dto.login)
        .bind(&dto.firstname)
        .bind(&dto.lastname)
        .bind(&dto.mail)
        .bind(dto.admin)
        .bind(dto.status)
        .bind(&dto.language)
        .bind(&dto.hashed_password)
        .bind(&dto.salt)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound(format!("User with id {} not found", id)))?;

        Ok(row)
    }

    async fn delete(&self, id: Id) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "User with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn exists(&self, id: Id) -> RepositoryResult<bool> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }
}
