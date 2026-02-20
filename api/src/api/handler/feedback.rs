use axum::{
    extract::{Path, Query, State},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::models::feedback::FeedbackAnalysis;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct FeedbackQuery {
    #[serde(default)]
    pub force: bool,
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn analyze_feedback(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<FeedbackQuery>,
) -> Result<Json<FeedbackAnalysis>, AppError> {
    info!(force = query.force, "Feedback analysis requested");

    // Verify project exists
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    let analysis = state
        .feedback_service
        .generate_feedback(
            project_id,
            query.force,
            &state.survey_repo,
            &state.feedback_repo,
        )
        .await
        .map_err(AppError::internal)?;

    Ok(Json(analysis))
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/projects/{project_id}/qualitative/feedback",
        post(analyze_feedback),
    )
}
