use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::models::connector::{Connector, ConnectorType};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateConnectorRequest {
    pub name: String,
    pub connector_type: ConnectorType,
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConnectorRequest {
    pub name: Option<String>,
    pub connector_type: Option<ConnectorType>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DeleteMessage {
    pub message: String,
}

async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<CreateConnectorRequest>,
) -> impl IntoResponse {
    match state.project_repo.find_by_id(project_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(AppError::not_found("Project not found")),
        Err(e) => return Err(AppError::from(e)),
    }

    let connector = Connector {
        id: Uuid::now_v7(),
        project_id,
        name: payload.name,
        connector_type: payload.connector_type,
        config: payload.config,
    };

    state
        .connector_repo
        .create(&connector)
        .await
        .map(|c| (StatusCode::CREATED, Json(c)))
        .map_err(AppError::from)
}

async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.project_repo.find_by_id(project_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return Err(AppError::not_found("Project not found")),
        Err(e) => return Err(AppError::from(e)),
    }

    state
        .connector_repo
        .find_by_project(project_id)
        .await
        .map(Json)
        .map_err(AppError::from)
}

async fn get_by_id(
    State(state): State<AppState>,
    Path((project_id, id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let connector = match state.connector_repo.find_by_id(id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(AppError::not_found("Connector not found")),
        Err(e) => return Err(AppError::from(e)),
    };

    if connector.project_id != project_id {
        return Err(AppError::not_found("Connector not found in this project"));
    }

    Ok(Json(connector))
}

async fn update(
    State(state): State<AppState>,
    Path((project_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateConnectorRequest>,
) -> impl IntoResponse {
    let existing = match state.connector_repo.find_by_id(id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(AppError::not_found("Connector not found")),
        Err(e) => return Err(AppError::from(e)),
    };

    if existing.project_id != project_id {
        return Err(AppError::not_found("Connector not found in this project"));
    }

    let updated = Connector {
        id: existing.id,
        project_id: existing.project_id,
        name: payload.name.unwrap_or(existing.name),
        connector_type: payload.connector_type.unwrap_or(existing.connector_type),
        config: payload.config.unwrap_or(existing.config),
    };

    state
        .connector_repo
        .update(&updated)
        .await
        .map(Json)
        .map_err(AppError::from)
}

async fn delete_connector(
    State(state): State<AppState>,
    Path((project_id, id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let connector = match state.connector_repo.find_by_id(id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(AppError::not_found("Connector not found")),
        Err(e) => return Err(AppError::from(e)),
    };

    if connector.project_id != project_id {
        return Err(AppError::not_found("Connector not found in this project"));
    }

    state
        .connector_repo
        .delete(id)
        .await
        .map(|_| {
            Json(DeleteMessage {
                message: "Connector deleted successfully".to_string(),
            })
        })
        .map_err(AppError::from)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{project_id}/connectors", post(create))
        .route("/projects/{project_id}/connectors", get(list))
        .route("/projects/{project_id}/connectors/{id}", get(get_by_id))
        .route("/projects/{project_id}/connectors/{id}", put(update))
        .route("/projects/{project_id}/connectors/{id}", delete(delete_connector))
}
