//! Relations repository
//!
//! Mirrors: app/models/relation.rb

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{Pagination, PaginatedResult, Repository, RepositoryError};

/// Relation type constants
pub mod relation_type {
    pub const RELATES: &str = "relates";
    pub const PRECEDES: &str = "precedes";
    pub const FOLLOWS: &str = "follows";
    pub const BLOCKS: &str = "blocks";
    pub const BLOCKED: &str = "blocked";
    pub const DUPLICATES: &str = "duplicates";
    pub const DUPLICATED: &str = "duplicated";
    pub const INCLUDES: &str = "includes";
    pub const PARTOF: &str = "partof";
    pub const REQUIRES: &str = "requires";
    pub const REQUIRED: &str = "required";

    /// Relation types that should be stored in reverse order
    pub fn reverse_type(t: &str) -> Option<&'static str> {
        match t {
            FOLLOWS => Some(PRECEDES),
            BLOCKED => Some(BLOCKS),
            DUPLICATED => Some(DUPLICATES),
            PARTOF => Some(INCLUDES),
            REQUIRED => Some(REQUIRES),
            _ => None,
        }
    }

    /// Get the symmetric type for a relation
    pub fn symmetric_type(t: &str) -> &str {
        match t {
            RELATES => RELATES,
            PRECEDES => FOLLOWS,
            FOLLOWS => PRECEDES,
            BLOCKS => BLOCKED,
            BLOCKED => BLOCKS,
            DUPLICATES => DUPLICATED,
            DUPLICATED => DUPLICATES,
            INCLUDES => PARTOF,
            PARTOF => INCLUDES,
            REQUIRES => REQUIRED,
            REQUIRED => REQUIRES,
            _ => t,
        }
    }

    /// Check if this is a valid relation type
    pub fn is_valid(t: &str) -> bool {
        matches!(
            t,
            RELATES | PRECEDES | FOLLOWS | BLOCKS | BLOCKED | DUPLICATES | DUPLICATED | INCLUDES | PARTOF | REQUIRES | REQUIRED
        )
    }
}

/// Maximum lag value
pub const MAX_LAG: i32 = 2000;
/// Minimum lag value
pub const MIN_LAG: i32 = -2000;

