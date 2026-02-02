# Architecture

This document describes the technical architecture of OpenProject RS.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Clients                                     │
│                  (Web App, Mobile, API Consumers)                        │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │ HTTPS
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           op-server                                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐   │
│  │   Health    │ │   Metrics   │ │    CORS     │ │   Compression   │   │
│  │   Checks    │ │  Middleware │ │  Middleware │ │    Middleware   │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            op-api                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      HAL Representers                            │   │
│  │  (WorkPackageRepresenter, ProjectRepresenter, UserRepresenter)   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                       API Handlers                               │   │
│  │     (work_packages.rs, projects.rs, users.rs, queries.rs)        │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    ▼             ▼             ▼
┌───────────────────────┐ ┌─────────────┐ ┌─────────────────────────────┐
│       op-auth         │ │  op-queries │ │       op-services           │
│  ┌─────────────────┐  │ │             │ │  ┌─────────────────────┐   │
│  │  JWT Auth       │  │ │  Filters    │ │  │  CreateWorkPackage  │   │
│  │  API Key Auth   │  │ │  Sorts      │ │  │  UpdateWorkPackage  │   │
│  │  Session Auth   │  │ │  Columns    │ │  │  DeleteWorkPackage  │   │
│  │  Permissions    │  │ │  Groups     │ │  │  (+ Project, User)  │   │
│  └─────────────────┘  │ │             │ │  └─────────────────────┘   │
└───────────────────────┘ └─────────────┘ └─────────────┬───────────────┘
                                                        │
                                          ┌─────────────┼─────────────┐
                                          ▼             ▼             ▼
                              ┌─────────────────┐ ┌───────────┐ ┌───────────────┐
                              │  op-contracts   │ │ op-models │ │    op-db      │
                              │  (Validation)   │ │  (Domain) │ │  (SQLx/Repo)  │
                              └─────────────────┘ └───────────┘ └───────┬───────┘
                                                                        │
                                                                        ▼
                                                              ┌─────────────────┐
                                                              │   PostgreSQL    │
                                                              └─────────────────┘
```

## Crate Hierarchy

```
op-core (Foundation)
    │
    ├── op-models (Domain Models)
    │       │
    │       └── op-contracts (Validation)
    │               │
    │               └── op-services (Business Logic)
    │                       │
    │                       ├── op-db (Database)
    │                       │
    │                       └── op-api (REST Handlers)
    │                               │
    │                               └── op-server (HTTP Server)
    │
    ├── op-auth (Authentication)
    │
    ├── op-queries (Query System)
    │
    ├── op-notifications (Background Jobs)
    │
    ├── op-attachments (File Storage)
    │
    └── op-journals (Audit Logging)
```

## Crate Details

### op-core

**Purpose**: Foundation crate with shared types and traits.

```rust
// Core traits
pub trait Entity {
    fn id(&self) -> Option<Id>;
}

pub trait Timestamped {
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

// Service result type
pub enum ServiceResult<T> {
    Success(T),
    ValidationError(ValidationErrors),
    NotFound,
    PermissionDenied,
    Conflict(String),
}

// Configuration
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub email: EmailConfig,
    pub storage: StorageConfig,
    pub features: FeatureFlags,
}
```

**Tests**: 25

### op-models

**Purpose**: Domain models matching OpenProject's data structures.

```rust
pub struct WorkPackage {
    pub id: Option<Id>,
    pub subject: String,
    pub description: Option<String>,
    pub project_id: Id,
    pub type_id: Id,
    pub status_id: Id,
    pub priority_id: Id,
    pub author_id: Id,
    pub assignee_id: Option<Id>,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub estimated_hours: Option<f64>,
    pub done_ratio: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Project {
    pub id: Option<Id>,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub public: bool,
    pub active: bool,
    pub parent_id: Option<Id>,
    // ... nested set fields for hierarchy
}
```

**Tests**: 16

### op-contracts

**Purpose**: Validation contracts for CRUD operations.

```rust
pub struct CreateWorkPackageContract<'a, U: UserContext> {
    work_package: &'a WorkPackage,
    user: &'a U,
}

impl<'a, U: UserContext> CreateWorkPackageContract<'a, U> {
    pub fn validate(&self) -> ValidationResult {
        let mut errors = ValidationErrors::new();

        // Subject required
        if self.work_package.subject.is_empty() {
            errors.add("subject", "can't be blank");
        }

        // Project required
        if self.work_package.project_id == 0 {
            errors.add("project", "can't be blank");
        }

        // Permission check
        if !self.user.can_create_work_packages(self.work_package.project_id) {
            errors.add("base", "permission denied");
        }

        errors.into()
    }
}
```

**Tests**: 45

### op-auth

**Purpose**: Authentication and authorization.

```rust
// JWT Authentication
pub struct JwtAuth {
    secret: String,
    expiration: Duration,
}

