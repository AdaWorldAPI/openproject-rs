# Models Port Status

## Core Domain Models

### Users & Authentication
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `User` | `op-models::user::User` | 🟡 | Base model defined |
| `Group` | `op-models::user::Group` | ⬜ | STI subclass |
| `PlaceholderUser` | `op-models::user::PlaceholderUser` | ⬜ | STI subclass |
| `Principal` | `op-models::user::Principal` | ⬜ | Base STI class |
| `UserPreference` | `op-models::user::UserPreference` | ⬜ | |
| `UserPassword` | `op-models::user::UserPassword` | ⬜ | Password history |
| `Session` | `op-models::user::Session` | ⬜ | |

### Projects
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Project` | `op-models::project::Project` | ⬜ | |
| `ProjectStatus` | `op-models::project::ProjectStatus` | ⬜ | |
| `ProjectCustomField` | `op-models::project::ProjectCustomField` | ⬜ | |
| `EnabledModule` | `op-models::project::EnabledModule` | ⬜ | |

### Work Packages
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `WorkPackage` | `op-models::work_package::WorkPackage` | 🟢 | Core model |
| `Status` | `op-models::status::Status` | ⬜ | |
| `Type` | `op-models::type_def::Type` | ⬜ | Work package type |
| `Priority` | `op-models::priority::Priority` | ⬜ | |
| `Version` | `op-models::version::Version` | ⬜ | |
| `Category` | `op-models::category::Category` | ⬜ | |
| `Relation` | `op-models::relation::Relation` | ⬜ | WP relationships |

### Memberships & Roles
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Member` | `op-models::member::Member` | ⬜ | |
| `MemberRole` | `op-models::member::MemberRole` | ⬜ | |
| `Role` | `op-models::role::Role` | ⬜ | |
| `RolePermission` | `op-models::role::RolePermission` | ⬜ | |

### Journals & Activity
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Journal` | `op-models::journal::Journal` | ⬜ | Audit log |
| `Journal::*Data` | `op-models::journal::*Data` | ⬜ | Per-entity data |
| `Activity` | `op-models::activity::Activity` | ⬜ | |

### Attachments & Files
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Attachment` | `op-models::attachment::Attachment` | ⬜ | |
| `Container` | `op-models::attachment::Container` | ⬜ | Polymorphic |
| `FileLink` | `op-models::file_link::FileLink` | ⬜ | External files |

### Custom Fields
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `CustomField` | `op-models::custom_field::CustomField` | ⬜ | Base class |
| `CustomValue` | `op-models::custom_value::CustomValue` | ⬜ | |
| `*CustomField` | Various | ⬜ | STI subclasses |

### Notifications
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Notification` | `op-models::notification::Notification` | ⬜ | |
| `NotificationSetting` | `op-models::notification::NotificationSetting` | ⬜ | |
| `ReminderNotification` | `op-models::notification::ReminderNotification` | ⬜ | |

### Queries & Views
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Query` | `op-models::query::Query` | ⬜ | Saved queries |
| `View` | `op-models::query::View` | ⬜ | |
| `QueryFilter` | `op-models::query::QueryFilter` | ⬜ | |

### Time & Costs
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `TimeEntry` | `op-models::time_entry::TimeEntry` | ⬜ | |
| `TimeEntryActivity` | `op-models::time_entry::TimeEntryActivity` | ⬜ | |
| `CostEntry` | `op-models::cost::CostEntry` | ⬜ | |
| `CostType` | `op-models::cost::CostType` | ⬜ | |
| `Budget` | `op-models::cost::Budget` | ⬜ | |

### Wiki & Documents
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Wiki` | `op-models::wiki::Wiki` | ⬜ | |
| `WikiPage` | `op-models::wiki::WikiPage` | ⬜ | |
| `WikiContent` | `op-models::wiki::WikiContent` | ⬜ | |
| `Document` | `op-models::document::Document` | ⬜ | |

### Meetings
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Meeting` | `op-models::meeting::Meeting` | ⬜ | |
| `MeetingAgenda` | `op-models::meeting::MeetingAgenda` | ⬜ | |
| `MeetingMinutes` | `op-models::meeting::MeetingMinutes` | ⬜ | |

### News & Forums
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `News` | `op-models::news::News` | ⬜ | |
| `Forum` | `op-models::forum::Forum` | ⬜ | |
| `Message` | `op-models::forum::Message` | ⬜ | Forum posts |

### OAuth & Authentication
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Doorkeeper::Application` | `op-models::oauth::Application` | ⬜ | OAuth apps |
| `Doorkeeper::AccessToken` | `op-models::oauth::AccessToken` | ⬜ | |
| `Doorkeeper::AccessGrant` | `op-models::oauth::AccessGrant` | ⬜ | |
| `LdapAuthSource` | `op-models::ldap::LdapAuthSource` | ⬜ | |
| `Token::*` | `op-models::token::*` | ⬜ | Various tokens |

### Webhooks
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Webhook` | `op-models::webhook::Webhook` | ⬜ | |
| `WebhookLog` | `op-models::webhook::WebhookLog` | ⬜ | |

### Storage (Nextcloud, etc.)
| Ruby Model | Rust Module | Status | Notes |
|------------|-------------|--------|-------|
| `Storage` | `op-models::storage::Storage` | ⬜ | |
| `ProjectStorage` | `op-models::storage::ProjectStorage` | ⬜ | |
| `StorageFile` | `op-models::storage::StorageFile` | ⬜ | |

## Progress Summary

- Total Models: ~80+
- Completed: 2 (User, WorkPackage)
- In Progress: 0
- Not Started: 78+

## Notes

The Ruby codebase uses:
- Single Table Inheritance (STI) for User/Group/PlaceholderUser
- Polymorphic associations for Attachments, CustomValues
- Complex validation via Contracts (separate from model validations)
- ActiveRecord callbacks (need to handle in Rust services)
