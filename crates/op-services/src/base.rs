//! Base service traits and implementations
//!
//! Mirrors:
//! - app/services/base_services/base_callable.rb
//! - app/services/base_services/base_contracted.rb
//! - app/services/base_services/write.rb

use async_trait::async_trait;
use op_contracts::base::{Contract, UserContext};
use op_core::error::ValidationErrors;

use crate::result::ServiceResult;

/// Base trait for all callable services
#[async_trait]
pub trait Callable<Params, Output> {
    /// Execute the service
    async fn call(&self, params: Params) -> ServiceResult<Output>;
}

/// Service context for managing service execution
pub struct ServiceContext<'a, U: UserContext> {
    pub user: &'a U,
    pub send_notifications: bool,
}

impl<'a, U: UserContext> ServiceContext<'a, U> {
    pub fn new(user: &'a U) -> Self {
        Self {
            user,
            send_notifications: true,
        }
    }

    pub fn without_notifications(user: &'a U) -> Self {
        Self {
            user,
            send_notifications: false,
        }
    }
}

/// Base contracted service that validates through contracts
pub struct BaseContractedService<'a, U: UserContext, C> {
    user: &'a U,
    contract: C,
}

impl<'a, U: UserContext, C> BaseContractedService<'a, U, C> {
    pub fn new(user: &'a U, contract: C) -> Self {
        Self { user, contract }
    }

    pub fn user(&self) -> &U {
        self.user
    }

    pub fn contract(&self) -> &C {
        &self.contract
    }

    /// Validate entity through contract
    pub fn validate<T>(&self, entity: &T) -> Result<(), ValidationErrors>
    where
        C: Contract<T>,
    {
        self.contract.validate(entity)
    }
}

/// Trait for services that modify entities
pub trait WriteService<T> {
    /// The input params type
    type Params;

    /// Set attributes on the entity from params
    fn set_attributes(&self, entity: &mut T, params: &Self::Params) -> Result<(), ValidationErrors>;

    /// Validate the entity
    fn validate(&self, entity: &T) -> Result<(), ValidationErrors>;

    /// Persist the entity (implement in database layer)
    fn persist(&self, entity: &mut T) -> Result<(), ValidationErrors>;
}

/// Trait for create services
pub trait CreateService<T>: WriteService<T> {
    /// Create a new entity instance
    fn new_instance(&self) -> T;

    /// Execute the create operation
    fn execute(&self, params: &Self::Params) -> ServiceResult<T>
    where
        T: Clone,
    {
        let mut entity = self.new_instance();

        // Set attributes
        if let Err(errors) = self.set_attributes(&mut entity, params) {
            return ServiceResult::failure(errors);
        }

        // Validate
        if let Err(errors) = self.validate(&entity) {
            return ServiceResult::failure(errors);
        }

        // Persist
        if let Err(errors) = self.persist(&mut entity) {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(entity)
    }
}

/// Trait for update services
pub trait UpdateService<T>: WriteService<T> {
    /// Execute the update operation
    fn execute(&self, entity: &mut T, params: &Self::Params) -> ServiceResult<T>
    where
        T: Clone,
    {
        // Set attributes
        if let Err(errors) = self.set_attributes(entity, params) {
            return ServiceResult::failure(errors);
        }

        // Validate
        if let Err(errors) = self.validate(entity) {
            return ServiceResult::failure(errors);
        }

        // Persist
        if let Err(errors) = self.persist(entity) {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(entity.clone())
    }
}

/// Trait for delete services
pub trait DeleteService<T> {
    /// Validate the delete can proceed
    fn validate_delete(&self, entity: &T) -> Result<(), ValidationErrors>;

    /// Execute the delete (implement in database layer)
    fn do_delete(&self, entity: &T) -> Result<(), ValidationErrors>;

    /// Execute the delete operation
    fn execute(&self, entity: &T) -> ServiceResult<()> {
        // Validate
        if let Err(errors) = self.validate_delete(entity) {
            return ServiceResult::failure(errors);
        }

        // Delete
        if let Err(errors) = self.do_delete(entity) {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(())
    }
}

/// Params for setting attributes (generic wrapper)
#[derive(Debug, Clone, Default)]
pub struct SetAttributesParams<T> {
    pub attributes: T,
    /// Skip contract validation (for internal operations)
    pub skip_validation: bool,
}

impl<T> SetAttributesParams<T> {
    pub fn new(attributes: T) -> Self {
        Self {
            attributes,
            skip_validation: false,
        }
    }