/// Relation row from database
#[derive(Debug, Clone, FromRow)]
pub struct RelationRow {
    pub id: i64,
    pub from_id: i64,
    pub to_id: i64,
    pub relation_type: String,
    pub lag: Option<i32>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RelationRow {
    /// Get the reverse type for this relation
    pub fn reverse_type(&self) -> &str {
        relation_type::symmetric_type(&self.relation_type)
    }

    /// Check if this is a follows relation
    pub fn is_follows(&self) -> bool {
        self.relation_type == relation_type::FOLLOWS
    }

    /// Check if this is a precedes relation
    pub fn is_precedes(&self) -> bool {
        self.relation_type == relation_type::PRECEDES
    }

    /// Check if this is a blocks relation
    pub fn is_blocks(&self) -> bool {
        self.relation_type == relation_type::BLOCKS
    }

    /// Check if this is a duplicates relation
    pub fn is_duplicates(&self) -> bool {
        self.relation_type == relation_type::DUPLICATES
    }
}

/// DTO for creating a relation
#[derive(Debug, Clone)]
pub struct CreateRelationDto {
    pub from_id: i64,
    pub to_id: i64,
    pub relation_type: String,
    pub lag: Option<i32>,
    pub description: Option<String>,
}

/// DTO for updating a relation
#[derive(Debug, Clone, Default)]
pub struct UpdateRelationDto {
    pub lag: Option<i32>,
    pub description: Option<Option<String>>,
}

/// Relation repository
pub struct RelationRepository {
    pool: PgPool,
}

impl RelationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find relations for a work package (either from or to)
    pub async fn find_by_work_package(
        &self,
        work_package_id: i64,
        pagination: Pagination,
    ) -> Result<PaginatedResult<RelationRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            WHERE from_id = $1 OR to_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(work_package_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM relations WHERE from_id = $1 OR to_id = $1",
        )
        .bind(work_package_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Find relations where work package is the predecessor (from side)
    pub async fn find_by_predecessor(&self, work_package_id: i64) -> Result<Vec<RelationRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            WHERE from_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(work_package_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find relations where work package is the successor (to side)
    pub async fn find_by_successor(&self, work_package_id: i64) -> Result<Vec<RelationRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            WHERE to_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(work_package_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find follows relations with lag
    pub async fn find_follows_with_lag(&self) -> Result<Vec<RelationRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            WHERE relation_type = $1 AND lag > 0
            ORDER BY created_at DESC
            "#,
        )
        .bind(relation_type::FOLLOWS)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Find relations by type
    pub async fn find_by_type(
        &self,
        relation_type: &str,
        pagination: Pagination,
    ) -> Result<PaginatedResult<RelationRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            WHERE relation_type = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(relation_type)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM relations WHERE relation_type = $1",
        )
        .bind(relation_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResult {
            items: rows,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }

    /// Check if a relation already exists between two work packages
    pub async fn relation_exists(
        &self,
        from_id: i64,
        to_id: i64,
    ) -> Result<bool, RepositoryError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM relations
            WHERE (from_id = $1 AND to_id = $2) OR (from_id = $2 AND to_id = $1)
            "#,
        )
        .bind(from_id)
        .bind(to_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Normalize the relation to canonical form (reverse if needed)
    fn normalize_relation(dto: &CreateRelationDto) -> (i64, i64, String) {
        if let Some(reverse) = relation_type::reverse_type(&dto.relation_type) {
            // Reverse the direction and use canonical type
            (dto.to_id, dto.from_id, reverse.to_string())
        } else {
            (dto.from_id, dto.to_id, dto.relation_type.clone())
        }
    }
}

#[async_trait]
impl Repository<RelationRow, CreateRelationDto, UpdateRelationDto> for RelationRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<RelationRow>, RepositoryError> {
        let row = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<RelationRow>, RepositoryError> {
        let rows = sqlx::query_as::<_, RelationRow>(
            r#"
            SELECT id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            FROM relations
            ORDER BY created_at DESC
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
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM relations")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn exists(&self, id: i64) -> Result<bool, RepositoryError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM relations WHERE id = $1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn create(&self, dto: CreateRelationDto) -> Result<RelationRow, RepositoryError> {
        // Validate relation type
        if !relation_type::is_valid(&dto.relation_type) {
            return Err(RepositoryError::Validation(format!(
                "Invalid relation type: {}",
                dto.relation_type
            )));
        }

        // Validate lag range
        if let Some(lag) = dto.lag {
            if lag < MIN_LAG || lag > MAX_LAG {
                return Err(RepositoryError::Validation(format!(
                    "Lag must be between {} and {}",
                    MIN_LAG, MAX_LAG
                )));
            }
        }

        // Validate from and to are different
        if dto.from_id == dto.to_id {
            return Err(RepositoryError::Validation(
                "A work package cannot be related to itself".to_string(),
            ));
        }

        // Normalize to canonical form
        let (from_id, to_id, relation_type) = Self::normalize_relation(&dto);

        // Check if relation already exists
        if self.relation_exists(from_id, to_id).await? {
            return Err(RepositoryError::Conflict(
                "Relation already exists between these work packages".to_string(),
            ));
        }

        let row = sqlx::query_as::<_, RelationRow>(
            r#"
            INSERT INTO relations (from_id, to_id, relation_type, lag, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            "#,
        )
        .bind(from_id)
        .bind(to_id)
        .bind(&relation_type)
        .bind(dto.lag)
        .bind(&dto.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: i64, dto: UpdateRelationDto) -> Result<RelationRow, RepositoryError> {
        // Verify relation exists
        let existing = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Relation {} not found", id)))?;

        // Validate lag if provided
        if let Some(lag) = dto.lag {
            if lag < MIN_LAG || lag > MAX_LAG {
                return Err(RepositoryError::Validation(format!(
                    "Lag must be between {} and {}",
                    MIN_LAG, MAX_LAG
                )));
            }
        }

        let new_lag = dto.lag.or(existing.lag);
        let new_description = match dto.description {
            Some(d) => d,
            None => existing.description,
        };

        let row = sqlx::query_as::<_, RelationRow>(
            r#"
            UPDATE relations
            SET lag = $2, description = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, from_id, to_id, relation_type, lag, description, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_lag)
        .bind(&new_description)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<(), RepositoryError> {
        // Check if relation exists
        if !self.exists(id).await? {
            return Err(RepositoryError::NotFound(format!(
                "Relation {} not found",
                id
            )));
        }

        sqlx::query("DELETE FROM relations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_type_reverse() {
        assert_eq!(relation_type::reverse_type(relation_type::FOLLOWS), Some(relation_type::PRECEDES));
        assert_eq!(relation_type::reverse_type(relation_type::BLOCKED), Some(relation_type::BLOCKS));
        assert_eq!(relation_type::reverse_type(relation_type::RELATES), None);
    }

    #[test]
    fn test_relation_type_symmetric() {
        assert_eq!(relation_type::symmetric_type(relation_type::FOLLOWS), relation_type::PRECEDES);
        assert_eq!(relation_type::symmetric_type(relation_type::PRECEDES), relation_type::FOLLOWS);
        assert_eq!(relation_type::symmetric_type(relation_type::RELATES), relation_type::RELATES);
    }

    #[test]
    fn test_relation_type_is_valid() {
        assert!(relation_type::is_valid(relation_type::RELATES));
        assert!(relation_type::is_valid(relation_type::FOLLOWS));
        assert!(!relation_type::is_valid("invalid"));
    }
}
