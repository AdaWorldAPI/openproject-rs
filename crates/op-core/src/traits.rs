//! Core traits that mirror OpenProject's Ruby patterns
//!
//! These traits provide the foundational interfaces for models, services, and contracts.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::result::{OpResult, ServiceResult};

/// Primary key type (mirrors Rails' ID)
pub type Id = i64;

/// Trait for entities that have a primary key
pub trait Identifiable {
    fn id(&self) -> Option<Id>;
    fn is_persisted(&self) -> bool {
        self.id().is_some()
    }
    fn is_new_record(&self) -> bool {
        !self.is_persisted()
    }
}

/// Trait for entities with UUID identifiers (for API exposure)
pub trait UuidIdentifiable {
    fn uuid(&self) -> Uuid;
}

/// Trait for entities with timestamps (created_at, updated_at)
pub trait Timestamped {
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn updated_at(&self) -> Option<DateTime<Utc>>;
}

/// Trait for soft-deletable entities
pub trait SoftDeletable {
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// Trait for entities that track who created/updated them
pub trait Auditable {
    fn created_by_id(&self) -> Option<Id>;
    fn updated_by_id(&self) -> Option<Id>;
}

/// Trait for lockable entities (optimistic locking)
pub trait Lockable {
    fn lock_version(&self) -> i32;
}

/// Trait for entities that belong to a project
pub trait ProjectScoped {
    fn project_id(&self) -> Option<Id>;
}

/// Base trait for all domain entities
pub trait Entity: Identifiable + Timestamped + Send + Sync {
    /// The database table name
    const TABLE_NAME: &'static str;

    /// Human-readable type name for error messages
    const TYPE_NAME: &'static str;
}

/// Trait for service objects (mirrors OpenProject's BaseServices::*)
///
/// In Ruby:
/// ```ruby
/// class WorkPackages::CreateService < BaseServices::Create
///   def perform(params)
///     # ...
///   end
/// end
/// ```
#[async_trait]
pub trait Service<Input, Output> {
    /// Perform the service operation
    async fn call(&self, input: Input) -> ServiceResult<Output>;
}

/// Trait for contract validation (mirrors OpenProject's Contracts)
///
/// In Ruby:
/// ```ruby
/// class WorkPackages::CreateContract < BaseContract
///   validate :validate_user_allowed_to_create
///   attribute :subject
/// end
/// ```
pub trait Contract<T> {
    /// Validate the entity
    fn validate(&self, entity: &T, user: &dyn UserContext) -> OpResult<()>;

    /// Get the list of writable attributes
    fn writable_attributes(&self) -> &[&'static str];

    /// Check if an attribute is writable
    fn attribute_writable(&self, attribute: &str) -> bool {
        self.writable_attributes().contains(&attribute)
    }
}

/// User context for permission checks
pub trait UserContext: Send + Sync {
    fn user_id(&self) -> Id;
    fn is_admin(&self) -> bool;
    fn is_anonymous(&self) -> bool;
    fn is_logged_in(&self) -> bool {
        !self.is_anonymous()
    }
    /// Check if user has permission in a project
    fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool;
    /// Check if user has global permission
    fn allowed_globally(&self, permission: &str) -> bool;
}

/// Trait for repository pattern (data access)
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync {
    /// Find by primary key
    async fn find(&self, id: Id) -> OpResult<T>;

    /// Find by primary key, returning None if not found
    async fn find_optional(&self, id: Id) -> OpResult<Option<T>>;

    /// Find all entities
    async fn find_all(&self) -> OpResult<Vec<T>>;

    /// Create a new entity
    async fn create(&self, entity: &T) -> OpResult<T>;

    /// Update an existing entity
    async fn update(&self, entity: &T) -> OpResult<T>;

    /// Delete an entity
    async fn delete(&self, id: Id) -> OpResult<()>;

    /// Check if entity exists
    async fn exists(&self, id: Id) -> OpResult<bool>;

    /// Count all entities
    async fn count(&self) -> OpResult<i64>;
}

/// Trait for query builders (mirrors ActiveRecord scopes)
pub trait Queryable: Sized {
    type Query;

    /// Start a new query
    fn query() -> Self::Query;
}

/// Trait for entities that can be represented in HAL+JSON format
pub trait HalRepresentable: Serialize {
    /// Get the HAL _type field
    fn hal_type(&self) -> &'static str;

    /// Get the self link href
    fn self_href(&self) -> String;

    /// Additional HAL links
    fn hal_links(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// Embedded resources
    fn hal_embedded(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

/// Trait for write models (form/input representations)
pub trait WriteModel: DeserializeOwned + Send {
    type Entity;

    /// Apply the write model to an entity
    fn apply_to(&self, entity: &mut Self::Entity);

    /// Create a new entity from this write model
    fn to_entity(&self) -> Self::Entity;
}
