# OpenProject-RS Parity Roadmap

> **Goal:** Reach functional parity with OpenProject (Ruby) in Rust
> **Scope:** ~500k lines Ruby → ~150k lines Rust (estimated)
> **Strategy:** Phased sprints with agent-sized work chunks

---

## Reference Documentation

Detailed inventories are maintained in `tracker/`:

| Document | Description |
|----------|-------------|
| [`tracker/INVENTORY.md`](./tracker/INVENTORY.md) | Complete Ruby codebase inventory |
| [`tracker/progress.md`](./tracker/progress.md) | Overall progress tracking |
| [`tracker/models/README.md`](./tracker/models/README.md) | Model port status |
| [`tracker/contracts/README.md`](./tracker/contracts/README.md) | Contract port status |
| [`tracker/services/README.md`](./tracker/services/README.md) | Service port status |
| [`tracker/api/README.md`](./tracker/api/README.md) | API endpoint port status |

---

## Current State Summary

### What's Built (openproject-rs)
- **17 crates** with clean architecture
- **~25k lines** of Rust code
- **Database layer:** 17 repositories (CRUD complete)
- **API routes:** 18 resources, ~120 endpoints defined
- **Handlers:** ~40% implemented, 60% stubs
- **Services:** Structure exists, business logic incomplete
- **Auth:** JWT/API key structure, validation stubbed

### What's Missing (Gap to Original)
- **70+ models** not yet implemented (see tracker/models/)
- **~100 services** not transcoded (see tracker/services/)
- **~45 contracts** not transcoded (see tracker/contracts/)
- **74 background jobs** not implemented
- **29 optional modules** not started
- **Custom fields, workflows, wiki, meetings, forums, etc.**

---

## Phase Overview

| Phase | Name | Scope | Est. Effort | Priority |
|-------|------|-------|-------------|----------|
| **P1** | Core Completion | Complete existing stubs | 2 weeks | CRITICAL |
| **P2** | Auth & Permissions | Real auth, RBAC enforcement | 1 week | CRITICAL |
| **P3** | Work Package Features | Relations, workflows, bulk ops | 2 weeks | HIGH |
| **P4** | Query Engine | Full filter/sort/group execution | 1 week | HIGH |
| **P5** | Custom Fields | Dynamic fields on entities | 2 weeks | HIGH |
| **P6** | Notifications | In-app + email notifications | 1 week | MEDIUM |
| **P7** | Background Jobs | Async processing infrastructure | 1 week | MEDIUM |
| **P8** | Wiki & Docs | Wiki pages, attachments | 1 week | MEDIUM |
| **P9** | Time Tracking | Time entries, cost tracking | 1 week | MEDIUM |
| **P10** | Collaboration | Forums, news, comments | 1 week | LOW |
| **P11** | Integrations | Webhooks, OAuth apps | 1 week | LOW |
| **P12** | Modules | Optional feature modules | 4 weeks | LOW |
| **P13** | Admin & Config | System settings, backups | 1 week | LOW |
| **P14** | Polish | Performance, tests, docs | 2 weeks | LOW |

**Total Estimate:** ~20 weeks for core parity

---

## Phase 1: Core Completion (CRITICAL)

**Goal:** Complete all existing stub implementations

### Sprint 1.1: Handler Implementation (3-4 days)
Complete handlers that currently return stubs or mock data.

**Chunk 1.1.1: Work Package Handlers**
```
Files: crates/op-api/src/handlers/work_packages.rs
Tasks:
- [ ] Wire list_work_packages to WorkPackageQueryExecutor
- [ ] Wire create_work_package through service layer
- [ ] Wire update_work_package with contract validation
- [ ] Wire delete_work_package with cascade logic
- [ ] Add form endpoint for work package schema
```

**Chunk 1.1.2: Project Handlers**
```
Files: crates/op-api/src/handlers/projects.rs
Tasks:
- [ ] Complete project creation with hierarchy
- [ ] Wire archive/unarchive logic
- [ ] Add copy project endpoint
- [ ] Wire project types/versions/categories
```

**Chunk 1.1.3: User Handlers**
```
Files: crates/op-api/src/handlers/users.rs
Tasks:
- [ ] Wire user creation with password hashing
- [ ] Complete lock/unlock with status transitions
- [ ] Add user preferences endpoint
- [ ] Wire user deletion (anonymization)
```

