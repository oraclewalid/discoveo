mod api;
mod infrastructure;
mod models;
mod services;

use axum::{routing::get, Router};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::handler::{connector, ga4, project};
use crate::infrastructure::connector_repository::ConnectorRepository;
use crate::infrastructure::project_repository::ProjectRepository;

#[derive(Clone)]
pub struct AppState {
    pub oauth_client: Arc<BasicClient>,
    pub connector_repo: ConnectorRepository,
    pub project_repo: ProjectRepository,
}

async fn health() -> &'static str {
    "OK"
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
    dotenvy::dotenv().ok();

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

    let state = AppState {
        oauth_client: Arc::new(create_oauth_client()),
        connector_repo: ConnectorRepository::new(pool.clone()),
        project_repo: ProjectRepository::new(pool),
    };

    let app = Router::new()
        .route("/health", get(health))
        .merge(project::routes())
        .merge(connector::routes())
        .merge(ga4::routes())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
