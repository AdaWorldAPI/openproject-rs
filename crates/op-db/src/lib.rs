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
pub mod time_entries;
pub mod statuses;
pub mod priorities;
pub mod types;
pub mod roles;
pub mod versions;
pub mod members;
pub mod activities;
pub mod categories;
pub mod relations;
pub mod watchers;
pub mod attachments;
pub mod queries;

// Re-exports
pub use pool::{Database, DatabaseConfig, PoolStats};
pub use repository::{
    Pagination, PaginatedResult, Repository, RepositoryContext, RepositoryError, RepositoryResult,
};
pub use work_packages::{CreateWorkPackageDto, UpdateWorkPackageDto, WorkPackageRepository};
pub use users::{status as user_status, CreateUserDto, UpdateUserDto, UserRepository, UserRow};
pub use projects::{CreateProjectDto, UpdateProjectDto, ProjectRepository, ProjectRow};
pub use query_executor::{WorkPackageQueryExecutor, WorkPackageRow};
pub use time_entries::{CreateTimeEntryDto, UpdateTimeEntryDto, TimeEntryRepository, TimeEntryRow};
pub use statuses::{CreateStatusDto, UpdateStatusDto, StatusRepository, StatusRow};
pub use priorities::{CreatePriorityDto, UpdatePriorityDto, PriorityRepository, PriorityRow};
pub use types::{CreateTypeDto, UpdateTypeDto, TypeRepository, TypeRow};
pub use roles::{CreateRoleDto, UpdateRoleDto, RoleRepository, RoleRow};
pub use versions::{CreateVersionDto, UpdateVersionDto, VersionRepository, VersionRow};
pub use members::{CreateMemberDto, UpdateMemberDto, MemberRepository, MemberRow, MemberWithRoles};
pub use activities::{CreateActivityDto, UpdateActivityDto, ActivityRepository, ActivityRow};
pub use categories::{CreateCategoryDto, UpdateCategoryDto, CategoryRepository, CategoryRow};
pub use relations::{relation_type, CreateRelationDto, UpdateRelationDto, RelationRepository, RelationRow};
pub use watchers::{CreateWatcherDto, UpdateWatcherDto, WatcherRepository, WatcherRow, WatcherWithUser};
pub use attachments::{status as attachment_status, CreateAttachmentDto, UpdateAttachmentDto, AttachmentRepository, AttachmentRow};
pub use queries::{CreateQueryDto, UpdateQueryDto, QueryRepository, QueryRow, QueryWithStarred};
