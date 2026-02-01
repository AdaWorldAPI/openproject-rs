# OpenProject Rust Integration Plan

## Current State Summary

### Completed (~20,000+ lines of Rust)

| Crate | Status | Tests | Description |
|-------|--------|-------|-------------|
| `op-core` | ✅ Complete | 22 | Core types, traits, error handling, pagination, HAL framework |
| `op-models` | ✅ Complete | 16 | Domain models (Project, User, WorkPackage, Status, Type, Priority, Role, Member, Version) |
| `op-contracts` | ✅ Complete | 45 | Validation contracts (CRUD for projects, users, work packages) |
| `op-auth` | ✅ Complete | 11 | JWT, API key, session auth, permission system, CurrentUser |
| `op-services` | ✅ Complete | 70 | Business logic services (CRUD for projects, users, work packages) |
| `op-db` | ✅ Complete | 14 | Database layer with SQLx (repositories, query executor) |
| `op-queries` | ✅ Complete | 37 | Query system (filters, sorts, columns, saved views) |
| `op-api` | ✅ Complete | 23 | REST API handlers, HAL representers, full OpenProject API compatibility |
| `op-notifications` | ✅ Complete | 20 | Background jobs, notifications, email delivery |
| `op-attachments` | ✅ Complete | 25 | File storage (local/S3), attachment service |
| `op-journals` | ✅ Complete | 14 | Audit logging, journal data, diff computation |
| `op-server` | 🔶 Partial | 0 | Server entry point (basic health check only) |

**Total: 284 tests passing**

### Remaining Crates (Optional)

| Crate | Priority | Complexity |
|-------|----------|------------|
| `op-cli` | Low | Low |
| `op-custom-fields` | Low | Medium |
| `op-projects` | Low | (Absorbed into services) |
| `op-users` | Low | (Absorbed into services) |
| `op-work-packages` | Low | (Absorbed into services) |

---

## Integration Plan Overview

```
Phase 1: Database Integration (Foundation)         ✅ COMPLETE
    ↓
Phase 2: API Layer Completion (Interface)          ✅ COMPLETE
    ↓
Phase 3: Authentication System (Security)          ✅ COMPLETE
    ↓
Phase 4: Background Jobs & Notifications (Async)   ✅ COMPLETE
    ↓
Phase 5: File Attachments & Storage (Media)        ✅ COMPLETE
    ↓
Phase 6: Advanced Features (Extensions)            ✅ COMPLETE (Journals)
    ↓
Phase 7: Production Readiness (Polish)             🔶 IN PROGRESS
```

---

## Phase 1: Database Integration

### 1.1 Schema Compatibility Mode

**Goal:** Connect to existing OpenProject PostgreSQL database without migrations.

```rust
// crates/op-db/src/compat.rs
pub struct CompatibilityChecker {
    pool: PgPool,
}

impl CompatibilityChecker {
    /// Verify database schema matches expected OpenProject schema
    pub async fn verify_schema(&self) -> Result<SchemaReport, DbError>;

    /// List missing/extra tables
    pub async fn diff_tables(&self) -> Result<TableDiff, DbError>;
}
```

**Tasks:**
- [ ] Create schema verification module
- [ ] Map SQLx queries to existing OpenProject tables
- [ ] Handle Rails-specific columns (`type` for STI, `lock_version`, timestamps)
- [ ] Test against real OpenProject database dump

### 1.2 Entity Mapping

**Map Rust structs to existing tables:**

| Rust Model | PostgreSQL Table | Notes |
|------------|------------------|-------|
| `Project` | `projects` | Nested set (lft/rgt) |
| `User` | `users` | Status enum, preferences in separate table |
| `WorkPackage` | `work_packages` | Many foreign keys |
| `Status` | `statuses` | Workflow transitions |
| `Type` | `types` | Used for STI-like behavior |
| `Priority` | `enumerations` | `type = 'IssuePriority'` |
| `Role` | `roles` | Has many permissions |
| `Member` | `members` | Join table users ↔ projects |
| `Version` | `versions` | Target versions |