    pub fn skip_validation(mut self) -> Self {
        self.skip_validation = true;
        self
    }
}

/// Async versions of the service traits for database operations

#[async_trait]
pub trait AsyncWriteService<T>: Send + Sync {
    type Params: Send + Sync;

    /// Set attributes on the entity from params
    fn set_attributes(&self, entity: &mut T, params: &Self::Params) -> Result<(), ValidationErrors>;

    /// Validate the entity
    fn validate(&self, entity: &T) -> Result<(), ValidationErrors>;

    /// Persist the entity to database
    async fn persist(&self, entity: &mut T) -> Result<(), ValidationErrors>;
}

#[async_trait]
pub trait AsyncCreateService<T: Send + Sync>: AsyncWriteService<T> {
    /// Create a new entity instance
    fn new_instance(&self) -> T;

    /// Execute the create operation
    async fn execute(&self, params: &Self::Params) -> ServiceResult<T>
    where
        T: Clone,
    {
        let mut entity = self.new_instance();

        // Set attributes
        if let Err(errors) = self.set_attributes(&mut entity, params) {
            return ServiceResult::failure(errors);
        }

        // Validate
        if let Err(errors) = self.validate(&entity) {
            return ServiceResult::failure(errors);
        }

        // Persist
        if let Err(errors) = self.persist(&mut entity).await {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(entity)
    }
}

#[async_trait]
pub trait AsyncUpdateService<T: Send + Sync>: AsyncWriteService<T> {
    /// Execute the update operation
    async fn execute(&self, entity: &mut T, params: &Self::Params) -> ServiceResult<T>
    where
        T: Clone,
    {
        // Set attributes
        if let Err(errors) = self.set_attributes(entity, params) {
            return ServiceResult::failure(errors);
        }

        // Validate
        if let Err(errors) = self.validate(entity) {
            return ServiceResult::failure(errors);
        }

        // Persist
        if let Err(errors) = self.persist(entity).await {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(entity.clone())
    }
}

#[async_trait]
pub trait AsyncDeleteService<T: Send + Sync>: Send + Sync {
    /// Validate the delete can proceed
    fn validate_delete(&self, entity: &T) -> Result<(), ValidationErrors>;

    /// Execute the delete in database
    async fn do_delete(&self, entity: &T) -> Result<(), ValidationErrors>;

    /// Execute the delete operation
    async fn execute(&self, entity: &T) -> ServiceResult<()> {
        // Validate
        if let Err(errors) = self.validate_delete(entity) {
            return ServiceResult::failure(errors);
        }

        // Delete
        if let Err(errors) = self.do_delete(entity).await {
            return ServiceResult::failure(errors);
        }

        ServiceResult::success(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock entity
    #[derive(Clone, Debug, Default)]
    struct MockEntity {
        id: Option<i64>,
        name: String,
    }

    // Mock params
    #[derive(Clone, Debug)]
    struct MockParams {
        name: String,
    }

    // Mock create service
    struct MockCreateService;

    impl WriteService<MockEntity> for MockCreateService {
        type Params = MockParams;

        fn set_attributes(
            &self,
            entity: &mut MockEntity,
            params: &Self::Params,
        ) -> Result<(), ValidationErrors> {
            entity.name = params.name.clone();
            Ok(())
        }

        fn validate(&self, entity: &MockEntity) -> Result<(), ValidationErrors> {
            if entity.name.is_empty() {
                let mut errors = ValidationErrors::new();
                errors.add("name", "can't be blank");
                return Err(errors);
            }
            Ok(())
        }

        fn persist(&self, entity: &mut MockEntity) -> Result<(), ValidationErrors> {
            entity.id = Some(1);
            Ok(())
        }
    }

    impl CreateService<MockEntity> for MockCreateService {
        fn new_instance(&self) -> MockEntity {
            MockEntity::default()
        }
    }

    #[test]
    fn test_create_service_success() {
        let service = MockCreateService;
        let params = MockParams {
            name: "Test".to_string(),
        };

        let result = service.execute(&params);
        assert!(result.is_success());
        let entity = result.result().unwrap();
        assert_eq!(entity.name, "Test");
        assert_eq!(entity.id, Some(1));
    }

    #[test]
    fn test_create_service_validation_failure() {
        let service = MockCreateService;
        let params = MockParams {
            name: "".to_string(),
        };

        let result = service.execute(&params);
        assert!(result.is_failure());
        assert!(result.errors().has_error("name"));
    }
}
