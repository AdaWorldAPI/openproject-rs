# Sprint Review: OpenProject 1:1 Rust Transcoding

## Current State

| Layer | Status | Notes |
|-------|--------|-------|
| **op-core** | Done | Foundation types, traits, config |
| **op-models** | Done | 9 domain models (WorkPackage, User, Project, Status, Priority, Type, Version, Member, Role) |
| **op-db repositories** | Partial | 4 of 12+ needed (WorkPackage, User, Project, TimeEntry) |
| **op-api handlers** | Partial | WorkPackages wired, rest are mocks |
| **op-services** | Framework | Architecture defined, implementations reference files but not wired |
| **op-contracts** | Framework | Validation pattern defined |

## Sprint Scope: Internal Code Only (No External Tests)

All tasks below can be validated via `cargo check` + `cargo test` without requiring external database/integration tests.

---

## Priority 1: Wire Existing Repositories to API Handlers

These repositories already exist in op-db but handlers return mock data.

### 1.1 Wire Projects Handler → ProjectRepository
- **File**: `crates/op-api/src/handlers/projects.rs`
- **Repo**: `crates/op-db/src/projects.rs` (already complete)
- **Work**:
  - Replace mock responses with `ProjectRepository::find_*` calls
  - Map `ProjectRow` → `ProjectResponse` with full HAL+JSON
  - Add `description`, `parent_id`, `created_at`, `updated_at` to response
  - Use `AppState::pool()` for database access
- **Pattern**: Copy from `work_packages.rs` handler

### 1.2 Wire Users Handler → UserRepository
- **File**: `crates/op-api/src/handlers/users.rs`
- **Repo**: `crates/op-db/src/users.rs` (already complete)
- **Work**:
  - Replace mock `list_users` with `UserRepository::find_all`
  - Replace mock `get_user` with `UserRepository::find_by_id`
  - Wire `create_user` → `UserRepository::create`
  - Wire `update_user` → `UserRepository::update`
  - Wire `delete_user` → `UserRepository::delete`
  - Password hashing via `op-auth` integration
- **Pattern**: Copy from `work_packages.rs` handler

---

## Priority 2: Create Missing Lookup Table Repositories

These are simple read-mostly tables. Create repositories + wire handlers.

### 2.1 StatusRepository
- **Tables**: `statuses` (id, name, is_closed, is_default, is_readonly, position, color_id, default_done_ratio)
- **Ruby model**: `app/models/status.rb`
- **New file**: `crates/op-db/src/statuses.rs`
- **Work**:
  - Create `StatusRow` matching schema
  - Implement `find_by_id`, `find_all`, `find_default`
  - Wire `handlers/statuses.rs` (remove hardcoded array)
  - Read-only (no create/update/delete needed for MVP)

### 2.2 PriorityRepository
- **Tables**: `enumerations` WHERE type = 'IssuePriority'
- **Ruby model**: `app/models/issue_priority.rb`
- **New file**: `crates/op-db/src/priorities.rs`
- **Work**:
  - Query `enumerations` table with type filter
  - Create `PriorityRow` (id, name, position, is_default, active, color_id)
  - Wire `handlers/priorities.rs`

### 2.3 TypeRepository
- **Tables**: `types` (id, name, position, is_default, is_in_roadmap, is_milestone, color_id, description, attribute_groups)
- **Ruby model**: `app/models/type.rb`
- **New file**: `crates/op-db/src/types.rs`
- **Work**:
  - Create `TypeRow` matching schema
  - Implement `find_by_id`, `find_all`, `find_by_project`
  - Wire `handlers/types.rs`

### 2.4 RoleRepository
- **Tables**: `roles` (id, name, position, assignable, builtin, type)
- **Ruby model**: `app/models/role.rb`
- **New file**: `crates/op-db/src/roles.rs`
- **Work**:
  - Create `RoleRow` with permissions from `role_permissions` join
  - Implement `find_by_id`, `find_all`, `find_assignable`
  - New handler: `handlers/roles.rs`
  - Route: `/api/v3/roles`

