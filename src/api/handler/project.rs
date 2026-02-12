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
use crate::models::project::Project;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteMessage {
    pub message: String,
}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let project = Project {
        id: Uuid::now_v7(),
        name: payload.name,
        description: payload.description,
    };

    state
        .project_repo
        .create(&project)
        .await
        .map(|p| (StatusCode::CREATED, Json(p)))
        .map_err(AppError::from)
}

async fn list(State(state): State<AppState>) -> impl IntoResponse {
    state
        .project_repo
        .find_all()
        .await
        .map(Json)
        .map_err(AppError::from)
}

async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.project_repo.find_by_id(id).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(AppError::not_found("Project not found")),
        Err(e) => Err(AppError::from(e)),
    }
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProjectRequest>,
) -> impl IntoResponse {
    let existing = match state.project_repo.find_by_id(id).await {
        Ok(Some(p)) => p,
        Ok(None) => return Err(AppError::not_found("Project not found")),
        Err(e) => return Err(AppError::from(e)),
    };

    let updated = Project {
        id: existing.id,
        name: payload.name.unwrap_or(existing.name),
        description: payload.description.or(existing.description),
    };

    state
        .project_repo
        .update(&updated)
        .await
        .map(Json)
        .map_err(AppError::from)
}

async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.project_repo.has_connectors(id).await {
        Ok(true) => {
            return Err(AppError::conflict(
                "Cannot delete project with existing connectors. Delete connectors first.",
            ))
        }
        Ok(false) => {}
        Err(e) => return Err(AppError::from(e)),
    }

    match state.project_repo.delete(id).await {
        Ok(true) => Ok(Json(DeleteMessage {
            message: "Project deleted successfully".to_string(),
        })),
        Ok(false) => Err(AppError::not_found("Project not found")),
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects", post(create))
        .route("/projects", get(list))
        .route("/projects/{id}", get(get_by_id))
        .route("/projects/{id}", put(update))
        .route("/projects/{id}", delete(delete_project))
}
