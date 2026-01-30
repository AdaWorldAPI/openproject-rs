# OpenProject RS - Complete Inventory

> Full excavation of Ruby OpenProject codebase for Rust port

## Executive Summary

This document catalogs all components from the Ruby OpenProject codebase that need to be ported to Rust. It serves as the master reference for the porting effort.

| Category | Count | Ported | Progress |
|----------|-------|--------|----------|
| Models | 80+ | 2 | 2.5% |
| Contracts | 50+ | 1 | 2% |
| Services | 100+ | 0 | 0% |
| API Endpoints | 150+ | 0 | 0% |
| Representers | 30+ | 0 | 0% |
| Database Tables | 100+ | 0 | 0% |
| **Total** | **~510** | **3** | **~0.6%** |

---

## I. Core Architecture Patterns

### Ruby → Rust Pattern Mapping

| Ruby Pattern | Rust Implementation |
|--------------|---------------------|
| ActiveRecord Model | Struct + Entity trait |
| ModelContract | Contract trait |
| BaseServices::Create | CreateService<T> |
| BaseServices::Update | UpdateService<T> |
| BaseServices::Delete | DeleteService<T> |
| ServiceResult | Result<T, OpError> |
| Representer (Roar) | Serialize + HalRepresentable trait |
| ActiveRecord Scope | Query builder methods |
| Concern | Trait or module |
| Callback (after_save, etc) | Explicit service calls |
| STI (Single Table Inheritance) | Enum or separate types |
| Polymorphic Association | Generic + trait objects |

### Key Trait Hierarchy

```rust
// Core traits (op-core::traits)
trait Identifiable { fn id(&self) -> Option<Id>; }
trait Timestamped { fn created_at(&self) -> Option<DateTime<Utc>>; }
trait Entity: Identifiable + Timestamped + Send + Sync { }
trait Service<Input, Output> { async fn call(&self, input: Input) -> ServiceResult<Output>; }
trait Contract<T> { fn validate(&self, entity: &T, user: &dyn UserContext) -> OpResult<()>; }
trait Repository<T: Entity> { async fn find(&self, id: Id) -> OpResult<T>; }
trait UserContext { fn allowed_in_project(&self, permission: &str, project_id: Id) -> bool; }
trait HalRepresentable { fn hal_type(&self) -> &'static str; fn self_href(&self) -> String; }
```

---

## II. Models Inventory

### Core Domain Models

#### Users & Authentication (7 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| User | `User` | `op_models::user::User` | 🟢 |
| Group | `Group` | `op_models::user::Group` | ⬜ |
| PlaceholderUser | `PlaceholderUser` | `op_models::user::PlaceholderUser` | ⬜ |
| Principal | `Principal` | `op_models::user::Principal` | ⬜ |
| UserPreference | `UserPreference` | `op_models::user::UserPreference` | ⬜ |
| UserPassword | `UserPassword` | `op_models::user::UserPassword` | ⬜ |
| Session | `Session` | `op_models::user::Session` | ⬜ |

#### Projects (4 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Project | `Project` | `op_models::project::Project` | ⬜ |
| ProjectStatus | `Projects::Status` | `op_models::project::ProjectStatus` | ⬜ |
| EnabledModule | `EnabledModule` | `op_models::project::EnabledModule` | ⬜ |
| ProjectCustomFieldProjectMapping | - | `op_models::project::CustomFieldMapping` | ⬜ |

#### Work Packages (10 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| WorkPackage | `WorkPackage` | `op_models::work_package::WorkPackage` | 🟢 |
| Status | `Status` | `op_models::status::Status` | ⬜ |
| Type | `Type` | `op_models::type_def::Type` | ⬜ |
| Priority | `IssuePriority` | `op_models::priority::Priority` | ⬜ |
| Version | `Version` | `op_models::version::Version` | ⬜ |
| Category | `Category` | `op_models::category::Category` | ⬜ |
| Relation | `Relation` | `op_models::relation::Relation` | ⬜ |
| Watcher | `Watcher` | `op_models::watcher::Watcher` | ⬜ |
| WorkPackageJournal | `Journal::WorkPackageJournal` | `op_models::journal::WorkPackageJournal` | ⬜ |
| WorkPackageCustomField | `WorkPackageCustomField` | `op_models::custom_field::WorkPackageCustomField` | ⬜ |