**Chunk 1.1.4: Query Handlers**
```
Files: crates/op-api/src/handlers/queries.rs
Tasks:
- [ ] Wire query execution to return real work packages
- [ ] Complete star/unstar with query_menu_items
- [ ] Add query schema endpoint
- [ ] Wire available filters/columns/sort_bys
```

### Sprint 1.2: Service Layer Completion (3-4 days)

**Chunk 1.2.1: Work Package Services**
```
Files: crates/op-services/src/work_packages/*.rs
Ruby ref: app/services/work_packages/*.rb
Tasks:
- [ ] CreateService - full validation, journaling
- [ ] UpdateService - change tracking, notifications
- [ ] DeleteService - cascade, cleanup
- [ ] SetAttributesService - attribute assignment
- [ ] CopyService - deep copy with relations
```

**Chunk 1.2.2: Project Services**
```
Files: crates/op-services/src/projects/*.rs
Ruby ref: app/services/projects/*.rb
Tasks:
- [ ] CreateService - hierarchy, defaults
- [ ] UpdateService - status changes
- [ ] ArchiveService - cascade to work packages
- [ ] CopyService - deep copy with members
```

**Chunk 1.2.3: User Services**
```
Files: crates/op-services/src/users/*.rs
Ruby ref: app/services/users/*.rb
Tasks:
- [ ] CreateService - invitation flow
- [ ] UpdateService - preference sync
- [ ] DeleteService - anonymization
- [ ] LockService - status transitions
```

### Sprint 1.3: Contract Validation (2-3 days)

**Chunk 1.3.1: Work Package Contracts**
```
Files: crates/op-contracts/src/work_packages/*.rs
Ruby ref: app/contracts/work_packages/*.rb
Tasks:
- [ ] Validate required fields (subject, type, status, project)
- [ ] Validate assignable values (status transitions)
- [ ] Validate dates (start <= due)
- [ ] Validate parent hierarchy (no cycles)
- [ ] Permission checks (view, edit, delete)
```

**Chunk 1.3.2: Project Contracts**
```
Files: crates/op-contracts/src/projects/*.rs
Ruby ref: app/contracts/projects/*.rb
Tasks:
- [ ] Validate identifier uniqueness
- [ ] Validate parent hierarchy
- [ ] Admin-only creation check
- [ ] Archive permission checks
```

---

## Phase 2: Auth & Permissions (CRITICAL)

**Goal:** Real authentication and authorization enforcement

### Sprint 2.1: Authentication (2-3 days)

**Chunk 2.1.1: JWT Validation**
```
Files: crates/op-auth/src/jwt.rs
Tasks:
- [ ] Token generation with claims
- [ ] Token validation (signature, expiry)
- [ ] Refresh token flow
- [ ] Token revocation (blocklist)
```

**Chunk 2.1.2: Session Management**
```
Files: crates/op-auth/src/session.rs
Tasks:
- [ ] Session creation on login
- [ ] Session storage (Redis/DB)
- [ ] Session validation middleware
- [ ] Session cleanup job
```

**Chunk 2.1.3: API Key Auth**
```
Files: crates/op-auth/src/api_key.rs
Tasks:
- [ ] API token generation
- [ ] Token scopes (read, write, admin)
- [ ] Token validation
- [ ] Rate limiting per token
```

### Sprint 2.2: Authorization (3-4 days)

**Chunk 2.2.1: Permission System**
```
Files: crates/op-auth/src/permissions.rs
Ruby ref: app/models/role.rb, app/models/role_permission.rb
Tasks:
- [ ] Define all permissions (from OpenProject)
- [ ] Role → permissions mapping
- [ ] Member → roles → permissions resolution
- [ ] Global permissions vs project permissions
```

**Chunk 2.2.2: Authorization Service**
```
Files: crates/op-auth/src/authorization.rs
Ruby ref: app/services/authorization/*.rb
Tasks:
- [ ] UserPermissibleService - check user can X
- [ ] allowed_to?(permission, project)
- [ ] allowed_globally?(permission)
- [ ] Cache permission results
```

**Chunk 2.2.3: Query Authorization**
```
Files: crates/op-db/src/query_executor.rs
Ruby ref: app/models/queries/work_packages/filter/*.rb
Tasks:
- [ ] Filter work packages by visibility
- [ ] Apply project membership filter
- [ ] Apply role-based filters
- [ ] Row-level security in SQL
```

---

## Phase 3: Work Package Features (HIGH)

### Sprint 3.1: Relations & Dependencies (2-3 days)

