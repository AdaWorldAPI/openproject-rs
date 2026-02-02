# API Documentation

OpenProject RS implements the OpenProject API v3, a RESTful API using HAL+JSON format.

## Base URL

```
http://localhost:8080/api/v3
```

## Authentication

### API Key

```bash
# Basic auth with API key
curl -u "apikey:your-api-key" http://localhost:8080/api/v3/users/me

# Or via header
curl -H "Authorization: Basic $(echo -n 'apikey:your-api-key' | base64)" \
  http://localhost:8080/api/v3/users/me
```

### JWT Token

```bash
# Get token
curl -X POST http://localhost:8080/api/v3/auth/token \
  -H "Content-Type: application/json" \
  -d '{"login": "admin", "password": "admin"}'

# Use token
curl -H "Authorization: Bearer eyJ..." http://localhost:8080/api/v3/users/me
```

## Response Format

All responses use HAL+JSON format:

```json
{
  "_type": "WorkPackage",
  "id": 1,
  "subject": "Example task",
  "_links": {
    "self": { "href": "/api/v3/work_packages/1" },
    "project": { "href": "/api/v3/projects/1" }
  },
  "_embedded": {
    "status": {
      "_type": "Status",
      "id": 1,
      "name": "New"
    }
  }
}
```

## Endpoints

### Root

#### GET /api/v3

Returns the API root with available resources.

**Response:**
```json
{
  "_type": "Root",
  "instanceName": "OpenProject RS",
  "coreVersion": "0.1.0",
  "_links": {
    "self": { "href": "/api/v3" },
    "configuration": { "href": "/api/v3/configuration" },
    "user": { "href": "/api/v3/users/me" },
    "projects": { "href": "/api/v3/projects" },
    "workPackages": { "href": "/api/v3/work_packages" }
  }
}
```

---

### Configuration

#### GET /api/v3/configuration

Returns instance configuration.

**Response:**
```json
{
  "_type": "Configuration",
  "maximumAttachmentFileSize": 268435456,
  "perPageOptions": [20, 100],
  "dateFormat": "%Y-%m-%d",
  "timeFormat": "%H:%M",
  "startOfWeek": 1,
  "activeFeatureFlags": ["bim", "boards", "webhooks"],
  "_links": {
    "self": { "href": "/api/v3/configuration" }
  }
}
```

---

### Users

#### GET /api/v3/users/me

Returns the current authenticated user.

**Response:**
```json
{
  "_type": "User",
  "id": 1,
  "login": "admin",
  "firstName": "Admin",
  "lastName": "User",
  "email": "admin@example.com",
  "admin": true,
  "status": "active",
  "createdAt": "2024-01-01T00:00:00Z",
  "updatedAt": "2024-01-01T00:00:00Z",
  "_links": {
    "self": { "href": "/api/v3/users/1" }
  }
}
```

#### GET /api/v3/users

List all users.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `offset` | integer | Page offset (default: 0) |
| `pageSize` | integer | Items per page (default: 20) |
| `filters` | string | JSON-encoded filters |

**Response:**
```json
{
  "_type": "Collection",
  "total": 100,
  "count": 20,
  "pageSize": 20,
  "offset": 0,
  "_embedded": {
    "elements": [
      { "_type": "User", "id": 1, ... },
      { "_type": "User", "id": 2, ... }
    ]
  },
  "_links": {
    "self": { "href": "/api/v3/users?offset=0&pageSize=20" },
    "next": { "href": "/api/v3/users?offset=20&pageSize=20" }
  }
}
```

#### GET /api/v3/users/:id

Get a specific user.

**Response:** Single user object.

---

### Projects

#### GET /api/v3/projects

List all projects visible to the current user.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `offset` | integer | Page offset |
| `pageSize` | integer | Items per page |
| `filters` | string | JSON-encoded filters |
| `select` | string | Fields to include |

**Filters:**
```json
[
  { "active": { "operator": "=", "values": ["t"] } },
  { "name_and_identifier": { "operator": "~", "values": ["search"] } }
]
```

