//! API Representers
//!
//! HAL+JSON representers for OpenProject API v3 compatibility.
//! These convert domain models to API response format.

pub mod hal;
pub mod work_package;
pub mod project;
pub mod user;
pub mod query;

// Re-exports
pub use hal::{HalCollection, HalEmbedded, HalError, HalLink, HalLinks, HalResource};
pub use work_package::{WorkPackageData, WorkPackageRepresenter, EmbedOptions};