#### Memberships & Roles (4 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Member | `Member` | `op_models::member::Member` | ⬜ |
| MemberRole | `MemberRole` | `op_models::member::MemberRole` | ⬜ |
| Role | `Role` | `op_models::role::Role` | ⬜ |
| RolePermission | - | `op_models::role::RolePermission` | ⬜ |

#### Journals & Activity (4 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Journal | `Journal` | `op_models::journal::Journal` | ⬜ |
| Customizable Journal | `Journal::CustomizableJournal` | `op_models::journal::CustomizableJournal` | ⬜ |
| Attachable Journal | `Journal::AttachableJournal` | `op_models::journal::AttachableJournal` | ⬜ |
| Activity | `Activity` | `op_models::activity::Activity` | ⬜ |

#### Custom Fields (10+ models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| CustomField | `CustomField` | `op_models::custom_field::CustomField` | ⬜ |
| CustomValue | `CustomValue` | `op_models::custom_value::CustomValue` | ⬜ |
| CustomOption | `CustomOption` | `op_models::custom_field::CustomOption` | ⬜ |
| WorkPackageCustomField | `WorkPackageCustomField` | - | ⬜ |
| ProjectCustomField | `ProjectCustomField` | - | ⬜ |
| UserCustomField | `UserCustomField` | - | ⬜ |
| TimeEntryCustomField | `TimeEntryCustomField` | - | ⬜ |
| VersionCustomField | `VersionCustomField` | - | ⬜ |

#### Attachments (3 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Attachment | `Attachment` | `op_models::attachment::Attachment` | ⬜ |
| AttachableJournal | `Journal::AttachableJournal` | - | ⬜ |
| FileLink | `Storages::FileLink` | `op_models::file_link::FileLink` | ⬜ |

#### Notifications (3 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Notification | `Notification` | `op_models::notification::Notification` | ⬜ |
| NotificationSetting | `NotificationSetting` | `op_models::notification::NotificationSetting` | ⬜ |
| ReminderNotification | `ReminderNotification` | `op_models::notification::ReminderNotification` | ⬜ |

#### Queries (5 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Query | `Query` | `op_models::query::Query` | ⬜ |
| View | `View` | `op_models::query::View` | ⬜ |
| QueryFilter | Embedded | `op_models::query::QueryFilter` | ⬜ |
| QueryColumn | Embedded | `op_models::query::QueryColumn` | ⬜ |
| QueryOrder | Embedded | `op_models::query::QueryOrder` | ⬜ |

#### Time & Costs (6 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| TimeEntry | `TimeEntry` | `op_models::time_entry::TimeEntry` | ⬜ |
| TimeEntryActivity | `TimeEntryActivity` | `op_models::time_entry::Activity` | ⬜ |
| CostEntry | `CostEntry` | `op_models::cost::CostEntry` | ⬜ |
| CostType | `CostType` | `op_models::cost::CostType` | ⬜ |
| Budget | `Budget` | `op_models::budget::Budget` | ⬜ |
| LaborBudgetItem | `LaborBudgetItem` | `op_models::budget::LaborBudgetItem` | ⬜ |

#### Wiki & Documents (4 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Wiki | `Wiki` | `op_models::wiki::Wiki` | ⬜ |
| WikiPage | `WikiPage` | `op_models::wiki::WikiPage` | ⬜ |
| WikiContent | `WikiContent` | `op_models::wiki::WikiContent` | ⬜ |
| Document | `Document` | `op_models::document::Document` | ⬜ |

#### OAuth (4 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| OAuthApplication | `Doorkeeper::Application` | `op_models::oauth::Application` | ⬜ |
| AccessToken | `Doorkeeper::AccessToken` | `op_models::oauth::AccessToken` | ⬜ |
| AccessGrant | `Doorkeeper::AccessGrant` | `op_models::oauth::AccessGrant` | ⬜ |
| OAuthClientToken | `OAuthClientToken` | `op_models::oauth::ClientToken` | ⬜ |

#### Storage & External Files (5 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Storage | `Storages::Storage` | `op_models::storage::Storage` | ⬜ |
| ProjectStorage | `Storages::ProjectStorage` | `op_models::storage::ProjectStorage` | ⬜ |
| FileLink | `Storages::FileLink` | `op_models::storage::FileLink` | ⬜ |
| NextcloudStorage | `Storages::NextcloudStorage` | `op_models::storage::NextcloudStorage` | ⬜ |
| OneDriveStorage | `Storages::OneDriveStorage` | `op_models::storage::OneDriveStorage` | ⬜ |

