# Services Port Status

## Overview

OpenProject uses the Service Object pattern extensively. Services handle business logic
and are organized under `app/services/` with a common base class pattern.

## Base Services

| Ruby Service | Rust Trait | Status | Notes |
|-------------|-----------|--------|-------|
| `BaseServices::Create` | `op-services::CreateService<T>` | ⬜ | Generic create |
| `BaseServices::Update` | `op-services::UpdateService<T>` | ⬜ | Generic update |
| `BaseServices::Delete` | `op-services::DeleteService<T>` | ⬜ | Generic delete |
| `BaseServices::Write` | `op-services::WriteService<T>` | ⬜ | Create or Update |
| `BaseServices::Copy` | `op-services::CopyService<T>` | ⬜ | Entity copying |
| `BaseServices::SetAttributes` | `op-services::SetAttributes<T>` | ⬜ | Attribute application |

## Domain Services

### Work Packages
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `WorkPackages::CreateService` | `op-services::work_packages::CreateService` | ⬜ | |
| `WorkPackages::UpdateService` | `op-services::work_packages::UpdateService` | ⬜ | |
| `WorkPackages::DeleteService` | `op-services::work_packages::DeleteService` | ⬜ | |
| `WorkPackages::CopyService` | `op-services::work_packages::CopyService` | ⬜ | |
| `WorkPackages::MoveService` | `op-services::work_packages::MoveService` | ⬜ | |
| `WorkPackages::SetAttributesService` | `op-services::work_packages::SetAttributesService` | ⬜ | |
| `WorkPackages::UpdateAncestorsService` | `op-services::work_packages::UpdateAncestorsService` | ⬜ | |
| `WorkPackages::ScheduleDependencyService` | `op-services::work_packages::ScheduleDependencyService` | ⬜ | |

### Users
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Users::CreateService` | `op-services::users::CreateService` | ⬜ | |
| `Users::UpdateService` | `op-services::users::UpdateService` | ⬜ | |
| `Users::DeleteService` | `op-services::users::DeleteService` | ⬜ | |
| `Users::RegisterUserService` | `op-services::users::RegisterService` | ⬜ | |
| `Users::LoginService` | `op-services::users::LoginService` | ⬜ | |
| `Users::ChangePasswordService` | `op-services::users::ChangePasswordService` | ⬜ | |

### Projects
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Projects::CreateService` | `op-services::projects::CreateService` | ⬜ | |
| `Projects::UpdateService` | `op-services::projects::UpdateService` | ⬜ | |
| `Projects::DeleteService` | `op-services::projects::DeleteService` | ⬜ | |
| `Projects::CopyService` | `op-services::projects::CopyService` | ⬜ | Complex |
| `Projects::ArchiveService` | `op-services::projects::ArchiveService` | ⬜ | |

### Members
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Members::CreateService` | `op-services::members::CreateService` | ⬜ | |
| `Members::UpdateService` | `op-services::members::UpdateService` | ⬜ | |
| `Members::DeleteService` | `op-services::members::DeleteService` | ⬜ | |

### Notifications
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Notifications::CreateService` | `op-services::notifications::CreateService` | ⬜ | |
| `Notifications::MarkAsReadService` | `op-services::notifications::MarkAsReadService` | ⬜ | |
| `Notifications::CreateFromJournalService` | `op-services::notifications::FromJournalService` | ⬜ | |

### Journals
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Journals::CreateService` | `op-services::journals::CreateService` | ⬜ | Audit logging |

### Attachments
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Attachments::CreateService` | `op-services::attachments::CreateService` | ⬜ | File upload |
| `Attachments::PrepareUploadService` | `op-services::attachments::PrepareUploadService` | ⬜ | Direct upload |

### OAuth
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `OAuth::Applications::CreateService` | `op-services::oauth::CreateAppService` | ⬜ | |
| `OAuth::Applications::UpdateService` | `op-services::oauth::UpdateAppService` | ⬜ | |
| `OAuth::GrantTokenService` | `op-services::oauth::GrantTokenService` | ⬜ | |

### Webhooks
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Webhooks::CreateService` | `op-services::webhooks::CreateService` | ⬜ | |
| `Webhooks::UpdateService` | `op-services::webhooks::UpdateService` | ⬜ | |
| `Webhooks::OutgoingWebhookService` | `op-services::webhooks::OutgoingService` | ⬜ | |

### Queries
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `Queries::CreateService` | `op-services::queries::CreateService` | ⬜ | |
| `Queries::UpdateService` | `op-services::queries::UpdateService` | ⬜ | |

### Custom Fields
| Ruby Service | Rust Module | Status | Notes |
|-------------|-------------|--------|-------|
| `CustomFields::CreateService` | `op-services::custom_fields::CreateService` | ⬜ | |
| `CustomFields::UpdateService` | `op-services::custom_fields::UpdateService` | ⬜ | |

## Background Jobs (Workers)

| Ruby Worker | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Mails::*` | `op-services::mail::*` | ⬜ | Email sending |
| `Notifications::*` | `op-services::notifications::workers::*` | ⬜ | |
| `Journals::CompletedJob` | `op-services::journals::CompletedWorker` | ⬜ | |
| `WorkPackages::*` | `op-services::work_packages::workers::*` | ⬜ | |

## Mailers

| Ruby Mailer | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `UserMailer` | `op-services::mail::UserMailer` | ⬜ | |
| `WorkPackageMailer` | `op-services::mail::WorkPackageMailer` | ⬜ | |
| `MemberMailer` | `op-services::mail::MemberMailer` | ⬜ | |

## Progress Summary

- Total Services: ~100+
- Completed: 0
- In Progress: 0
- Not Started: 100+

## Implementation Notes

### ServiceResult Pattern

Ruby:
```ruby
ServiceResult.success(result: work_package)
ServiceResult.failure(errors: contract.errors)
```

Rust:
```rust
ServiceResult::success(work_package)
ServiceResult::failure(validation_errors)
```

### Service Call Pattern

Ruby:
```ruby
WorkPackages::CreateService
  .new(user: current_user)
  .call(params)
```

Rust:
```rust
CreateService::new(current_user)
    .call(params)
    .await
```
