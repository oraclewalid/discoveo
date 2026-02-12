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