#### Webhooks (3 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Webhook | `Webhooks::Webhook` | `op_models::webhook::Webhook` | ⬜ |
| WebhookLog | `Webhooks::Log` | `op_models::webhook::WebhookLog` | ⬜ |
| WebhookEvent | `Webhooks::Event` | `op_models::webhook::WebhookEvent` | ⬜ |

#### Meetings (4 models)
| Model | Ruby Class | Rust Struct | Status |
|-------|------------|-------------|--------|
| Meeting | `Meeting` | `op_models::meeting::Meeting` | ⬜ |
| MeetingAgenda | `MeetingAgenda` | `op_models::meeting::MeetingAgenda` | ⬜ |
| MeetingMinutes | `MeetingMinutes` | `op_models::meeting::MeetingMinutes` | ⬜ |
| MeetingParticipant | `MeetingParticipant` | `op_models::meeting::Participant` | ⬜ |

---

## III. Contracts Inventory

### Base Contracts
| Contract | Ruby Class | Rust Trait/Struct | Status |
|----------|------------|-------------------|--------|
| ModelContract | `ModelContract` | `op_contracts::ModelContract` | 🟢 |
| BaseContract | `BaseContract` | `op_contracts::BaseContract` | 🟢 |
| DeleteContract | `DeleteContract` | `op_contracts::DeleteContract` | ⬜ |

### Domain Contracts

#### Work Packages (4)
- `WorkPackages::BaseContract`
- `WorkPackages::CreateContract`
- `WorkPackages::UpdateContract`
- `WorkPackages::DeleteContract`

#### Users (4)
- `Users::BaseContract`
- `Users::CreateContract`
- `Users::UpdateContract`
- `Users::DeleteContract`

#### Projects (4)
- `Projects::BaseContract`
- `Projects::CreateContract`
- `Projects::UpdateContract`
- `Projects::DeleteContract`

#### Members (4)
- `Members::BaseContract`
- `Members::CreateContract`
- `Members::UpdateContract`
- `Members::DeleteContract`

#### Queries (3)
- `Queries::BaseContract`
- `Queries::CreateContract`
- `Queries::UpdateContract`

#### Time Entries (3)
- `TimeEntries::BaseContract`
- `TimeEntries::CreateContract`
- `TimeEntries::UpdateContract`

#### Attachments (2)
- `Attachments::BaseContract`
- `Attachments::CreateContract`

#### Notifications (3)
- `Notifications::BaseContract`
- `Notifications::CreateContract`
- `Notifications::UpdateContract`

#### Versions (3)
- `Versions::BaseContract`
- `Versions::CreateContract`
- `Versions::UpdateContract`

#### OAuth Applications (3)
- `OAuth::Applications::BaseContract`
- `OAuth::Applications::CreateContract`
- `OAuth::Applications::UpdateContract`

#### Webhooks (3)
- `Webhooks::BaseContract`
- `Webhooks::CreateContract`
- `Webhooks::UpdateContract`

#### Module Contracts
- `Boards::BaseContract`
- `Budgets::BaseContract`
- `Meetings::BaseContract`
- `Storages::BaseContract`

---

## IV. Services Inventory

### Base Services Pattern

```ruby
# Ruby
class BaseServices::Create
  def initialize(user:, contract_class: nil, contract_options: {})
  def call(params) -> ServiceResult
end
```

```rust
// Rust
pub struct CreateService<T, C> {
    user: CurrentUser,
    contract: C,
}
impl<T, C: Contract<T>> Service<CreateParams, T> for CreateService<T, C> {
    async fn call(&self, params: CreateParams) -> ServiceResult<T>;
}
```

### Domain Services

#### Work Packages (10+)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `WorkPackages::CreateService` | `work_packages::CreateService` |
| Update | `WorkPackages::UpdateService` | `work_packages::UpdateService` |
| Delete | `WorkPackages::DeleteService` | `work_packages::DeleteService` |
| Copy | `WorkPackages::CopyService` | `work_packages::CopyService` |
| Move | `WorkPackages::MoveService` | `work_packages::MoveService` |
| SetAttributes | `WorkPackages::SetAttributesService` | `work_packages::SetAttributesService` |
| UpdateAncestors | `WorkPackages::UpdateAncestorsService` | `work_packages::UpdateAncestorsService` |
| ScheduleDependency | `WorkPackages::ScheduleDependencyService` | `work_packages::ScheduleDependencyService` |
| Bulk | `WorkPackages::Bulk::*` | `work_packages::bulk::*` |