impl JwtAuth {
    pub fn create_token(&self, user: &User) -> Result<String, AuthError>;
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError>;
}

// API Key Authentication
pub struct ApiKeyAuth;

impl ApiKeyAuth {
    pub async fn authenticate(&self, key: &str) -> Result<User, AuthError>;
}

// Permission System
pub trait UserContext {
    fn can_view_project(&self, project_id: Id) -> bool;
    fn can_edit_work_package(&self, wp: &WorkPackage) -> bool;
    fn can_create_work_packages(&self, project_id: Id) -> bool;
}
```

**Tests**: 11

### op-services

**Purpose**: Business logic layer implementing service objects pattern.

```rust
pub struct CreateWorkPackageService<R: WorkPackageRepository> {
    repository: Arc<R>,
}

impl<R: WorkPackageRepository> CreateWorkPackageService<R> {
    pub async fn call<U: UserContext>(
        &self,
        params: WorkPackageParams,
        user: &U,
    ) -> ServiceResult<WorkPackage> {
        // 1. Build work package from params
        let work_package = WorkPackage::from_params(params);

        // 2. Validate with contract
        let contract = CreateWorkPackageContract::new(&work_package, user);
        if let Err(errors) = contract.validate() {
            return ServiceResult::ValidationError(errors);
        }

        // 3. Persist to database
        let created = self.repository.create(work_package).await?;

        // 4. Create journal entry
        self.create_journal(&created, user).await?;

        ServiceResult::Success(created)
    }
}
```

**Tests**: 70

### op-db

**Purpose**: Database layer with SQLx.

```rust
#[async_trait]
pub trait WorkPackageRepository: Send + Sync {
    async fn find(&self, id: Id) -> Result<Option<WorkPackage>, DbError>;
    async fn find_all(&self, query: &Query) -> Result<Vec<WorkPackage>, DbError>;
    async fn create(&self, wp: WorkPackage) -> Result<WorkPackage, DbError>;
    async fn update(&self, wp: WorkPackage) -> Result<WorkPackage, DbError>;
    async fn delete(&self, id: Id) -> Result<(), DbError>;
}

pub struct QueryExecutor {
    pool: PgPool,
}

impl QueryExecutor {
    pub async fn execute(&self, query: &Query) -> Result<Vec<WorkPackage>, DbError> {
        let sql = self.build_query(query);
        sqlx::query_as(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::from)
    }
}
```

**Tests**: 14

### op-queries

**Purpose**: Query system for filtering, sorting, and saved views.

```rust
pub struct Query {
    pub id: Option<Id>,
    pub name: String,
    pub filters: FilterSet,
    pub sorts: SortOrder,
    pub columns: Vec<Column>,
    pub group_by: Option<GroupBy>,
    pub display_sums: bool,
}

pub struct FilterSet {
    filters: Vec<Filter>,
}

pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    Between,
    In,
    IsNull,
    IsNotNull,
}

// Builder pattern
let query = QueryBuilder::new()
    .filter("status_id", FilterOperator::In, vec!["1", "2", "3"])
    .filter("assignee_id", FilterOperator::Equals, user_id)
    .sort("priority", SortDirection::Desc)
    .sort("due_date", SortDirection::Asc)
    .build();
```

**Tests**: 37

### op-api

**Purpose**: REST API handlers with HAL+JSON responses.

```rust
// HAL Representer
#[derive(Serialize)]
pub struct HalWorkPackage {
    pub _type: String,
    pub id: Id,
    pub subject: String,
    pub _links: WorkPackageLinks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _embedded: Option<WorkPackageEmbedded>,
}

// Handler
pub async fn list_work_packages(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
    user: CurrentUser,
) -> Result<Json<HalCollection<HalWorkPackage>>, ApiError> {
    let query = params.to_query();
    let work_packages = state.services.work_packages.list(&query, &user).await?;

    Ok(Json(HalCollection::from(work_packages)))
}
```

**Tests**: 23

### op-notifications

**Purpose**: Background jobs and notifications.

```rust
// Job Queue
#[async_trait]
pub trait Job: Send + Sync {
    async fn perform(&self, ctx: &JobContext) -> Result<(), JobError>;
    fn queue_name(&self) -> &str { "default" }
    fn retries(&self) -> u32 { 3 }
}

pub struct JobQueue {
    jobs: RwLock<VecDeque<QueuedJob>>,
}

// Notification Service
pub struct NotificationService {
    channels: Vec<Box<dyn Channel>>,
}

