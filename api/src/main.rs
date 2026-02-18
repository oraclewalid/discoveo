mod api;
mod infrastructure;
mod models;
mod services;

use axum::{routing::get, Router};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::handler::{connector, feedback, funnel, ga4, project, survey};
use crate::infrastructure::connector_repository::ConnectorRepository;
use crate::infrastructure::feedback_repository::FeedbackRepository;
use crate::infrastructure::project_repository::ProjectRepository;
use crate::infrastructure::survey_repository::SurveyRepository;
use crate::services::connector_service::ConnectorService;
use crate::services::embedding_service::EmbeddingService;
use crate::services::feedback_service::FeedbackService;

#[derive(Clone)]
pub struct AppState {
    pub oauth_client: Arc<BasicClient>,
    pub connector_repo: ConnectorRepository,
    pub connector_service: ConnectorService,
    pub project_repo: ProjectRepository,
    pub survey_repo: SurveyRepository,
    pub feedback_repo: FeedbackRepository,
    pub embedding_service: EmbeddingService,
    pub feedback_service: FeedbackService,
    pub frontend_url: String,
    pub duckdb_base_path: String,
    pub pool: PgPool,
}

async fn health() -> &'static str {
    "OK"
}

/// Mask credentials in a database URL for safe logging
fn mask_url(url: &str) -> String {
    // postgres://user:password@host:port/db -> postgres://***:***@host:port/db
    if let Some(at_pos) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            return format!("{}://***:***@{}", &url[..scheme_end], &url[at_pos + 1..]);
        }
    }
    "***".to_string()
}

fn create_oauth_client() -> BasicClient {
    let client_id =
        std::env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID environment variable");
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .expect("Missing GOOGLE_CLIENT_SECRET environment variable");
    let redirect_url = std::env::var("GOOGLE_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:3000/connectors/ga4/callback".to_string());

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting server...");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let duckdb_base_path =
        std::env::var("DUCKDB_BASE_PATH").unwrap_or_else(|_| "/tmp/ga4_data".to_string());

    let connector_repo = ConnectorRepository::new(pool.clone());
    let connector_service = ConnectorService::new(
        connector_repo.clone(),
        duckdb_base_path.clone(),
    );

    let embedding_service = EmbeddingService::new()
        .expect("Failed to initialize embedding service");

    let bedrock_token = std::env::var("AWS_BEARER_TOKEN_BEDROCK").ok();
    let anthropic_model = std::env::var("ANTHROPIC_MODEL").ok();
    let feedback_service = FeedbackService::new(bedrock_token, anthropic_model);

    // Log startup configuration
    tracing::info!("=== Startup Configuration ===");
    tracing::info!(database_url = %mask_url(&database_url), "Database");
    tracing::info!(frontend_url = %frontend_url, "Frontend");
    tracing::info!(duckdb_base_path = %duckdb_base_path, "DuckDB storage");
    tracing::info!(
        google_oauth_redirect = %std::env::var("GOOGLE_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:3000/connectors/ga4/callback".to_string()),
        google_client_id_set = true,
        "OAuth"
    );
    tracing::info!(
        feedback_model = %feedback_service.model_id(),
        bedrock_token_set = feedback_service.has_bearer_token(),
        "Feedback analysis (Bedrock)"
    );
    tracing::info!(
        embedding_model = "MultilingualE5Base",
        "Embedding service (FastEmbed)"
    );
    tracing::info!(
        max_connections = 5,
        cors = "permissive (allow all)",
        bind = "0.0.0.0:3000",
        "Server"
    );
    tracing::info!("=== Configuration loaded ===");

    let state = AppState {
        oauth_client: Arc::new(create_oauth_client()),
        connector_repo,
        connector_service,
        project_repo: ProjectRepository::new(pool.clone()),
        survey_repo: SurveyRepository::new(pool.clone()),
        feedback_repo: FeedbackRepository::new(pool.clone()),
        embedding_service,
        feedback_service,
        frontend_url,
        duckdb_base_path,
        pool,
    };

    let app = Router::new()
        .route("/health", get(health))
        .merge(project::routes())
        .merge(connector::routes())
        .merge(ga4::routes())
        .merge(funnel::routes())
        .merge(survey::routes())
        .merge(feedback::routes())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