### 1.3 Query Translation Layer

```rust
// crates/op-db/src/query_executor.rs
pub struct QueryExecutor {
    pool: PgPool,
}

impl QueryExecutor {
    /// Execute an op-queries Query against the database
    pub async fn execute(&self, query: &Query) -> Result<PaginatedResult<WorkPackage>, DbError>;

    /// Build SQL from filter set
    fn build_where_clause(&self, filters: &FilterSet) -> String;

    /// Build SQL from sort order
    fn build_order_clause(&self, sorts: &SortOrder) -> String;
}
```

---

## Phase 2: API Layer Completion

### 2.1 HAL+JSON Representers

OpenProject uses HAL (Hypertext Application Language) for API responses.

```rust
// crates/op-api/src/representers/hal.rs
#[derive(Serialize)]
pub struct HalResource<T> {
    #[serde(flatten)]
    pub resource: T,
    pub _type: String,
    pub _links: HalLinks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _embedded: Option<HalEmbedded>,
}

#[derive(Serialize)]
pub struct HalCollection<T> {
    pub _type: String,
    pub count: i64,
    pub total: i64,
    pub pageSize: i64,
    pub offset: i64,
    pub _embedded: HalEmbeddedElements<T>,
    pub _links: HalCollectionLinks,
}
```

**Tasks:**
- [ ] Create HAL serialization framework
- [ ] Implement representers for each model:
  - [ ] `WorkPackageRepresenter`
  - [ ] `ProjectRepresenter`
  - [ ] `UserRepresenter`
  - [ ] `StatusRepresenter`
  - [ ] `TypeRepresenter`
  - [ ] `PriorityRepresenter`
  - [ ] `QueryRepresenter`
- [ ] Handle `_links` generation with proper hrefs
- [ ] Support `_embedded` for related resources

### 2.2 Complete API Handlers

| Endpoint | Method | Handler | Status |
|----------|--------|---------|--------|
| `/api/v3/work_packages` | GET | `list_work_packages` | 🔶 Partial |
| `/api/v3/work_packages` | POST | `create_work_package` | 🔶 Partial |
| `/api/v3/work_packages/{id}` | GET | `get_work_package` | 🔶 Partial |
| `/api/v3/work_packages/{id}` | PATCH | `update_work_package` | ❌ Missing |
| `/api/v3/work_packages/{id}` | DELETE | `delete_work_package` | ❌ Missing |
| `/api/v3/projects` | GET | `list_projects` | 🔶 Partial |
| `/api/v3/projects/{id}` | GET | `get_project` | 🔶 Partial |
| `/api/v3/users` | GET | `list_users` | 🔶 Partial |
| `/api/v3/users/me` | GET | `current_user` | ❌ Missing |
| `/api/v3/queries` | GET | `list_queries` | ❌ Missing |
| `/api/v3/queries` | POST | `create_query` | ❌ Missing |
| `/api/v3/queries/{id}` | GET | `get_query` | ❌ Missing |
| `/api/v3/configuration` | GET | `get_configuration` | ❌ Missing |

### 2.3 Error Response Format

```rust
// OpenProject API error format
#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub _type: String, // "Error"
    pub errorIdentifier: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _embedded: Option<ApiErrorDetails>,
}
```

---

## Phase 3: Authentication System

### 3.1 Session Authentication

```rust
// crates/op-auth/src/session.rs
pub struct SessionStore {
    // Use Redis or in-memory store
}

pub struct SessionAuth;

impl<S> FromRequestParts<S> for AuthenticatedUser {
    // Extract user from session cookie
}
```

### 3.2 API Key Authentication

```rust
// crates/op-auth/src/api_key.rs
pub struct ApiKeyAuth;

// Header: Authorization: Basic base64(apikey:x)
// or: X-Authentication-Token: {api_key}
```

### 3.3 Authentication Middleware

```rust
// crates/op-api/src/middleware/auth.rs
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Check session, API key, or OAuth token
    // Set current user in request extensions
}
```