### 2.5 VersionRepository
- **Tables**: `versions` (id, project_id, name, description, effective_date, status, sharing, wiki_page_title)
- **Ruby model**: `app/models/version.rb`
- **New file**: `crates/op-db/src/versions.rs`
- **Work**:
  - Create `VersionRow` matching schema
  - Implement `find_by_id`, `find_all`, `find_by_project`, `find_open`
  - New handler: `handlers/versions.rs`
  - Route: `/api/v3/versions`

---

## Priority 3: Core Business Entity Repositories

### 3.1 MemberRepository
- **Tables**: `members` (id, project_id, user_id) + `member_roles` (member_id, role_id)
- **Ruby model**: `app/models/member.rb`
- **New file**: `crates/op-db/src/members.rs`
- **Work**:
  - Create `MemberRow` with role_ids from join
  - Implement `find_by_project`, `find_by_user`, `find_by_project_and_user`
  - New handler: `handlers/memberships.rs`
  - Route: `/api/v3/memberships`

### 3.2 ActivityRepository (for time tracking)
- **Tables**: `enumerations` WHERE type = 'TimeEntryActivity'
- **Ruby model**: `app/models/time_entry_activity.rb`
- **New file**: `crates/op-db/src/activities.rs`
- **Work**:
  - Query `enumerations` table with type filter
  - Create `ActivityRow` (id, name, position, is_default, active)
  - New handler: `handlers/activities.rs`
  - Route: `/api/v3/time_entries/activities`

### 3.3 CategoryRepository
- **Tables**: `categories` (id, project_id, name, assigned_to_id)
- **Ruby model**: `app/models/category.rb`
- **New file**: `crates/op-db/src/categories.rs`
- **Work**:
  - Create `CategoryRow` matching schema
  - Implement `find_by_id`, `find_by_project`
  - New handler: `handlers/categories.rs`
  - Route: `/api/v3/categories`

---

## Priority 4: Relational Features

### 4.1 RelationRepository
- **Tables**: `relations` (id, from_id, to_id, relation_type, delay, description)
- **Ruby model**: `app/models/relation.rb`
- **New file**: `crates/op-db/src/relations.rs`
- **Work**:
  - Create `RelationRow` (from_id, to_id, relation_type: follows/blocks/duplicates/etc)
  - Implement `find_by_work_package`, `find_by_from`, `find_by_to`
  - New handler: `handlers/relations.rs`
  - Route: `/api/v3/relations`

### 4.2 WatcherRepository
- **Tables**: `watchers` (id, watchable_type, watchable_id, user_id)
- **Ruby model**: `app/models/watcher.rb`
- **New file**: `crates/op-db/src/watchers.rs`
- **Work**:
  - Create `WatcherRow` matching schema
  - Implement `find_by_work_package`, `add_watcher`, `remove_watcher`
  - Add to work_package handlers: `GET/POST/DELETE /api/v3/work_packages/:id/watchers`

### 4.3 AttachmentRepository
- **Tables**: `attachments` (id, container_id, container_type, filename, disk_filename, filesize, content_type, digest, author_id)
- **Ruby model**: `app/models/attachment.rb`
- **New file**: `crates/op-db/src/attachments.rs`
- **Work**:
  - Create `AttachmentRow` matching schema
  - Implement `find_by_container`, `create`, `delete`
  - Wire existing `op-attachments` crate to storage
  - Handler: `handlers/attachments.rs`
  - Routes: `/api/v3/attachments`, `/api/v3/work_packages/:id/attachments`

---

## Priority 5: Query System

