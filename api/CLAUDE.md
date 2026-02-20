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

Optional:
- `FRONTEND_URL` - Frontend URL for OAuth redirects (default: `http://localhost:5173`)
- `DUCKDB_BASE_PATH` - DuckDB storage path (default: `/tmp/ga4_data`)
- `AWS_BEARER_TOKEN_BEDROCK` - AWS Bedrock token for AI feedback analysis
- `ANTHROPIC_MODEL` - Bedrock model ID (default: `anthropic.claude-sonnet-4-20250514-v1:0`)

## Architecture

Rust API using Axum for a connector management system with analytics, survey analysis, and AI-powered feedback. Projects contain connectors (GA4), qualitative survey data with vector embeddings, and AI-generated feedback analyses.

### Layer Structure

```
main.rs                → App setup, AppState, router composition
├── api/
│   ├── handler/       → HTTP handlers (request/response only)
│   │   ├── project.rs
│   │   ├── connector.rs
│   │   ├── ga4.rs
│   │   ├── funnel.rs
│   │   ├── survey.rs
│   │   ├── feedback.rs
│   │   └── cro.rs
│   └── error.rs       → AppError type with IntoResponse impl
├── services/          → Business logic, orchestration, external API calls
│   ├── connector_service.rs
│   ├── ga4_service.rs
│   ├── storage_service.rs
│   ├── storage_utils.rs
│   ├── embedding_service.rs
│   ├── feedback_service.rs
│   ├── cro_agent_service.rs
│   └── cro_tools.rs
├── infrastructure/    → Repository pattern for database access (SQLx, DuckDB)
│   ├── project_repository.rs
│   ├── connector_repository.rs
│   ├── survey_repository.rs
│   ├── feedback_repository.rs
│   └── funnel_repository.rs
└── models/            → Domain structs with Serialize/Deserialize
    ├── project.rs
    ├── connector.rs
    ├── survey.rs
    ├── feedback.rs
    └── cro_report.rs
```

### Strict Layer Responsibilities

The codebase follows a **Handler → Service → Repository** pattern. Each layer has strict responsibilities:

| Layer | Responsibility | Allowed to call |
|-------|---------------|-----------------|
| **Handler** | Parse request, validate input, return response | Service, Repository |
| **Service** | Business logic, orchestration, external APIs | Repository, other Services |
| **Repository** | Database queries only (SQL) | Nothing |

**Rules:**
- Handlers must NOT contain business logic or direct SQL queries
- Repositories must NOT call other repositories or services
- Services own all business logic: validation rules, orchestration, transformations
- Keep each layer thin — if a handler grows complex, extract logic into a service

## Coding Guidelines

### Functional Programming Paradigm

Prefer functional style over imperative. This means:

**Use combinators instead of loops:**
```rust
// GOOD — use map, filter, flat_map, fold
let names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();

let total = values.iter().fold(0, |acc, v| acc + v);

let all_items: Vec<Item> = groups.iter().flat_map(|g| &g.items).cloned().collect();

// BAD — imperative loop
let mut names = Vec::new();
for p in &projects {
    names.push(p.name.clone());
}
```

**Avoid mutability:**
```rust
// GOOD — immutable bindings, chain transformations
let result = input
    .iter()
    .filter(|x| x.is_valid())
    .map(|x| transform(x))
    .collect::<Vec<_>>();

// BAD — mutable accumulator
let mut result = Vec::new();
for x in &input {
    if x.is_valid() {
        result.push(transform(x));
    }
}
```

**Use `Option`/`Result` combinators instead of `if let` / `match` when possible:**
```rust
// GOOD
let name = user.name.as_deref().unwrap_or("anonymous");
let value = parse(input).map(|v| v * 2).unwrap_or_default();

// OK when side effects or complex branching is needed
match result {
    Ok(data) => process(data),
    Err(e) => log_and_fallback(e),
}
```

### Small Functions

- Each function should do **one thing**
- If a function exceeds ~20 lines, split it into smaller helpers
- Name functions after what they return or what they do
- Prefer pure functions (no side effects) where possible

### Error Handling

- Handlers return `impl IntoResponse`. Use `AppError` for errors
- Use `?` operator with `map_err` for conversions
- `AppError::not_found()`, `AppError::bad_request()`, `AppError::conflict()`, `AppError::internal()`
- `From<sqlx::Error>` is implemented for automatic conversion

