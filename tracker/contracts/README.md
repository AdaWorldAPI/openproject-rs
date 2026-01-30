# Contracts Port Status

## Overview

OpenProject uses a Contract pattern for validation that separates validation logic from models.
This is implemented via the `ModelContract` base class in `app/contracts/model_contract.rb`.

## Base Contracts

| Ruby Contract | Rust Trait/Struct | Status | Notes |
|--------------|-------------------|--------|-------|
| `ModelContract` | `op-contracts::ModelContract` | ⬜ | Base contract trait |
| `BaseContract` | `op-contracts::BaseContract` | ⬜ | Alias for ModelContract |
| `DeleteContract` | `op-contracts::DeleteContract` | ⬜ | For delete operations |

## Contract Hierarchy Pattern

In Ruby:
```ruby
module WorkPackages
  class BaseContract < ModelContract
    # Shared validations
  end

  class CreateContract < BaseContract
    # Create-specific validations
  end

  class UpdateContract < BaseContract
    # Update-specific validations
  end
end
```

In Rust:
```rust
pub trait WorkPackageContract: Contract<WorkPackage> {
    // Shared validations
}

pub struct CreateContract { user: User }
pub struct UpdateContract { user: User }

impl WorkPackageContract for CreateContract { }
impl WorkPackageContract for UpdateContract { }
```

## Domain Contracts

### Work Packages
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `WorkPackages::BaseContract` | `op-contracts::work_packages::BaseContract` | ⬜ | |
| `WorkPackages::CreateContract` | `op-contracts::work_packages::CreateContract` | ⬜ | |
| `WorkPackages::UpdateContract` | `op-contracts::work_packages::UpdateContract` | ⬜ | |
| `WorkPackages::DeleteContract` | `op-contracts::work_packages::DeleteContract` | ⬜ | |

### Users
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Users::BaseContract` | `op-contracts::users::BaseContract` | ⬜ | |
| `Users::CreateContract` | `op-contracts::users::CreateContract` | ⬜ | |
| `Users::UpdateContract` | `op-contracts::users::UpdateContract` | ⬜ | |
| `Users::DeleteContract` | `op-contracts::users::DeleteContract` | ⬜ | |

### Projects
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Projects::BaseContract` | `op-contracts::projects::BaseContract` | ⬜ | |
| `Projects::CreateContract` | `op-contracts::projects::CreateContract` | ⬜ | |
| `Projects::UpdateContract` | `op-contracts::projects::UpdateContract` | ⬜ | |
| `Projects::DeleteContract` | `op-contracts::projects::DeleteContract` | ⬜ | |

### Members
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Members::BaseContract` | `op-contracts::members::BaseContract` | ⬜ | |
| `Members::CreateContract` | `op-contracts::members::CreateContract` | ⬜ | |
| `Members::UpdateContract` | `op-contracts::members::UpdateContract` | ⬜ | |
| `Members::DeleteContract` | `op-contracts::members::DeleteContract` | ⬜ | |

### Queries
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Queries::BaseContract` | `op-contracts::queries::BaseContract` | ⬜ | |
| `Queries::CreateContract` | `op-contracts::queries::CreateContract` | ⬜ | |
| `Queries::UpdateContract` | `op-contracts::queries::UpdateContract` | ⬜ | |

### Time Entries
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `TimeEntries::BaseContract` | `op-contracts::time_entries::BaseContract` | ⬜ | |
| `TimeEntries::CreateContract` | `op-contracts::time_entries::CreateContract` | ⬜ | |
| `TimeEntries::UpdateContract` | `op-contracts::time_entries::UpdateContract` | ⬜ | |

### Attachments
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Attachments::BaseContract` | `op-contracts::attachments::BaseContract` | ⬜ | |
| `Attachments::CreateContract` | `op-contracts::attachments::CreateContract` | ⬜ | |

### Notifications
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Notifications::BaseContract` | `op-contracts::notifications::BaseContract` | ⬜ | |
| `Notifications::CreateContract` | `op-contracts::notifications::CreateContract` | ⬜ | |
| `Notifications::UpdateContract` | `op-contracts::notifications::UpdateContract` | ⬜ | |

### Versions
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Versions::BaseContract` | `op-contracts::versions::BaseContract` | ⬜ | |
| `Versions::CreateContract` | `op-contracts::versions::CreateContract` | ⬜ | |
| `Versions::UpdateContract` | `op-contracts::versions::UpdateContract` | ⬜ | |

### OAuth Applications
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `OAuth::Applications::BaseContract` | `op-contracts::oauth::BaseContract` | ⬜ | |
| `OAuth::Applications::CreateContract` | `op-contracts::oauth::CreateContract` | ⬜ | |
| `OAuth::Applications::UpdateContract` | `op-contracts::oauth::UpdateContract` | ⬜ | |

### Webhooks
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Webhooks::BaseContract` | `op-contracts::webhooks::BaseContract` | ⬜ | |
| `Webhooks::CreateContract` | `op-contracts::webhooks::CreateContract` | ⬜ | |
| `Webhooks::UpdateContract` | `op-contracts::webhooks::UpdateContract` | ⬜ | |

## Module Contracts

### Boards (modules/boards)
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Boards::BaseContract` | `op-contracts::boards::BaseContract` | ⬜ | |

### Budgets (modules/budgets)
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Budgets::BaseContract` | `op-contracts::budgets::BaseContract` | ⬜ | |

### Meetings (modules/meeting)
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Meetings::BaseContract` | `op-contracts::meetings::BaseContract` | ⬜ | |

### Storages (modules/storages)
| Ruby Contract | Rust Module | Status | Notes |
|--------------|-------------|--------|-------|
| `Storages::BaseContract` | `op-contracts::storages::BaseContract` | ⬜ | |

## Contract Validation Patterns

Key validation patterns to implement:

1. **attribute_writable?** - Check if attribute can be modified
2. **validate_*_allowed** - Permission checks
3. **validate_*_exists** - Reference validation
4. **validate_*_not_nil** - Required field checks
5. **validate_*_in_range** - Value range validation
6. **validate_*_unique** - Uniqueness constraints
7. **validate_user_allowed_to_*** - User permission checks

## Progress Summary

- Total Contracts: ~50+
- Completed: 0
- In Progress: 0
- Not Started: 50+