#### Users (7)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Users::CreateService` | `users::CreateService` |
| Update | `Users::UpdateService` | `users::UpdateService` |
| Delete | `Users::DeleteService` | `users::DeleteService` |
| Register | `Users::RegisterUserService` | `users::RegisterService` |
| Login | - | `users::LoginService` |
| ChangePassword | - | `users::ChangePasswordService` |
| SetAttributes | `Users::SetAttributesService` | `users::SetAttributesService` |

#### Projects (6)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Projects::CreateService` | `projects::CreateService` |
| Update | `Projects::UpdateService` | `projects::UpdateService` |
| Delete | `Projects::DeleteService` | `projects::DeleteService` |
| Copy | `Projects::CopyService` | `projects::CopyService` |
| Archive | `Projects::ArchiveService` | `projects::ArchiveService` |
| Unarchive | `Projects::UnarchiveService` | `projects::UnarchiveService` |

#### Members (4)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Members::CreateService` | `members::CreateService` |
| Update | `Members::UpdateService` | `members::UpdateService` |
| Delete | `Members::DeleteService` | `members::DeleteService` |
| SetAttributes | `Members::SetAttributesService` | `members::SetAttributesService` |

#### Notifications (4)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Notifications::CreateService` | `notifications::CreateService` |
| MarkRead | `Notifications::MarkAsReadService` | `notifications::MarkAsReadService` |
| FromJournal | `Notifications::CreateFromJournalService` | `notifications::FromJournalService` |
| Bulk | Various | `notifications::BulkService` |

#### Journals (2)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Journals::CreateService` | `journals::CreateService` |
| Complete | `Journals::CompletedJob` | `journals::CompletedService` |

#### Attachments (3)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Attachments::CreateService` | `attachments::CreateService` |
| PrepareUpload | `Attachments::PrepareUploadService` | `attachments::PrepareUploadService` |
| FinishUpload | `Attachments::FinishDirectUploadService` | `attachments::FinishUploadService` |

#### OAuth (3)
| Service | Ruby | Rust |
|---------|------|------|
| CreateApp | `OAuth::Applications::CreateService` | `oauth::CreateAppService` |
| UpdateApp | `OAuth::Applications::UpdateService` | `oauth::UpdateAppService` |
| GrantToken | Various | `oauth::GrantTokenService` |

#### Webhooks (3)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Webhooks::CreateService` | `webhooks::CreateService` |
| Update | `Webhooks::UpdateService` | `webhooks::UpdateService` |
| Outgoing | `Webhooks::OutgoingWebhookService` | `webhooks::OutgoingService` |

#### Queries (3)
| Service | Ruby | Rust |
|---------|------|------|
| Create | `Queries::CreateService` | `queries::CreateService` |
| Update | `Queries::UpdateService` | `queries::UpdateService` |
| SetAttributes | `Queries::SetAttributesService` | `queries::SetAttributesService` |

---

## V. API Endpoints Inventory

See [api/README.md](./api/README.md) for complete endpoint listing.

### Summary by Resource

| Resource | Endpoints | Status |
|----------|-----------|--------|
| Work Packages | 12 | ⬜ |
| Projects | 11 | ⬜ |
| Users | 6 | ⬜ |
| Groups | 5 | ⬜ |
| Memberships | 5 | ⬜ |
| Roles | 2 | ⬜ |
| Statuses | 2 | ⬜ |
| Types | 2 | ⬜ |
| Priorities | 2 | ⬜ |
| Versions | 5 | ⬜ |
| Queries | 7 | ⬜ |
| Time Entries | 5 | ⬜ |
| Activities | 2 | ⬜ |
| Attachments | 5 | ⬜ |
| Notifications | 5 | ⬜ |
| Relations | 5 | ⬜ |
| Custom Fields | 1 | ⬜ |
| Custom Actions | 3 | ⬜ |
| Root/Config | 2 | ⬜ |
| Grids/Boards | 5+ | ⬜ |
| Budgets | 2+ | ⬜ |
| Meetings | 5+ | ⬜ |
| Storages | 10+ | ⬜ |
| **Total** | **~150** | ⬜ |

---

## VI. HAL+JSON Representers

Each API response uses HAL (Hypertext Application Language) format.