**Chunk 3.1.1: Relation Types**
```
Files: crates/op-db/src/relations.rs (extend)
Ruby ref: app/models/relation.rb
Tasks:
- [ ] All relation types (follows, precedes, blocks, etc.)
- [ ] Symmetric relations handling
- [ ] Lag time for follows/precedes
- [ ] Circular dependency detection
```

**Chunk 3.1.2: Relation Services**
```
Files: crates/op-services/src/relations/*.rs (new)
Ruby ref: app/services/relations/*.rb
Tasks:
- [ ] CreateRelationService
- [ ] DeleteRelationService
- [ ] Validate no circular dependencies
- [ ] Reschedule on dependency change
```

### Sprint 3.2: Status Workflows (2-3 days)

**Chunk 3.2.1: Workflow Model**
```
Files: crates/op-db/src/workflows.rs (new)
Ruby ref: app/models/workflow.rb
Tasks:
- [ ] WorkflowRow entity
- [ ] WorkflowRepository
- [ ] Find allowed transitions for role/type/status
```

**Chunk 3.2.2: Status Transition Validation**
```
Files: crates/op-contracts/src/work_packages/status.rs (new)
Ruby ref: app/contracts/work_packages/base_contract.rb
Tasks:
- [ ] Validate status transition allowed
- [ ] Check workflow for type/role
- [ ] Enforce on update
```

### Sprint 3.3: Bulk Operations (2-3 days)

**Chunk 3.3.1: Bulk Update**
```
Files: crates/op-api/src/handlers/work_packages_bulk.rs (new)
Ruby ref: app/services/work_packages/bulk/*.rb
Tasks:
- [ ] PATCH /api/v3/work_packages/bulk
- [ ] Bulk status change
- [ ] Bulk assignment
- [ ] Bulk priority change
```

**Chunk 3.3.2: Bulk Move/Copy**
```
Files: crates/op-services/src/work_packages/bulk.rs (new)
Ruby ref: app/workers/work_packages/*.rb
Tasks:
- [ ] Move work packages to project
- [ ] Copy work packages
- [ ] Async job for large batches
```

---

## Phase 4: Query Engine (HIGH)

### Sprint 4.1: Filter Execution (2-3 days)

**Chunk 4.1.1: Filter Types**
```
Files: crates/op-queries/src/filters/*.rs
Ruby ref: app/models/queries/filters/*.rb
Tasks:
- [ ] Text filters (subject, description contains)
- [ ] Status filter (open, closed, specific)
- [ ] Assignee filter (me, user, group, unassigned)
- [ ] Date filters (created, updated, due date ranges)
- [ ] Version filter
- [ ] Type filter
- [ ] Priority filter
- [ ] Parent/child filters
- [ ] Custom field filters
```

**Chunk 4.1.2: Filter to SQL**
```
Files: crates/op-db/src/query_executor.rs (extend)
Tasks:
- [ ] Build WHERE clause from filters
- [ ] Handle OR/AND combinations
- [ ] Parameterized queries (prevent SQL injection)
- [ ] Optimize with indexes
```

### Sprint 4.2: Sort & Group (1-2 days)

**Chunk 4.2.1: Multi-column Sort**
```
Files: crates/op-queries/src/sorts.rs (extend)
Tasks:
- [ ] Multi-column ORDER BY
- [ ] Null handling (NULLS FIRST/LAST)
- [ ] Custom field sorting
```

**Chunk 4.2.2: Grouping**
```
Files: crates/op-queries/src/grouping.rs (new)
Ruby ref: app/models/queries/group_by*.rb
Tasks:
- [ ] Group by status, assignee, type, etc.
- [ ] Aggregate counts per group
- [ ] Sums (estimated hours, story points)
```

---

## Phase 5: Custom Fields (HIGH)

### Sprint 5.1: Custom Field Infrastructure (3-4 days)

**Chunk 5.1.1: Custom Field Model**
```
Files: crates/op-db/src/custom_fields.rs (new)
Ruby ref: app/models/custom_field.rb
Tasks:
- [ ] CustomFieldRow entity
- [ ] CustomFieldRepository
- [ ] Field types (string, int, float, date, bool, list, user)
- [ ] Field formats and constraints
```

**Chunk 5.1.2: Custom Values**
```
Files: crates/op-db/src/custom_values.rs (new)
Ruby ref: app/models/custom_value.rb
Tasks:
- [ ] CustomValueRow entity
- [ ] Polymorphic association (customizable_type/id)
- [ ] Value storage (typed columns or JSON)
```

