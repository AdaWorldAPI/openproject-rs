//! # op-services
//!
//! Business logic services for OpenProject RS.
//!
//! This crate implements the service layer pattern, mirroring OpenProject's
//! Ruby service objects. Services encapsulate business logic and coordinate
//! between contracts, models, and persistence.
//!
//! ## Structure
//!
//! - `result` - ServiceResult type for operation outcomes
//! - `base` - Base service traits (Callable, WriteService, etc.)
//! - `work_packages` - Work package CRUD services
//!
//! ## Example
//!
//! ```ignore
//! use op_services::work_packages::{CreateWorkPackageService, WorkPackageParams};
//!
//! let service = CreateWorkPackageService::new(&user);
//! let params = WorkPackageParams::new()
//!     .with_subject("New Feature")
//!     .with_project_id(1);
//!
//! let result = service.call(params);
//! if result.success() {
//!     println!("Created: {:?}", result.result());
//! } else {
//!     println!("Errors: {:?}", result.errors());
//! }
//! ```

pub mod result;
pub mod base;
pub mod work_packages;
pub mod projects;
pub mod users;

// Re-exports
pub use result::ServiceResult;
pub use base::{
    Callable,
    ServiceContext,
    WriteService,
    CreateService,
    UpdateService,
    DeleteService,
    AsyncWriteService,
    AsyncCreateService,
    AsyncUpdateService,
    AsyncDeleteService,
};

// Re-export contract types for convenience
pub use op_contracts::base as base_contracts;