**Response:**
```json
{
  "_type": "Collection",
  "total": 10,
  "_embedded": {
    "elements": [
      {
        "_type": "Project",
        "id": 1,
        "name": "My Project",
        "identifier": "my-project",
        "description": { "raw": "Description", "html": "<p>Description</p>" },
        "public": true,
        "active": true,
        "createdAt": "2024-01-01T00:00:00Z",
        "_links": {
          "self": { "href": "/api/v3/projects/1" },
          "workPackages": { "href": "/api/v3/projects/1/work_packages" }
        }
      }
    ]
  }
}
```

#### GET /api/v3/projects/:id

Get a specific project.

#### POST /api/v3/projects

Create a new project.

**Request:**
```json
{
  "name": "New Project",
  "identifier": "new-project",
  "description": { "raw": "Project description" },
  "public": false
}
```

**Response:** Created project object (201 Created).

#### PATCH /api/v3/projects/:id

Update a project.

#### DELETE /api/v3/projects/:id

Delete a project (204 No Content).

---

### Work Packages

#### GET /api/v3/work_packages

List work packages across all projects.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `offset` | integer | Page offset |
| `pageSize` | integer | Items per page |
| `filters` | string | JSON-encoded filters |
| `sortBy` | string | Sort criteria |
| `groupBy` | string | Group by field |
| `showSums` | boolean | Show aggregations |

**Filters:**
```json
[
  { "status_id": { "operator": "o", "values": [] } },
  { "assignee": { "operator": "=", "values": ["me"] } },
  { "project_id": { "operator": "=", "values": ["1", "2"] } },
  { "type_id": { "operator": "=", "values": ["1"] } },
  { "due_date": { "operator": "<t+", "values": ["7"] } }
]
```

**Filter Operators:**
| Operator | Description |
|----------|-------------|
| `=` | Equals |
| `!` | Not equals |
| `~` | Contains |
| `o` | Open (status) |
| `c` | Closed (status) |
| `>t-` | More than days ago |
| `<t+` | Less than days from now |
| `t` | Today |
| `w` | This week |

**Sort By:**
```
sortBy=[["priority","desc"],["due_date","asc"]]
```

**Response:**
```json
{
  "_type": "Collection",
  "total": 500,
  "count": 20,
  "_embedded": {
    "elements": [
      {
        "_type": "WorkPackage",
        "id": 1,
        "subject": "Fix login bug",
        "description": { "raw": "...", "html": "..." },
        "startDate": "2024-01-01",
        "dueDate": "2024-01-15",
        "estimatedTime": "PT8H",
        "spentTime": "PT4H",
        "percentageDone": 50,
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-10T00:00:00Z",
        "_links": {
          "self": { "href": "/api/v3/work_packages/1" },
          "project": { "href": "/api/v3/projects/1", "title": "My Project" },
          "type": { "href": "/api/v3/types/1", "title": "Task" },
          "status": { "href": "/api/v3/statuses/1", "title": "New" },
          "priority": { "href": "/api/v3/priorities/2", "title": "Normal" },
          "author": { "href": "/api/v3/users/1", "title": "Admin" },
          "assignee": { "href": "/api/v3/users/2", "title": "Developer" }
        }
      }
    ]
  }
}
```

#### GET /api/v3/work_packages/:id

Get a specific work package.

#### POST /api/v3/work_packages

Create a new work package.

**Request:**
```json
{
  "subject": "New task",
  "description": { "raw": "Task description" },
  "_links": {
    "project": { "href": "/api/v3/projects/1" },
    "type": { "href": "/api/v3/types/1" },
    "status": { "href": "/api/v3/statuses/1" },
    "priority": { "href": "/api/v3/priorities/2" },
    "assignee": { "href": "/api/v3/users/2" }
  }
}
```

#### PATCH /api/v3/work_packages/:id

Update a work package.

**Request:**
```json
{
  "lockVersion": 5,
  "subject": "Updated subject",
  "_links": {
    "status": { "href": "/api/v3/statuses/2" }
  }
}
```

**Note:** `lockVersion` is required for optimistic locking.

#### DELETE /api/v3/work_packages/:id

Delete a work package (204 No Content).

---

### Queries

#### GET /api/v3/queries

List saved queries.