### Sprint 5.2: Custom Field Integration (3-4 days)

**Chunk 5.2.1: Work Package Custom Fields**
```
Files: crates/op-services/src/work_packages/custom_fields.rs (new)
Tasks:
- [ ] Load custom fields for work package
- [ ] Validate custom field values
- [ ] Save custom values on create/update
- [ ] Include in API response
```

**Chunk 5.2.2: Query Custom Fields**
```
Files: crates/op-queries/src/filters/custom_field.rs (new)
Tasks:
- [ ] Filter by custom field value
- [ ] Sort by custom field
- [ ] Custom field columns in results
```

---

## Phase 6: Notifications (MEDIUM)

### Sprint 6.1: Notification Infrastructure (2-3 days)

**Chunk 6.1.1: Notification Model**
```
Files: crates/op-db/src/notifications.rs (extend)
Ruby ref: app/models/notification.rb
Tasks:
- [ ] NotificationRow entity
- [ ] NotificationRepository
- [ ] Notification reasons (mentioned, assigned, watched)
- [ ] Read/unread status
```

**Chunk 6.1.2: Notification Settings**
```
Files: crates/op-db/src/notification_settings.rs (new)
Ruby ref: app/models/notification_setting.rb
Tasks:
- [ ] Per-user notification preferences
- [ ] Channel settings (in-app, email, digest)
- [ ] Project-specific overrides
```

### Sprint 6.2: Notification Delivery (2-3 days)

**Chunk 6.2.1: In-App Notifications**
```
Files: crates/op-notifications/src/in_app.rs (new)
Tasks:
- [ ] Create notifications on events
- [ ] List user notifications
- [ ] Mark as read
- [ ] Clear notifications
```

**Chunk 6.2.2: Email Notifications**
```
Files: crates/op-notifications/src/email.rs (new)
Ruby ref: app/mailers/*.rb
Tasks:
- [ ] Email template rendering
- [ ] SMTP delivery
- [ ] Digest aggregation
- [ ] Unsubscribe handling
```

---

## Phase 7: Background Jobs (MEDIUM)

### Sprint 7.1: Job Infrastructure (2-3 days)

**Chunk 7.1.1: Job Queue**
```
Files: crates/op-jobs/src/queue.rs (new crate)
Tasks:
- [ ] Job definition trait
- [ ] PostgreSQL-backed queue (or Redis)
- [ ] Job priorities
- [ ] Retry with backoff
- [ ] Dead letter queue
```

**Chunk 7.1.2: Job Workers**
```
Files: crates/op-jobs/src/worker.rs
Tasks:
- [ ] Worker pool management
- [ ] Graceful shutdown
- [ ] Health monitoring
- [ ] Metrics (jobs/sec, latency)
```

### Sprint 7.2: Core Jobs (2-3 days)

**Chunk 7.2.1: Email Jobs**
```
Files: crates/op-jobs/src/jobs/mail.rs
Tasks:
- [ ] SendEmailJob
- [ ] DigestEmailJob
- [ ] ReminderEmailJob
```

**Chunk 7.2.2: Cleanup Jobs**
```
Files: crates/op-jobs/src/jobs/cleanup.rs
Tasks:
- [ ] ClearOldSessionsJob
- [ ] ClearTempFilesJob
- [ ] ClearOrphanedAttachmentsJob
```

---

## Phase 8: Wiki & Documentation (MEDIUM)

### Sprint 8.1: Wiki Model (2-3 days)

**Chunk 8.1.1: Wiki Pages**
```
Files: crates/op-db/src/wiki_pages.rs (new)
Ruby ref: app/models/wiki_page.rb, app/models/wiki.rb
Tasks:
- [ ] WikiRow entity (per project)
- [ ] WikiPageRow entity (hierarchical)
- [ ] WikiPageRepository
- [ ] Version history
```

**Chunk 8.1.2: Wiki API**
```
Files: crates/op-api/src/handlers/wiki_pages.rs (new)
Ruby ref: lib/api/v3/wiki_pages/*.rb
Tasks:
- [ ] GET /projects/:id/wiki_pages
- [ ] GET /wiki_pages/:id
- [ ] POST/PATCH/DELETE wiki pages
- [ ] Markdown rendering
```

### Sprint 8.2: Attachments (2-3 days)

