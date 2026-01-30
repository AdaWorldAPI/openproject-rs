# API Port Status

## Overview

OpenProject exposes a comprehensive REST API (API v3) using HAL+JSON format.
The API is built with Grape (Ruby framework) and uses Roar for HAL representation.

## API v3 Endpoints

### Core Resources

#### Work Packages
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/work_packages` | GET | `list_work_packages` | ⬜ | Collection |
| `/api/v3/work_packages` | POST | `create_work_package` | ⬜ | Create |
| `/api/v3/work_packages/:id` | GET | `get_work_package` | ⬜ | Show |
| `/api/v3/work_packages/:id` | PATCH | `update_work_package` | ⬜ | Update |
| `/api/v3/work_packages/:id` | DELETE | `delete_work_package` | ⬜ | Delete |
| `/api/v3/work_packages/:id/form` | POST | `work_package_form` | ⬜ | Form schema |
| `/api/v3/work_packages/:id/activities` | GET | `work_package_activities` | ⬜ | Journal/comments |
| `/api/v3/work_packages/:id/relations` | GET | `work_package_relations` | ⬜ | |
| `/api/v3/work_packages/:id/revisions` | GET | `work_package_revisions` | ⬜ | Git commits |
| `/api/v3/work_packages/:id/watchers` | GET | `work_package_watchers` | ⬜ | |
| `/api/v3/work_packages/schemas/:id` | GET | `work_package_schema` | ⬜ | |

#### Projects
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/projects` | GET | `list_projects` | ⬜ | Collection |
| `/api/v3/projects` | POST | `create_project` | ⬜ | Create |
| `/api/v3/projects/:id` | GET | `get_project` | ⬜ | Show |
| `/api/v3/projects/:id` | PATCH | `update_project` | ⬜ | Update |
| `/api/v3/projects/:id` | DELETE | `delete_project` | ⬜ | Delete |
| `/api/v3/projects/:id/form` | POST | `project_form` | ⬜ | Form schema |
| `/api/v3/projects/:id/work_packages` | GET | `project_work_packages` | ⬜ | |
| `/api/v3/projects/:id/versions` | GET | `project_versions` | ⬜ | |
| `/api/v3/projects/:id/types` | GET | `project_types` | ⬜ | |
| `/api/v3/projects/:id/categories` | GET | `project_categories` | ⬜ | |
| `/api/v3/projects/available_parent_projects` | GET | `available_parents` | ⬜ | |

#### Users
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/users` | GET | `list_users` | ⬜ | Collection |
| `/api/v3/users` | POST | `create_user` | ⬜ | Create |
| `/api/v3/users/:id` | GET | `get_user` | ⬜ | Show |
| `/api/v3/users/:id` | PATCH | `update_user` | ⬜ | Update |
| `/api/v3/users/:id` | DELETE | `delete_user` | ⬜ | Delete |
| `/api/v3/users/me` | GET | `current_user` | ⬜ | Self |

#### Principals (Users + Groups)
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/principals` | GET | `list_principals` | ⬜ | Collection |

#### Groups
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/groups` | GET | `list_groups` | ⬜ | Collection |
| `/api/v3/groups` | POST | `create_group` | ⬜ | Create |
| `/api/v3/groups/:id` | GET | `get_group` | ⬜ | Show |
| `/api/v3/groups/:id` | PATCH | `update_group` | ⬜ | Update |
| `/api/v3/groups/:id` | DELETE | `delete_group` | ⬜ | Delete |

#### Memberships
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/memberships` | GET | `list_memberships` | ⬜ | Collection |
| `/api/v3/memberships` | POST | `create_membership` | ⬜ | Create |
| `/api/v3/memberships/:id` | GET | `get_membership` | ⬜ | Show |
| `/api/v3/memberships/:id` | PATCH | `update_membership` | ⬜ | Update |
| `/api/v3/memberships/:id` | DELETE | `delete_membership` | ⬜ | Delete |