**Tasks:**
- [ ] Implement session storage (Redis preferred)
- [ ] Add API key validation against `tokens` table
- [ ] Create authentication middleware
- [ ] Add permission checking to handlers
- [ ] Implement `/api/v3/users/me` endpoint

---

## Phase 4: Background Jobs & Notifications

### 4.1 Job Queue System

```rust
// crates/op-jobs/src/lib.rs (new crate)
#[async_trait]
pub trait Job: Send + Sync {
    async fn perform(&self, ctx: &JobContext) -> Result<(), JobError>;
    fn queue_name(&self) -> &str { "default" }
    fn retries(&self) -> u32 { 3 }
}

pub struct JobQueue {
    // Use PostgreSQL-based queue (like Sidekiq/GoodJob)
    // or dedicated queue (Redis, RabbitMQ)
}
```

### 4.2 Notifications System

```rust
// crates/op-notifications/src/lib.rs
pub struct NotificationService {
    mailer: Box<dyn Mailer>,
    in_app: InAppNotificationStore,
}

pub enum NotificationChannel {
    Email,
    InApp,
    Webhook,
}

pub struct NotificationEvent {
    pub event_type: String, // "work_package_created", "mentioned", etc.
    pub resource_type: String,
    pub resource_id: Id,
    pub actor_id: Id,
    pub recipients: Vec<Id>,
}
```

**Tasks:**
- [ ] Create `op-jobs` crate with job runner
- [ ] Implement email notification jobs
- [ ] Create in-app notification storage
- [ ] Add webhook delivery support
- [ ] Integrate with work package/project services

---

## Phase 5: File Attachments & Storage

### 5.1 Storage Abstraction

```rust
// crates/op-attachments/src/storage.rs
#[async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Bytes) -> Result<(), StorageError>;
    async fn get(&self, key: &str) -> Result<Bytes, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    async fn url(&self, key: &str, expires_in: Duration) -> Result<String, StorageError>;
}

pub struct LocalStorage { root: PathBuf }
pub struct S3Storage { client: S3Client, bucket: String }
```

### 5.2 Attachment Model

```rust
// crates/op-attachments/src/model.rs
pub struct Attachment {
    pub id: Id,
    pub container_type: String, // "WorkPackage", "WikiPage", etc.
    pub container_id: Id,
    pub filename: String,
    pub disk_filename: String,
    pub filesize: i64,
    pub content_type: String,
    pub digest: String,
    pub author_id: Id,
    pub created_at: DateTime<Utc>,
}
```

---

## Phase 6: Advanced Features

### 6.1 Custom Fields

```rust
// crates/op-custom-fields/src/lib.rs (new crate)
pub struct CustomField {
    pub id: Id,
    pub field_type: CustomFieldType,
    pub name: String,
    pub field_format: String, // "string", "int", "list", "user", etc.
    pub possible_values: Vec<String>,
    pub regexp: Option<String>,
    pub is_required: bool,
    pub is_for_all: bool,
}

pub enum CustomFieldType {
    WorkPackage,
    Project,
    User,
    Version,
    TimeEntry,
}
```

### 6.2 Journals (Audit Log)

```rust
// crates/op-journals/src/lib.rs (new crate)
pub struct Journal {
    pub id: Id,
    pub journable_type: String,
    pub journable_id: Id,
    pub user_id: Id,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub version: i32,
    pub data: JournalData, // Snapshot of entity at this version
}

// Implement journaling in services
impl CreateWorkPackageService {
    fn call(&self, params: WorkPackageParams) -> ServiceResult<WorkPackage> {
        // ... create work package ...
        self.create_journal(&work_package, "created")?;
        // ...
    }
}
```

### 6.3 Additional Models

| Model | Table | Priority |
|-------|-------|----------|
| Wiki | `wikis`, `wiki_pages`, `wiki_contents` | Medium |
| Forum | `forums`, `messages` | Low |
| News | `news` | Low |
| TimeEntry | `time_entries` | High |
| CostEntry | `cost_entries` | Medium |
| Meeting | `meetings`, `meeting_contents` | Low |
| Document | `documents` | Low |

