//! Repository traits and base implementations
//!
//! Provides generic CRUD operations for database entities.

use async_trait::async_trait;
use op_core::traits::Id;
use sqlx::PgPool;

/// Error type for repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Base repository trait for CRUD operations
#[async_trait]
pub trait Repository<T, CreateDto, UpdateDto>: Send + Sync {
    /// Find an entity by ID
    async fn find_by_id(&self, id: Id) -> RepositoryResult<Option<T>>;

    /// Find all entities with pagination
    async fn find_all(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<T>>;

    /// Count all entities
    async fn count(&self) -> RepositoryResult<i64>;

    /// Create a new entity
    async fn create(&self, dto: CreateDto) -> RepositoryResult<T>;

    /// Update an existing entity
    async fn update(&self, id: Id, dto: UpdateDto) -> RepositoryResult<T>;

    /// Delete an entity by ID
    async fn delete(&self, id: Id) -> RepositoryResult<()>;

    /// Check if an entity exists
    async fn exists(&self, id: Id) -> RepositoryResult<bool>;
}

/// Repository context with database pool
pub struct RepositoryContext {
    pool: PgPool,
}

impl RepositoryContext {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl Clone for RepositoryContext {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

/// Trait for entities that can be converted to/from database rows
pub trait FromRow: Sized {
    type Row;
    fn from_row(row: Self::Row) -> RepositoryResult<Self>;
}

/// Pagination parameters for queries
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

impl Pagination {
    pub fn new(limit: i64, offset: i64) -> Self {
        Self { limit, offset }
    }

    pub fn page(page: i64, per_page: i64) -> Self {
        Self {
            limit: per_page,
            offset: (page - 1) * per_page,
        }
    }
}

/// Query result with pagination metadata
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: Pagination) -> Self {
        Self {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        }
    }

    pub fn page(&self) -> i64 {
        if self.limit == 0 {
            1
        } else {
            (self.offset / self.limit) + 1
        }
    }

    pub fn total_pages(&self) -> i64 {
        if self.limit == 0 {
            1
        } else {
            (self.total + self.limit - 1) / self.limit
        }
    }

    pub fn has_next(&self) -> bool {
        self.offset + self.limit < self.total
    }

    pub fn has_prev(&self) -> bool {
        self.offset > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_default() {
        let p = Pagination::default();
        assert_eq!(p.limit, 20);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn test_pagination_page() {
        let p = Pagination::page(3, 10);
        assert_eq!(p.limit, 10);
        assert_eq!(p.offset, 20);
    }

    #[test]
    fn test_paginated_result() {
        let items = vec![1, 2, 3, 4, 5];
        let result = PaginatedResult::new(items, 50, Pagination::page(2, 5));

        assert_eq!(result.page(), 2);
        assert_eq!(result.total_pages(), 10);
        assert!(result.has_next());
        assert!(result.has_prev());
    }
}
