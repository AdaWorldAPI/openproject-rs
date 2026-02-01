//! # op-db
//!
//! Database layer for OpenProject RS.
//!
//! This crate provides PostgreSQL database access using SQLx, including:
//!
//! - Connection pool management
//! - Repository pattern for CRUD operations
//! - Entity mappings for work packages, users, and projects
//!
//! ## Example
//!
//! ```ignore
//! use op_db::{Database, DatabaseConfig};
//! use op_db::work_packages::WorkPackageRepository;
//! use op_db::repository::Repository;
//!
//! let config = DatabaseConfig::from_env();
//! let db = Database::connect(&config).await?;
//!
//! let repo = WorkPackageRepository::new(db.pool().clone());
//! let work_package = repo.find_by_id(1).await?;
//! ```

pub mod pool;
pub mod repository;
pub mod work_packages;
pub mod users;
pub mod projects;
pub mod query_executor;

// Re-exports
pub use pool::{Database, DatabaseConfig, PoolStats};
pub use repository::{
    Pagination, PaginatedResult, Repository, RepositoryContext, RepositoryError, RepositoryResult,
};
pub use work_packages::{CreateWorkPackageDto, UpdateWorkPackageDto, WorkPackageRepository};
pub use users::{CreateUserDto, UpdateUserDto, UserRepository, UserRow};
pub use projects::{CreateProjectDto, UpdateProjectDto, ProjectRepository, ProjectRow};
pub use query_executor::{WorkPackageQueryExecutor, WorkPackageRow};