```rust
async fn get_by_id(...) -> impl IntoResponse {
    state.repo.find_by_id(id).await.map(Json).map_err(AppError::from)
}
```

### Handler Pattern

Each handler module exports a `routes()` function:
```rust
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects", post(create))
        .route("/projects/{id}", get(get_by_id))
}
```

### General Rules

- **No `clone()` unless necessary** — prefer references and borrows
- **No `unwrap()` in production code** — use `?`, `unwrap_or`, `unwrap_or_default`, or proper error handling
- **No dead code** — delete unused functions, imports, and variables
- **Database**: SQLx with compile-time checked queries. Schema changes require migration + recompile
- **Async**: Use `async`/`await` throughout. Background tasks via `tokio::spawn`

## Data Model

- **Project**: Container for connectors and surveys (id, name, description)
- **Connector**: External service integration belonging to a project (id, project_id, name, type, config JSONB)
- **SurveyResponse**: Qualitative data with vector embeddings (id, project_id, date, country, url, device, browser, os, ratings, comments, comment_embedding, embedding_status)
- **FeedbackAnalysis**: AI-generated analysis cached with 24h TTL (id, project_id, analysis JSONB, narrative, model_used)
- **CroReport**: AI-generated CRO audit combining GA4 + survey + feedback data (id, project_id, connector_id, executive_summary, funnel_analysis, qualitative_insights, recommendations)
- Delete behavior: RESTRICT (can't delete project with connectors)

## API Endpoints

### Projects
`/projects` — CRUD (POST, GET, GET/{id}, PUT/{id}, DELETE/{id})

### Connectors
`/projects/{project_id}/connectors` — CRUD

### GA4
- `/projects/{project_id}/connectors/ga4/auth` — OAuth URL (GET)
- `/projects/{project_id}/connectors/ga4/auth/redirect` — OAuth redirect (GET)
- `/connectors/ga4/callback` — OAuth callback (GET)
- `/projects/{project_id}/connectors/ga4/status` — Token status (GET)
- `/projects/{project_id}/connectors/ga4/properties` — List GA4 properties (GET)
- `/projects/{project_id}/connectors/ga4/{connector_id}/property` — Select property (PUT)
- `/projects/{project_id}/connectors/ga4/disconnect` — Disconnect (GET)
- `/projects/{project_id}/connectors/ga4/{connector_id}/pull` — Pull data (POST)

### Funnel Analytics (DuckDB)
- `/projects/{project_id}/connectors/ga4/{connector_id}/funnel` — Funnel analysis (GET)
- `/projects/{project_id}/connectors/ga4/{connector_id}/scroll-depth` — Scroll depth (GET)
- `/projects/{project_id}/connectors/ga4/{connector_id}/page-paths` — Page paths (GET)
- `/projects/{project_id}/connectors/ga4/{connector_id}/debug/events` — Event names (GET)

### Surveys & Qualitative
- `/projects/{project_id}/qualitative/surveys` — Upload CSV (POST multipart)
- `/projects/{project_id}/qualitative/stats` — Statistics (GET)
- `/projects/{project_id}/qualitative/embeddings/status` — Embedding progress (GET)
- `/projects/{project_id}/qualitative/comments/search` — Vector similarity search (POST)

### AI Feedback
- `/projects/{project_id}/qualitative/feedback` — Generate/retrieve analysis (POST, `?force=true` to skip cache)

### CRO Report (AI Agent)
- `/projects/{project_id}/cro/report` — Generate full CRO audit (POST, auto-detects GA4 connector, persisted to DB)
- `/projects/{project_id}/cro/reports` — List all CRO reports for project (GET)
- `/projects/{project_id}/cro/reports/{report_id}` — Get specific report (GET)

## Dual Database Strategy

**PostgreSQL** (via SQLx) — Transactional data: projects, connectors, surveys, embeddings (pgvector), feedback analyses

**DuckDB** — Analytical data: GA4 events and page paths, stored per connector at:
```
{DUCKDB_BASE_PATH}/{project_id}/{connector_id}/ga4.duckdb
```

### GA4 DuckDB Tables

**ga4_events** — Composite PK: (date, country, device_category, event_name, browser, operating_system, screen_resolution)

**ga4_page_paths** — Composite PK: (date, page_path)

### Incremental Sync

| Scenario | Start Date | Rationale |
|----------|------------|-----------|
| First sync | `today - 90 days` | Initial backfill |
| Subsequent syncs | `max_date - 2 days` | Lookback for GA4 reprocessing |
| Manual override | Request body `start_date` | User-specified |

- `LOOKBACK_DAYS = 2`, `DEFAULT_BACKFILL_DAYS = 90` (in `storage_service.rs`)
- First sync uses `bulk_insert()` (DuckDB appender, ~10x faster)
- Incremental sync uses `upsert()` (INSERT OR REPLACE for deduplication)

## GA4 OAuth Flow

1. `GET /projects/{id}/connectors/ga4/auth` → Returns Google OAuth URL (project_id in state param)
2. User authorizes on Google
3. `GET /connectors/ga4/callback?code=...&state=...` → Exchanges code for tokens, creates connector
4. Tokens stored in connector's `config` JSONB field
5. Token refresh handled automatically when expired

## CRO Agent (AI-Powered Conversion Audit)

The CRO agent is a ReAct-style AI agent that combines GA4 quantitative data, survey qualitative data, and LLM feedback analysis to produce a comprehensive CRO (Conversion Rate Optimization) report.

### Architecture

```
POST /projects/{project_id}/cro/report?connector_id=...
    │
    ▼
cro handler → CroAgentService (ReAct loop) → Bedrock Claude (tool_use)
                    │                              │
                    │  ┌───────────────────────────┘
                    │  │ tool calls
                    ▼  ▼
              Tool execution (cro_tools.rs)
              ├── get_funnel_overview      → funnel_repository
              ├── compare_periods          → funnel_repository (×2, diff)
              ├── get_page_paths           → funnel_repository
              ├── get_drop_off_points      → funnel_repository (sorted)
              ├── search_survey_comments   → embedding_service + survey_repo
              ├── get_survey_by_period     → survey_repo (date-filtered)
              ├── get_survey_stats         → survey_repo
              └── get_feedback_themes      → feedback_repo
```

### Agent Loop

1. Handler builds a `ToolContext` with project/connector IDs and repo references
2. `CroAgentService` sends initial message to Bedrock with 8 tool definitions
3. **Loop** (max 15 turns):
   - Bedrock returns `tool_use` blocks → agent executes each tool → sends results back
   - Bedrock returns `end_turn` → agent extracts final JSON report → loop ends
4. Final text is parsed into `CroReport` (structured JSON + narrative)

### Agent Tools

| Tool | Purpose | Data Source |
|------|---------|-------------|
| `get_funnel_overview` | Full funnel for a date range | DuckDB `ga4_events` |
| `compare_periods` | Side-by-side funnel comparison | DuckDB `ga4_events` (×2) |
| `get_page_paths` | Page-level metrics | DuckDB `ga4_page_paths` |
| `get_drop_off_points` | Biggest funnel drops (sorted) | DuckDB `ga4_events` |
| `search_survey_comments` | Semantic search on comments | pgvector cosine similarity |
| `get_survey_by_period` | Comments filtered by date | PostgreSQL `survey_responses` |
| `get_survey_stats` | Overall survey statistics | PostgreSQL `survey_responses` |
| `get_feedback_themes` | Latest LLM feedback themes | PostgreSQL `feedback_analyses` |

### Key Design Decisions

- **Tool-based** (not single-prompt): The agent decides what to investigate, enabling temporal correlation (e.g. detect funnel drop in week X → pull survey comments from same period)
- **Bedrock `tool_use` API**: Messages use `content: Vec<ContentBlock>` with `text`, `tool_use`, and `tool_result` block types
- **Stateless**: No caching — each request runs a fresh agent loop
- **Graceful degradation**: If no survey data exists, the agent still produces a report from GA4 data only

## Embedding Pipeline

1. CSV uploaded via `/qualitative/surveys` (multipart)
2. Responses inserted in PostgreSQL with `embedding_status = 'pending'`
3. Background `tokio::spawn` task generates embeddings (FastEmbed, MultilingualE5Base, 768 dimensions)
4. Embeddings stored in pgvector column, status updated to `completed`/`failed`/`skipped`
5. Vector similarity search via cosine distance (`<=>` operator)