---

## Phase 7: Production Readiness

### 7.1 Configuration Management

```rust
// crates/op-core/src/config.rs
#[derive(Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: Option<RedisConfig>,
    pub storage: StorageConfig,
    pub mail: MailConfig,
    pub security: SecurityConfig,
    pub features: FeatureFlags,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Load from environment, YAML, TOML
    }
}
```

### 7.2 Health Checks

```rust
// crates/op-server/src/health.rs
pub struct HealthChecker {
    db: PgPool,
    redis: Option<RedisPool>,
    storage: Box<dyn Storage>,
}

impl HealthChecker {
    pub async fn check_all(&self) -> HealthReport {
        HealthReport {
            database: self.check_db().await,
            redis: self.check_redis().await,
            storage: self.check_storage().await,
            // ...
        }
    }
}
```

### 7.3 Observability

- [ ] Structured logging (tracing)
- [ ] Metrics (Prometheus)
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Error tracking (Sentry integration)

### 7.4 Performance Optimization

- [ ] Connection pooling tuning
- [ ] Query optimization
- [ ] Caching layer (Redis)
- [ ] Response compression
- [ ] Rate limiting

---

## Implementation Priority Matrix

```
High Priority / High Impact:
├── Phase 1: Database Integration
├── Phase 2: API Layer Completion
└── Phase 3: Authentication System

Medium Priority / Medium Impact:
├── Phase 4: Background Jobs & Notifications
└── Phase 5: File Attachments

Lower Priority / Feature Extensions:
├── Phase 6: Advanced Features (Custom Fields, Journals)
└── Phase 7: Production Readiness
```

---

## Recommended Next Steps

### Immediate (Next Session)

1. **Complete API Layer** (Phase 2)
   - Implement HAL representers
   - Complete work package CRUD endpoints
   - Add query execution endpoint

2. **Database Integration** (Phase 1)
   - Create query executor that translates op-queries to SQL
   - Test against OpenProject database dump

### Short-term (1-2 Sessions)

3. **Authentication** (Phase 3)
   - API key authentication
   - Current user endpoint
   - Permission middleware

4. **Integration Testing**
   - Set up test database with seed data
   - Create API integration tests
   - Verify compatibility with OpenProject frontend

### Medium-term (3-5 Sessions)

5. **Notifications** (Phase 4)
6. **Attachments** (Phase 5)
7. **Journals/Audit Log** (Phase 6)

---

## Architecture Decision Records

### ADR-001: Database Compatibility

**Decision:** Use existing OpenProject PostgreSQL schema without modifications.

**Rationale:** Allows drop-in replacement without data migration. Enables gradual transition.

**Consequences:** Must handle Rails conventions (STI, timestamps, lock_version).

### ADR-002: HAL+JSON API Format

**Decision:** Maintain full compatibility with OpenProject API v3 format.

**Rationale:** Allows existing frontend and integrations to work unchanged.

**Consequences:** More complex serialization, but proven API design.

### ADR-003: Service Layer Pattern

**Decision:** Use service objects for business logic (already implemented).

**Rationale:** Mirrors OpenProject's Ruby architecture, makes porting straightforward.

**Consequences:** Familiar pattern, easy to test, clear separation of concerns.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Schema drift from OpenProject | Medium | High | Version lock, schema verification |
| API incompatibility | Medium | High | Extensive API testing, HAL validation |
| Performance regression | Low | Medium | Benchmarking, profiling |
| Authentication bypass | Low | Critical | Security audit, penetration testing |
| Data corruption | Low | Critical | Transaction handling, backup strategy |

---

## Success Metrics

1. **Functional Parity**
   - All API v3 endpoints implemented
   - Existing frontend works without modification

2. **Performance**
   - Response times ≤ Ruby implementation
   - Memory usage < 50% of Ruby

3. **Reliability**
   - 99.9% uptime target
   - Zero data loss

4. **Developer Experience**
   - Clear documentation
   - Easy local development setup
   - CI/CD pipeline