### Representer Pattern

```ruby
# Ruby (Roar)
class API::V3::WorkPackages::WorkPackageRepresenter < Roar::Decorator
  include Roar::JSON::HAL

  property :id
  property :subject

  link :self do
    { href: api_v3_paths.work_package(represented.id) }
  end
end
```

```rust
// Rust
impl HalRepresentable for WorkPackage {
    fn hal_type(&self) -> &'static str { "WorkPackage" }
    fn self_href(&self) -> String {
        format!("/api/v3/work_packages/{}", self.id.unwrap_or(0))
    }
}
```

### Representers to Port

| Representer | Ruby | Rust | Status |
|-------------|------|------|--------|
| WorkPackage | `WorkPackageRepresenter` | `representers::WorkPackage` | ⬜ |
| Project | `ProjectRepresenter` | `representers::Project` | ⬜ |
| User | `UserRepresenter` | `representers::User` | ⬜ |
| Status | `StatusRepresenter` | `representers::Status` | ⬜ |
| Type | `TypeRepresenter` | `representers::Type` | ⬜ |
| Priority | `PriorityRepresenter` | `representers::Priority` | ⬜ |
| Version | `VersionRepresenter` | `representers::Version` | ⬜ |
| Member | `MemberRepresenter` | `representers::Member` | ⬜ |
| Role | `RoleRepresenter` | `representers::Role` | ⬜ |
| Query | `QueryRepresenter` | `representers::Query` | ⬜ |
| Activity | `ActivityRepresenter` | `representers::Activity` | ⬜ |
| Attachment | `AttachmentRepresenter` | `representers::Attachment` | ⬜ |
| Notification | `NotificationRepresenter` | `representers::Notification` | ⬜ |
| TimeEntry | `TimeEntryRepresenter` | `representers::TimeEntry` | ⬜ |
| Collection | `CollectionRepresenter` | `representers::Collection<T>` | ⬜ |
| Error | `ErrorRepresenter` | `representers::Error` | ⬜ |
| Schema | `SchemaRepresenter` | `representers::Schema` | ⬜ |
| Form | `FormRepresenter` | `representers::Form` | ⬜ |

---

## VII. Database Schema

### Key Tables (~100 tables)

| Category | Tables |
|----------|--------|
| Users | users, user_passwords, user_preferences, sessions, tokens |
| Projects | projects, project_statuses, enabled_modules |
| Work Packages | work_packages, statuses, types, priorities, versions, categories, relations |
| Members | members, member_roles, roles, role_permissions |
| Journals | journals, customizable_journals, attachable_journals |
| Custom Fields | custom_fields, custom_values, custom_options |
| Attachments | attachments, attachment_journals |
| Notifications | notifications, notification_settings, reminder_notifications |
| Queries | queries, views |
| Time/Costs | time_entries, cost_entries, cost_types, labor_budget_items, material_budget_items |
| Wiki | wikis, wiki_pages, wiki_contents |
| OAuth | oauth_applications, oauth_access_tokens, oauth_access_grants |
| Storage | storages, project_storages, file_links |
| Webhooks | webhooks, webhook_logs |
| Settings | settings |
| Workflows | workflows |

### Migration Strategy

Rust version should work with existing PostgreSQL database. No schema changes.

1. Use SQLx with compile-time query checking
2. Map existing tables to Rust structs
3. Preserve all column names and types
4. Handle nullable columns with `Option<T>`
5. Use `i64` for all ID columns (match Rails bigint)

---

## VIII. Authentication & Authorization

### Authentication Methods

| Method | Ruby Implementation | Rust Implementation |
|--------|---------------------|---------------------|
| Session | `ActionController::Session` | `tower-sessions` + cookies |
| Basic Auth | API key header | `Authorization: Basic` header |
| OAuth 2.0 | Doorkeeper | `oauth2` crate |
| LDAP | `LdapAuthSource` | `ldap3` crate |
| OIDC | `OpenIDConnect` | `openidconnect` crate |

### Authorization (Permissions)

Ruby uses a declarative permission system:

```ruby
# config/initializers/permissions.rb
OpenProject::AccessControl.map do |map|
  map.permission :view_work_packages, { work_packages: [:index, :show] }
  map.permission :add_work_packages, { work_packages: [:new, :create] }
  # ...
end
```

Rust equivalent:

```rust
// op-auth/src/permissions.rs
pub enum Permission {
    ViewWorkPackages,
    AddWorkPackages,
    EditWorkPackages,
    DeleteWorkPackages,
    // ... ~100 permissions
}

impl CurrentUser {
    pub fn allowed_in_project(&self, perm: Permission, project_id: Id) -> bool;
    pub fn allowed_globally(&self, perm: Permission) -> bool;
}
```

---

## IX. DTOs & Request/Response Types

### Create DTOs

| Entity | Ruby Params | Rust DTO |
|--------|-------------|----------|
| WorkPackage | `work_package_params` | `CreateWorkPackageDto` |
| Project | `project_params` | `CreateProjectDto` |
| User | `user_params` | `CreateUserDto` |
| Member | `member_params` | `CreateMemberDto` |
| TimeEntry | `time_entry_params` | `CreateTimeEntryDto` |
| Attachment | `attachment_params` | `CreateAttachmentDto` |
| Query | `query_params` | `CreateQueryDto` |

### Update DTOs

Same pattern with `Update*Dto` variants. Key difference: all fields are `Option<T>`.

### HAL Response Wrappers

```rust
pub struct HalResource<T> {
    #[serde(flatten)]
    pub inner: T,
    #[serde(rename = "_type")]
    pub hal_type: String,
    #[serde(rename = "_links")]
    pub links: HalLinks,
    #[serde(rename = "_embedded", skip_serializing_if = "Option::is_none")]
    pub embedded: Option<serde_json::Value>,
}

pub struct HalCollection<T> {
    #[serde(rename = "_type")]
    pub hal_type: String,
    pub total: i64,
    pub count: i64,
    #[serde(rename = "_embedded")]
    pub embedded: EmbeddedElements<T>,
    #[serde(rename = "_links")]
    pub links: HalLinks,
}
```

---

## X. Background Jobs

| Ruby Job | Rust Implementation | Priority |
|----------|---------------------|----------|
| `Mails::*` | `tokio::spawn` + SMTP | High |
| `Notifications::*` | `op_notifications::workers` | High |
| `Journals::CompletedJob` | `op_journals::workers` | Medium |
| `Attachments::CleanupJob` | `op_attachments::workers` | Low |
| `Projects::CopyJob` | `op_projects::workers` | Medium |
| `WorkPackages::ExportJob` | `op_work_packages::workers` | Low |
| `Webhooks::DeliverJob` | `op_webhooks::workers` | High |

---

## XI. Implementation Priority

### Phase 1: Core (Current)
1. ✅ op-core (traits, errors, types)
2. 🟡 op-models (User, WorkPackage defined)
3. 🟡 op-contracts (base defined)
4. ⬜ op-db (SQLx setup)

### Phase 2: Work Packages
1. ⬜ Complete WorkPackage model
2. ⬜ WorkPackage contracts
3. ⬜ WorkPackage services
4. ⬜ WorkPackage API endpoints
5. ⬜ WorkPackage representer

### Phase 3: Projects
1. ⬜ Project model
2. ⬜ Project contracts
3. ⬜ Project services
4. ⬜ Project API endpoints

### Phase 4: Users & Auth
1. ⬜ Complete User model
2. ⬜ Authentication middleware
3. ⬜ User services
4. ⬜ User API endpoints

### Phase 5: Remaining Resources
1. ⬜ Members, Roles
2. ⬜ Queries
3. ⬜ Time Entries
4. ⬜ Notifications
5. ⬜ Attachments

---

## XII. Code Reuse Opportunities

### From Ruby Codebase

1. **Validation Rules**: Extract from contracts → implement in Rust contracts
2. **Permission Definitions**: Extract from `config/initializers/permissions.rb`
3. **API Response Structures**: Extract from representers → implement as Serialize
4. **Query Filters**: Extract filter logic → implement in query builders
5. **Business Rules**: Extract from services → implement in Rust services

### Shared Logic

| Logic | Ruby Location | Reuse Strategy |
|-------|---------------|----------------|
| Derived dates (WP scheduling) | `WorkPackages::Shared::*` | Port algorithm |
| Ancestry (hierarchy) | `acts_as_tree`, `ancestry` | Use `closure_tree` pattern |
| Slugs/identifiers | `FriendlyId` | Implement similar trait |
| Versioned history | Journals | Implement journaling service |
| Full-text search | `PgSearch` | Use PostgreSQL tsvector |

---

*Generated for OpenProject RS port tracking*
*Last updated: 2026-01-30*