**Chunk 8.2.1: File Upload**
```
Files: crates/op-api/src/handlers/attachments.rs (extend)
Tasks:
- [ ] Multipart upload parsing
- [ ] Direct upload (presigned URLs)
- [ ] File type validation
- [ ] Size limits
```

**Chunk 8.2.2: File Storage**
```
Files: crates/op-attachments/src/storage/*.rs
Tasks:
- [ ] Local filesystem storage
- [ ] S3-compatible storage
- [ ] Attachment thumbnails
- [ ] File download with auth
```

---

## Phase 9: Time Tracking (MEDIUM)

### Sprint 9.1: Time Entries (2-3 days)

**Chunk 9.1.1: Time Entry Features**
```
Files: crates/op-db/src/time_entries.rs (extend)
Ruby ref: app/models/time_entry.rb
Tasks:
- [ ] Activity types (design, development, etc.)
- [ ] Billable flag
- [ ] Comments
- [ ] Date validation
```

**Chunk 9.1.2: Time Entry Services**
```
Files: crates/op-services/src/time_entries/*.rs (new)
Tasks:
- [ ] CreateTimeEntryService
- [ ] UpdateTimeEntryService
- [ ] Validation (hours > 0, valid date)
- [ ] Permission checks
```

### Sprint 9.2: Cost Tracking (2-3 days)

**Chunk 9.2.1: Cost Types**
```
Files: crates/op-db/src/cost_types.rs (new)
Ruby ref: modules/costs/app/models/*.rb
Tasks:
- [ ] CostTypeRow entity
- [ ] CostEntryRow entity
- [ ] Hourly rates
```

**Chunk 9.2.2: Cost Rollups**
```
Files: crates/op-services/src/costs/*.rs (new)
Tasks:
- [ ] Calculate work package costs
- [ ] Project cost summaries
- [ ] Budget tracking
```

---

## Phase 10: Collaboration (LOW)

### Sprint 10.1: Forums & Messages (2-3 days)

**Chunk 10.1.1: Forum Model**
```
Files: crates/op-db/src/forums.rs (new)
Ruby ref: app/models/forum.rb, app/models/message.rb
Tasks:
- [ ] ForumRow entity
- [ ] MessageRow entity (threaded)
- [ ] ForumRepository, MessageRepository
```

**Chunk 10.1.2: Forum API**
```
Files: crates/op-api/src/handlers/messages.rs (new)
Tasks:
- [ ] GET /projects/:id/forums
- [ ] GET /forums/:id/messages
- [ ] POST/PATCH/DELETE messages
```

### Sprint 10.2: News & Comments (2-3 days)

**Chunk 10.2.1: News**
```
Files: crates/op-db/src/news.rs (new)
Ruby ref: app/models/news.rb
Tasks:
- [ ] NewsRow entity
- [ ] NewsRepository
- [ ] GET/POST/PATCH/DELETE endpoints
```

**Chunk 10.2.2: Comments**
```
Files: crates/op-db/src/comments.rs (new)
Ruby ref: app/models/comment.rb
Tasks:
- [ ] CommentRow entity (polymorphic)
- [ ] CommentRepository
- [ ] Add comments to news, wiki pages
```

---

## Phase 11: Integrations (LOW)

### Sprint 11.1: Webhooks (2-3 days)

**Chunk 11.1.1: Webhook Model**
```
Files: crates/op-db/src/webhooks.rs (new)
Ruby ref: modules/webhooks/app/models/*.rb
Tasks:
- [ ] WebhookRow entity
- [ ] WebhookRepository
- [ ] Event subscriptions
```

**Chunk 11.1.2: Webhook Delivery**
```
Files: crates/op-webhooks/src/*.rs (new crate)
Tasks:
- [ ] Event triggering
- [ ] HTTP POST delivery
- [ ] Retry logic
- [ ] Signature verification
```

### Sprint 11.2: OAuth Applications (2-3 days)

**Chunk 11.2.1: OAuth Server**
```
Files: crates/op-auth/src/oauth/*.rs (new)
Ruby ref: app/models/oauth_application.rb
Tasks:
- [ ] OAuth application registration
- [ ] Authorization code flow
- [ ] Token endpoint
- [ ] Scope management
```

---

## Phase 12: Optional Modules (LOW)

### Sprint 12.1: Meetings (3-4 days)
```
Ruby ref: modules/meeting/
Tasks:
- [ ] Meeting model
- [ ] Agenda items
- [ ] Participants
- [ ] Meeting API
```