#### Roles
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/roles` | GET | `list_roles` | ⬜ | Collection |
| `/api/v3/roles/:id` | GET | `get_role` | ⬜ | Show |

#### Statuses
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/statuses` | GET | `list_statuses` | ⬜ | Collection |
| `/api/v3/statuses/:id` | GET | `get_status` | ⬜ | Show |

#### Types
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/types` | GET | `list_types` | ⬜ | Collection |
| `/api/v3/types/:id` | GET | `get_type` | ⬜ | Show |

#### Priorities
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/priorities` | GET | `list_priorities` | ⬜ | Collection |
| `/api/v3/priorities/:id` | GET | `get_priority` | ⬜ | Show |

#### Versions
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/versions` | GET | `list_versions` | ⬜ | Collection |
| `/api/v3/versions` | POST | `create_version` | ⬜ | Create |
| `/api/v3/versions/:id` | GET | `get_version` | ⬜ | Show |
| `/api/v3/versions/:id` | PATCH | `update_version` | ⬜ | Update |
| `/api/v3/versions/:id` | DELETE | `delete_version` | ⬜ | Delete |

#### Queries
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/queries` | GET | `list_queries` | ⬜ | Collection |
| `/api/v3/queries` | POST | `create_query` | ⬜ | Create |
| `/api/v3/queries/:id` | GET | `get_query` | ⬜ | Show |
| `/api/v3/queries/:id` | PATCH | `update_query` | ⬜ | Update |
| `/api/v3/queries/:id` | DELETE | `delete_query` | ⬜ | Delete |
| `/api/v3/queries/default` | GET | `default_query` | ⬜ | |
| `/api/v3/queries/available_projects` | GET | `query_projects` | ⬜ | |

#### Time Entries
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/time_entries` | GET | `list_time_entries` | ⬜ | Collection |
| `/api/v3/time_entries` | POST | `create_time_entry` | ⬜ | Create |
| `/api/v3/time_entries/:id` | GET | `get_time_entry` | ⬜ | Show |
| `/api/v3/time_entries/:id` | PATCH | `update_time_entry` | ⬜ | Update |
| `/api/v3/time_entries/:id` | DELETE | `delete_time_entry` | ⬜ | Delete |

#### Activities
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/activities/:id` | GET | `get_activity` | ⬜ | Journal entry |
| `/api/v3/activities/:id` | PATCH | `update_activity` | ⬜ | Edit comment |

#### Attachments
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/attachments` | POST | `create_attachment` | ⬜ | Upload |
| `/api/v3/attachments/:id` | GET | `get_attachment` | ⬜ | Show |
| `/api/v3/attachments/:id` | DELETE | `delete_attachment` | ⬜ | Delete |
| `/api/v3/attachments/:id/content` | GET | `attachment_content` | ⬜ | Download |
| `/api/v3/attachments/prepare` | POST | `prepare_attachment` | ⬜ | Direct upload |

#### Notifications
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/notifications` | GET | `list_notifications` | ⬜ | Collection |
| `/api/v3/notifications/:id` | GET | `get_notification` | ⬜ | Show |
| `/api/v3/notifications/:id/read_ian` | POST | `mark_read` | ⬜ | Mark read |
| `/api/v3/notifications/:id/unread_ian` | POST | `mark_unread` | ⬜ | Mark unread |
| `/api/v3/notifications/read_ian` | POST | `mark_all_read` | ⬜ | Bulk read |

#### Relations
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/relations` | GET | `list_relations` | ⬜ | Collection |
| `/api/v3/relations` | POST | `create_relation` | ⬜ | Create |
| `/api/v3/relations/:id` | GET | `get_relation` | ⬜ | Show |
| `/api/v3/relations/:id` | PATCH | `update_relation` | ⬜ | Update |
| `/api/v3/relations/:id` | DELETE | `delete_relation` | ⬜ | Delete |

#### Custom Fields
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/custom_fields` | GET | `list_custom_fields` | ⬜ | Collection |

