# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Run the server (loads .env automatically)
cargo run

# Check compilation without building
cargo check

# Run with specific log level
RUST_LOG=api=debug cargo run
RUST_LOG=api=trace cargo run  # verbose

# Database migrations (requires sqlx-cli: cargo install sqlx-cli)
sqlx migrate run
sqlx migrate revert
sqlx database reset -y  # destructive: drops and recreates
```

## Environment Setup

Required environment variables (see `.env.example`):
- `DATABASE_URL` - PostgreSQL connection string
- `GOOGLE_CLIENT_ID` - Google OAuth client ID
- `GOOGLE_CLIENT_SECRET` - Google OAuth client secret
- `GOOGLE_REDIRECT_URL` - OAuth callback URL (default: `http://localhost:3000/connectors/ga4/callback`)

## Architecture

This is a Rust API using Axum for a connector management system. Projects contain connectors, and connectors integrate with external services (currently GA4/Google Analytics).

### Layer Structure

```
main.rs              → App setup, router composition via handler::routes()
├── api/
│   ├── handler/     → HTTP handlers, each exports routes() -> Router<AppState>
│   └── error.rs     → AppError type with IntoResponse impl
├── infrastructure/  → Repository pattern for database access (SQLx)
└── models/          → Domain structs with Serialize/Deserialize
```

### Key Patterns

**Handler pattern**: Each handler module exports a `routes()` function:
```rust
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects", post(create))
        .route("/projects/{id}", get(get_by_id))
}
```

**Error handling**: Handlers return `impl IntoResponse`. Use `AppError` for errors:
```rust
async fn get_by_id(...) -> impl IntoResponse {
    state.repo.find_by_id(id).await.map(Json).map_err(AppError::from)
}
```
- `AppError::not_found()`, `AppError::bad_request()`, `AppError::conflict()`, `AppError::internal()`
- `From<sqlx::Error>` is implemented for automatic conversion with `?`

**Database**: SQLx with compile-time checked queries. Schema changes require migration + recompile.

### Data Model

- **Project**: Container for connectors (id, name, description)
- **Connector**: External service integration belonging to a project (id, project_id, name, type, config JSONB)
- Delete behavior: RESTRICT (can't delete project with connectors)

### GA4 OAuth Flow

1. `GET /projects/{id}/connectors/ga4/auth` → Returns Google OAuth URL (project_id encoded in state param)
2. User authorizes on Google
3. `GET /connectors/ga4/callback?code=...&state=...` → Exchanges code for tokens, creates connector
4. Tokens stored in connector's `config` JSONB field

## API Endpoints

Projects: `/projects` (CRUD)
Connectors: `/projects/{project_id}/connectors` (CRUD)
GA4: `/projects/{project_id}/connectors/ga4/[auth|status|properties|disconnect]`
GA4 Data: `POST /projects/{project_id}/connectors/ga4/{connector_id}/pull`

## GA4 Data Storage

### Storage Location

GA4 data is stored in DuckDB (columnar database optimized for analytics):
```
/tmp/ga4_data/{project_id}/{connector_id}/ga4.duckdb
```

### Schema

```sql
CREATE TABLE ga4_records (
    -- Dimensions (composite primary key)
    date VARCHAR,
    country VARCHAR,
    device_category VARCHAR,
    event_name VARCHAR,
    browser VARCHAR,
    operating_system VARCHAR,
    screen_resolution VARCHAR,
    -- Metrics
    active_users BIGINT,
    sessions BIGINT,
    screen_page_views BIGINT,
    bounce_rate DOUBLE,
    average_session_duration DOUBLE,
    PRIMARY KEY (date, country, device_category, event_name, browser, operating_system, screen_resolution)
);
```

### Deduplication Strategy

The 7 dimension fields form a **composite primary key**. GA4 aggregates data by these dimensions, so each combination is unique per pull.

- **UPSERT via `INSERT OR REPLACE`**: Existing records (same key) get updated, new records get inserted
- **No duplicates**: Primary key constraint prevents duplicate rows across multiple pulls

### Incremental Sync

When pulling data, the start date is calculated automatically:

| Scenario | Start Date | Rationale |
|----------|------------|-----------|
| First sync (no data) | `today - 30 days` | Initial backfill |
| Subsequent syncs | `max_date - 2 days` | 2-day lookback handles GA4 data reprocessing (up to 72h) |
| Manual override | Request body `start_date` | User-specified date |

Configuration constants in `storage_service.rs`:
- `LOOKBACK_DAYS = 2` - Re-pull buffer for GA4 reprocessing
- `DEFAULT_BACKFILL_DAYS = 30` - Initial sync range

### Performance Optimization

| Sync Type | Method | Performance |
|-----------|--------|-------------|
| First sync (empty table) | `bulk_insert()` - DuckDB appender | ~10x faster |
| Incremental sync | `upsert()` - INSERT OR REPLACE | Handles deduplication |

The system detects first sync via `SELECT COUNT(*)` and chooses the appropriate method.
