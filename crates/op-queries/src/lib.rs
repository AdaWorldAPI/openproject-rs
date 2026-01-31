//! # op-queries
//!
//! Query system for OpenProject RS.
//!
//! This crate implements the query layer for filtering, sorting, and displaying
//! work packages. It mirrors OpenProject's Query model and associated classes.
//!
//! ## Structure
//!
//! - `filters` - Filter types and operators for querying work packages
//! - `sorts` - Sort orders and directions
//! - `columns` - Column configuration for display
//! - `query` - The Query model for saved views
//! - `builder` - Fluent API for constructing queries
//!
//! ## Example
//!
//! ```
//! use op_queries::builder::{QueryBuilder, presets};
//! use op_queries::filters::{Filter, FilterValue};
//!
//! // Build a custom query
//! let query = QueryBuilder::new()
//!     .name("Open bugs")
//!     .project(1)
//!     .type_ids(vec![3]) // Bug type
//!     .sort_by_priority()
//!     .group_by_status()
//!     .build();
//!
//! assert!(query.has_filters());
//! assert!(query.is_grouped());
//!
//! // Or use a preset
//! let my_work = presets::my_work_packages();
//! ```

pub mod filters;
pub mod sorts;
pub mod columns;
pub mod query;
pub mod builder;

// Re-exports for convenience
pub use filters::{Filter, FilterOperator, FilterSet, FilterValue};
pub use sorts::{SortCriterion, SortDirection, SortOrder};
pub use columns::{Column, ColumnSet, ColumnType};
pub use query::{DisplayRepresentation, GroupBy, Query, QueryVisibility};
pub use builder::{QueryBuilder, presets};