#### Custom Actions
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/custom_actions` | GET | `list_custom_actions` | ⬜ | Collection |
| `/api/v3/custom_actions/:id` | GET | `get_custom_action` | ⬜ | Show |
| `/api/v3/custom_actions/:id/execute` | POST | `execute_custom_action` | ⬜ | Execute |

#### Root
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3` | GET | `api_root` | ⬜ | API info |
| `/api/v3/configuration` | GET | `api_configuration` | ⬜ | Settings |

### Module-specific Endpoints

#### Boards (Kanban)
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/grids` | GET | `list_grids` | ⬜ | Board grids |
| `/api/v3/grids/:id` | * | Various | ⬜ | CRUD |

#### Budgets
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/budgets` | GET | `list_budgets` | ⬜ | Collection |
| `/api/v3/budgets/:id` | GET | `get_budget` | ⬜ | Show |

#### Meetings
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/meetings` | GET | `list_meetings` | ⬜ | Collection |
| `/api/v3/meetings/:id` | * | Various | ⬜ | CRUD |

#### Storages
| Endpoint | Method | Rust Handler | Status | Notes |
|----------|--------|--------------|--------|-------|
| `/api/v3/storages` | GET | `list_storages` | ⬜ | Collection |
| `/api/v3/storages/:id` | * | Various | ⬜ | CRUD |
| `/api/v3/file_links` | * | Various | ⬜ | External files |

## HAL+JSON Representers

Each endpoint requires a HAL representer to format responses.

| Ruby Representer | Rust Module | Status | Notes |
|-----------------|-------------|--------|-------|
| `WorkPackageRepresenter` | `op-api::representers::WorkPackage` | ⬜ | |
| `ProjectRepresenter` | `op-api::representers::Project` | ⬜ | |
| `UserRepresenter` | `op-api::representers::User` | ⬜ | |
| `StatusRepresenter` | `op-api::representers::Status` | ⬜ | |
| `TypeRepresenter` | `op-api::representers::Type` | ⬜ | |
| `PriorityRepresenter` | `op-api::representers::Priority` | ⬜ | |
| `VersionRepresenter` | `op-api::representers::Version` | ⬜ | |
| `MemberRepresenter` | `op-api::representers::Member` | ⬜ | |
| `RoleRepresenter` | `op-api::representers::Role` | ⬜ | |
| `QueryRepresenter` | `op-api::representers::Query` | ⬜ | |
| `ActivityRepresenter` | `op-api::representers::Activity` | ⬜ | |
| `AttachmentRepresenter` | `op-api::representers::Attachment` | ⬜ | |
| `NotificationRepresenter` | `op-api::representers::Notification` | ⬜ | |
| `RelationRepresenter` | `op-api::representers::Relation` | ⬜ | |
| `TimeEntryRepresenter` | `op-api::representers::TimeEntry` | ⬜ | |
| `CollectionRepresenter` | `op-api::representers::Collection` | ⬜ | Generic |
| `ErrorRepresenter` | `op-api::representers::Error` | ⬜ | |

## Form Schemas

OpenProject uses form schemas to describe available fields for create/update.

| Schema | Rust Module | Status | Notes |
|--------|-------------|--------|-------|
| `WorkPackageSchema` | `op-api::schemas::WorkPackage` | ⬜ | |
| `ProjectSchema` | `op-api::schemas::Project` | ⬜ | |
| `UserSchema` | `op-api::schemas::User` | ⬜ | |

## Authentication

| Method | Rust Module | Status | Notes |
|--------|-------------|--------|-------|
| Basic Auth | `op-auth::basic` | ⬜ | API key |
| OAuth 2.0 | `op-auth::oauth` | ⬜ | Bearer token |
| Session | `op-auth::session` | ⬜ | Cookie-based |

## Progress Summary

- Total Endpoints: ~150+
- Completed: 0
- In Progress: 0
- Not Started: 150+