### 5.1 QueryRepository
- **Tables**: `queries` (id, project_id, name, filters, column_names, sort_criteria, group_by, display_sums, is_public, user_id)
- **Ruby model**: `app/models/query.rb`
- **New file**: `crates/op-db/src/queries.rs`
- **Work**:
  - Create `QueryRow` with JSON fields for filters/columns/sort
  - Implement `find_by_id`, `find_by_project`, `find_by_user`, `find_default`
  - Wire to existing `op-queries` crate for filter building
  - Handler: `handlers/queries.rs` (replace mock)
  - Route: `/api/v3/queries`

---

## Priority 6: Audit Trail

### 6.1 JournalRepository
- **Tables**: `journals` (id, journable_type, journable_id, user_id, notes, created_at, version) + `journal_*_journals` tables
- **Ruby model**: `app/models/journal.rb`
- **New file**: `crates/op-db/src/journals.rs`
- **Work**:
  - Wire existing `op-journals` crate to database
  - Create journal entries on work_package create/update
  - Implement `find_by_work_package`
  - Add to work_package response: `_embedded.activities`

---

## Out of Scope (Requires External Tests)

These items require integration/E2E testing:

- Docker build verification
- Railway deployment
- Real database migrations
- File upload to S3/local storage
- Email sending via MS Graph
- OAuth/OIDC authentication flow
- WebSocket notifications

---

## Sprint Estimates

| Priority | Items | Complexity |
|----------|-------|------------|
| P1 | 2 | Low - repos exist, just wire handlers |
| P2 | 5 | Low - simple lookup tables |
| P3 | 3 | Medium - business entities with joins |
| P4 | 3 | Medium - relational features |
| P5 | 1 | Medium - JSON fields, filter building |
| P6 | 1 | Medium - journaling on mutations |

**Total**: 15 items

---

## Validation Criteria

Each item is complete when:

1. `cargo check` passes
2. `cargo test` passes (298+ tests)
3. Handler returns real database data (not mock JSON)
4. HAL+JSON response includes `_links` section
5. DTOs match OpenProject API v3 spec

---

## Recommended Sprint Order

```
Sprint 1: P1 (wire existing)
  ├── 1.1 Projects handler
  └── 1.2 Users handler

Sprint 2: P2 (lookup tables)
  ├── 2.1 StatusRepository
  ├── 2.2 PriorityRepository
  ├── 2.3 TypeRepository
  ├── 2.4 RoleRepository
  └── 2.5 VersionRepository

Sprint 3: P3 (business entities)
  ├── 3.1 MemberRepository
  ├── 3.2 ActivityRepository
  └── 3.3 CategoryRepository

Sprint 4: P4+P5+P6 (relations, queries, journals)
  ├── 4.1 RelationRepository
  ├── 4.2 WatcherRepository
  ├── 4.3 AttachmentRepository
  ├── 5.1 QueryRepository
  └── 6.1 JournalRepository
```

---

## Ruby Model → Rust Repository Mapping

| Ruby Model | Rust Repo | Status |
|------------|-----------|--------|
| `WorkPackage` | `WorkPackageRepository` | Done |
| `User` | `UserRepository` | Done |
| `Project` | `ProjectRepository` | Done |
| `TimeEntry` | `TimeEntryRepository` | Done |
| `Status` | `StatusRepository` | TODO |
| `IssuePriority` | `PriorityRepository` | TODO |
| `Type` | `TypeRepository` | TODO |
| `Role` | `RoleRepository` | TODO |
| `Version` | `VersionRepository` | TODO |
| `Member` | `MemberRepository` | TODO |
| `TimeEntryActivity` | `ActivityRepository` | TODO |
| `Category` | `CategoryRepository` | TODO |
| `Relation` | `RelationRepository` | TODO |
| `Watcher` | `WatcherRepository` | TODO |
| `Attachment` | `AttachmentRepository` | TODO |
| `Query` | `QueryRepository` | TODO |
| `Journal` | `JournalRepository` | TODO |

**Done**: 4 / **TODO**: 13