impl NotificationService {
    pub async fn notify(&self, event: NotificationEvent) -> Result<(), NotifyError> {
        for channel in &self.channels {
            channel.deliver(&event).await?;
        }
        Ok(())
    }
}
```

**Tests**: 20

### op-attachments

**Purpose**: File storage abstraction.

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Bytes) -> StorageResult<FileMetadata>;
    async fn get(&self, key: &str) -> StorageResult<Bytes>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn url(&self, key: &str, expires_in: Duration) -> StorageResult<String>;
}

pub struct LocalStorage { root: PathBuf }
pub struct S3Storage { client: S3Client, bucket: String }
pub struct MemoryStorage { files: RwLock<HashMap<String, Bytes>> }
```

**Tests**: 25

### op-journals

**Purpose**: Audit logging with change tracking.

```rust
pub struct Journal {
    pub id: Option<Id>,
    pub journable_type: JournalType,
    pub journable_id: Id,
    pub user_id: Id,
    pub notes: Option<String>,
    pub version: JournalVersion,
    pub created_at: DateTime<Utc>,
}

pub struct JournalDiff {
    changes: Vec<FieldChange>,
}

pub struct FieldChange {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

impl JournalDiff {
    pub fn compute(old: &JournalData, new: &JournalData) -> Self {
        // Compare field by field and record changes
    }
}
```

**Tests**: 14

### op-server

**Purpose**: HTTP server binary with middleware.

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Load config from environment
    let config = AppConfig::from_env()?;

    // Initialize components
    let health = HealthChecker::new(config.clone());
    let metrics = Metrics::new();

    // Build router
    let app = Router::new()
        .merge(health_routes())
        .merge(metrics_routes())
        .nest("/api/v3", api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive());

    // Start server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}
```

**Tests**: 11

## Data Flow

### Request Lifecycle

```
1. HTTP Request arrives at op-server
   │
2. Middleware chain processes request
   │ - TraceLayer (logging)
   │ - CompressionLayer (gzip)
   │ - CorsLayer (CORS headers)
   │ - MetricsMiddleware (timing)
   │
3. Router matches endpoint
   │
4. op-api handler receives request
   │ - Extract parameters
   │ - Parse authentication
   │ - Validate input
   │
5. op-services processes business logic
   │ - op-contracts validates
   │ - op-auth checks permissions
   │
6. op-db executes database operations
   │
7. Response flows back through layers
   │ - HAL representer formats response
   │ - Middleware adds headers
   │
8. HTTP Response sent to client
```

### Service Object Pattern

```rust
// 1. Params come in from API
let params = WorkPackageParams { subject: "Task", ... };

// 2. Service orchestrates the operation
let service = CreateWorkPackageService::new(repo);
let result = service.call(params, &user).await;

// 3. Contract validates
let contract = CreateWorkPackageContract::new(&wp, &user);
contract.validate()?;

// 4. Repository persists
let created = repo.create(wp).await?;

// 5. Journal records change
let journal = Journal::for_creation(&created, &user);
journal_service.record(journal).await?;

// 6. Notifications dispatched
notification_service.notify(WorkPackageCreated { wp: &created }).await?;
```

## Error Handling

```rust
// Unified error type
pub enum AppError {
    NotFound(String),
    Validation(ValidationErrors),
    Permission(String),
    Database(DbError),
    Internal(anyhow::Error),
}

// Converts to appropriate HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                json!({ "_type": "Error", "message": msg })
            ),
            AppError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "_type": "Error", "errors": errors })
            ),
            // ...
        };
        (status, Json(body)).into_response()
    }
}
```

## Testing Strategy

```
Unit Tests (per crate)
├── op-core: Type behavior, config parsing
├── op-models: Model construction, defaults
├── op-contracts: Validation rules
├── op-services: Business logic (mocked repos)
├── op-db: Query building, SQL generation
├── op-queries: Filter/sort logic
├── op-api: Handler responses
├── op-notifications: Job processing
├── op-attachments: Storage operations
├── op-journals: Diff computation
└── op-server: Endpoint integration

Integration Tests
├── API endpoint tests
├── Database integration
└── Full request/response cycles
```

## Performance Considerations

1. **Connection Pooling**: SQLx manages a pool of database connections
2. **Async I/O**: All I/O is non-blocking via Tokio
3. **Zero-Copy**: Bytes type used for efficient data transfer
4. **Lazy Loading**: Related resources loaded on demand
5. **Caching**: Health check results cached for 10s
6. **Compression**: gzip compression for API responses

## Security

1. **Authentication**: JWT tokens with configurable expiration
2. **Authorization**: Role-based permissions checked at service layer
3. **Input Validation**: Contracts validate all user input
4. **SQL Injection**: Prevented by SQLx parameterized queries
5. **Non-root**: Docker container runs as non-root user
6. **Secrets**: Sensitive config from environment variables
