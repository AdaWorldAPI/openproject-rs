# OpenProject RS - Port Tracker

This directory tracks the progress of porting OpenProject from Ruby/Rails to Rust.

## Status Legend

- ⬜ Not Started
- 🟡 In Progress
- 🟢 Complete
- 🔴 Blocked
- ⏸️ On Hold

## Directory Structure

```
tracker/
├── README.md                 # This file
├── INVENTORY.md             # Full inventory of Ruby codebase
├── contracts/               # Contract port status
├── models/                  # Model port status
├── services/                # Service port status
├── api/                     # API endpoint port status
└── progress.md              # Overall progress summary
```

## Quick Links

- [Full Inventory](./INVENTORY.md)
- [Models Progress](./models/README.md)
- [Contracts Progress](./contracts/README.md)
- [Services Progress](./services/README.md)
- [API Progress](./api/README.md)

## Port Strategy

1. **Phase 1: Core Foundation** (op-core)
   - Error types and results
   - Core traits (Entity, Service, Contract)
   - Pagination and API utilities
   - Configuration loading

2. **Phase 2: Data Layer** (op-models, op-db)
   - Define all model structs
   - Database schema and migrations
   - Repository implementations
   - Query builders

3. **Phase 3: Business Logic** (op-contracts, op-services)
   - Port all contracts (validation rules)
   - Port all services (business operations)
   - Maintain Ruby API compatibility

4. **Phase 4: API Layer** (op-api, op-server)
   - HAL+JSON representers
   - API v3 endpoints
   - Authentication middleware
   - Rate limiting

5. **Phase 5: Integration** (op-cli, op-server)
   - Full server assembly
   - CLI tools
   - Docker deployment
   - Testing suite

## Compatibility Goals

- 100% API v3 compatibility
- Same database schema (can run against existing PostgreSQL)
- Configuration via same environment variables
- Drop-in replacement for Ruby version