**Response:**
```json
{
  "_type": "Collection",
  "_embedded": {
    "elements": [
      {
        "_type": "Query",
        "id": 1,
        "name": "My Tasks",
        "public": false,
        "starred": true,
        "filters": [...],
        "sortBy": [...],
        "columns": [...],
        "_links": {
          "self": { "href": "/api/v3/queries/1" },
          "results": { "href": "/api/v3/work_packages?query_id=1" }
        }
      }
    ]
  }
}
```

#### GET /api/v3/queries/:id

Get a specific query with its configuration.

#### POST /api/v3/queries

Create a new saved query.

---

### Statuses

#### GET /api/v3/statuses

List all work package statuses.

**Response:**
```json
{
  "_type": "Collection",
  "_embedded": {
    "elements": [
      {
        "_type": "Status",
        "id": 1,
        "name": "New",
        "color": "#1A67A3",
        "isClosed": false,
        "isDefault": true,
        "position": 1
      }
    ]
  }
}
```

---

### Types

#### GET /api/v3/types

List all work package types.

**Response:**
```json
{
  "_type": "Collection",
  "_embedded": {
    "elements": [
      {
        "_type": "Type",
        "id": 1,
        "name": "Task",
        "color": "#1A67A3",
        "isDefault": true,
        "isMilestone": false,
        "position": 1
      }
    ]
  }
}
```

---

### Priorities

#### GET /api/v3/priorities

List all priorities.

**Response:**
```json
{
  "_type": "Collection",
  "_embedded": {
    "elements": [
      {
        "_type": "Priority",
        "id": 1,
        "name": "Low",
        "color": "#69B3FF",
        "isDefault": false,
        "isActive": true,
        "position": 1
      }
    ]
  }
}
```

---

## Health & Metrics

### GET /health

Simple health check.

**Response:** `OK` (200)

### GET /health/live

Kubernetes liveness probe.

**Response:** `OK` (200)

### GET /health/ready

Kubernetes readiness probe.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "components": [
    { "name": "database", "status": "healthy", "response_time_ms": 2 },
    { "name": "memory", "status": "healthy" },
    { "name": "disk", "status": "healthy" }
  ],
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### GET /health/full

Detailed health report with all component statuses.

### GET /metrics

Prometheus-format metrics.

**Response:**
```
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total 1000

# HELP http_request_duration_ms_total Total HTTP request duration
# TYPE http_request_duration_ms_total counter
http_request_duration_ms_total 50000

# HELP db_queries_total Total database queries
# TYPE db_queries_total counter
db_queries_total 5000

# HELP uptime_seconds Server uptime
# TYPE uptime_seconds gauge
uptime_seconds 3600
```

### GET /metrics.json

JSON-format metrics.

**Response:**
```json
{
  "http": {
    "requests_total": 1000,
    "requests_2xx": 950,
    "requests_4xx": 40,
    "requests_5xx": 10,
    "active_connections": 5
  },
  "database": {
    "queries_total": 5000,
    "query_duration_ms_total": 10000
  },
  "uptime_seconds": 3600
}
```

---

## Error Responses

All errors follow this format:

```json
{
  "_type": "Error",
  "errorIdentifier": "urn:openproject-org:api:v3:errors:NotFound",
  "message": "The requested resource could not be found."
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful delete) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict (lock version mismatch) |
| 422 | Unprocessable Entity (validation errors) |
| 500 | Internal Server Error |

### Validation Errors

```json
{
  "_type": "Error",
  "errorIdentifier": "urn:openproject-org:api:v3:errors:PropertyConstraintViolation",
  "message": "Subject can't be blank.",
  "_embedded": {
    "details": {
      "attribute": "subject"
    }
  }
}
```

---

## Pagination

All collection endpoints support pagination:

```
GET /api/v3/work_packages?offset=20&pageSize=10
```

**Response includes:**
```json
{
  "total": 100,
  "count": 10,
  "pageSize": 10,
  "offset": 20,
  "_links": {
    "self": { "href": "...?offset=20&pageSize=10" },
    "first": { "href": "...?offset=0&pageSize=10" },
    "prev": { "href": "...?offset=10&pageSize=10" },
    "next": { "href": "...?offset=30&pageSize=10" },
    "last": { "href": "...?offset=90&pageSize=10" }
  }
}
```

---

## Rate Limiting

Currently no rate limiting is implemented. For production, consider using a reverse proxy (nginx, Traefik) or API gateway for rate limiting.