### Sprint 12.2: Gantt/Calendar (3-4 days)
```
Ruby ref: modules/gantt/, modules/calendar/
Tasks:
- [ ] Gantt data endpoint
- [ ] Calendar events
- [ ] iCal export
```

### Sprint 12.3: Boards (2-3 days)
```
Ruby ref: modules/boards/
Tasks:
- [ ] Board model
- [ ] Board lists
- [ ] Card positions
```

### Sprint 12.4: Team Planner (3-4 days)
```
Ruby ref: modules/team_planner/
Tasks:
- [ ] Resource allocation
- [ ] Capacity planning
- [ ] Placeholder users
```

---

## Phase 13: Admin & Config (LOW)

### Sprint 13.1: System Settings (2-3 days)

**Chunk 13.1.1: Settings Model**
```
Files: crates/op-db/src/settings.rs (new)
Ruby ref: app/models/setting.rb
Tasks:
- [ ] SettingRow entity
- [ ] Settings cache
- [ ] Settings API
```

**Chunk 13.1.2: Admin Endpoints**
```
Files: crates/op-api/src/handlers/admin/*.rs (new)
Tasks:
- [ ] GET/PATCH /admin/settings
- [ ] User management admin
- [ ] System info
```

### Sprint 13.2: Backups (2-3 days)

**Chunk 13.2.1: Backup Service**
```
Files: crates/op-services/src/backups/*.rs (new)
Ruby ref: app/services/backups/*.rb
Tasks:
- [ ] Database dump
- [ ] Attachments archive
- [ ] Backup encryption
- [ ] Backup download
```

---

## Phase 14: Polish (LOW)

### Sprint 14.1: Performance (3-4 days)
- [ ] Database query optimization
- [ ] Connection pooling tuning
- [ ] Response caching
- [ ] Pagination optimization

### Sprint 14.2: Testing (3-4 days)
- [ ] Unit test coverage >80%
- [ ] Integration tests for all endpoints
- [ ] Load testing
- [ ] Security testing

### Sprint 14.3: Documentation (2-3 days)
- [ ] API documentation (OpenAPI)
- [ ] Deployment guide
- [ ] Migration guide from Ruby
- [ ] Architecture docs

---

## Agent Chunking Strategy

### Chunk Size Guidelines
- **Small chunk:** 1 file, 1 feature (~100-300 lines)
- **Medium chunk:** 2-3 files, related features (~300-600 lines)
- **Large chunk:** Module/domain (~600-1000 lines)

### Agent Assignment Pattern
```
Per Chunk:
1. Read Ruby reference files (model, service, contract)
2. Create/update Rust files
3. Run cargo build/test
4. Commit with descriptive message
5. Move to next chunk
```

### Context Management
- Each sprint = 1 agent session (avoid context overflow)
- Each chunk = 1 focused task
- Cross-reference via file paths, not code dumps
- Use grep/glob for discovery, read specific files

### Dependency Order
```
P1 (Core) → P2 (Auth) → P3 (WP Features) → P4 (Query)
                ↓
           P5 (Custom Fields)
                ↓
    P6 (Notifications) + P7 (Jobs)
                ↓
    P8-P14 (can parallelize)
```

---

## Success Metrics

### Phase Completion Criteria
- [ ] All endpoints return real data (not stubs)
- [ ] All CRUD operations work end-to-end
- [ ] Authentication validates real tokens
- [ ] Authorization enforces permissions
- [ ] cargo test passes
- [ ] Basic smoke tests pass

### Parity Checklist
- [ ] Work packages: create, update, delete, list, filter
- [ ] Projects: full lifecycle
- [ ] Users: auth, permissions, preferences
- [ ] Queries: save, load, execute
- [ ] Custom fields: define, use, filter
- [ ] Notifications: in-app delivery
- [ ] Attachments: upload, download
- [ ] API: all v3 endpoints functional

---

## Risk Mitigation

### High-Risk Areas
1. **Custom fields** - Complex polymorphic system
2. **Workflows** - State machine logic
3. **Permissions** - Complex query filtering
4. **Attachments** - File handling edge cases

### Mitigation Strategies
- Start with simpler implementations, iterate
- Keep Ruby reference open during transcoding
- Test against real OpenProject database
- Document deviations from Ruby behavior

---

*Last updated: 2026-02-13*
*Status: Phase 1-6 in progress*
