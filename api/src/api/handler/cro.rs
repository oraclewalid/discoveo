use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::models::connector::ConnectorType;
use crate::models::cro_report::CroReport;
use crate::services::cro_tools::ToolContext;
use crate::AppState;

#[instrument(skip(state), fields(project_id = %project_id))]
async fn generate_report(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<CroReport>, AppError> {
    info!("CRO report requested");

    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    let connector = state
        .connector_repo
        .find_by_project_and_type(project_id, ConnectorType::Ga4)
        .await
        .map_err(AppError::from)?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::not_found("No GA4 connector found for this project"))?;

    let connector_id = connector.id;
    info!(connector_id = %connector_id, "Found GA4 connector");

    let ctx = ToolContext {
        project_id,
        connector_id,
        duckdb_base_path: state.duckdb_base_path.clone(),
        survey_repo: state.survey_repo.clone(),
        feedback_repo: state.feedback_repo.clone(),
        embedding_service: state.embedding_service.clone(),
    };

    let report = state
        .cro_agent_service
        .generate_report(project_id, connector_id, ctx)
        .await
        .map_err(AppError::internal)?;

    if let Err(e) = state.cro_repo.insert(&report).await {
        warn!(error = %e, "Failed to persist CRO report");
    }

    Ok(Json(report))
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn list_reports(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<CroReport>>, AppError> {
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    let reports = state
        .cro_repo
        .find_by_project(project_id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(reports))
}

#[instrument(skip(state), fields(project_id = %project_id, report_id = %report_id))]
async fn get_report(
    State(state): State<AppState>,
    Path((project_id, report_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CroReport>, AppError> {
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    let report = state
        .cro_repo
        .find_by_id(report_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("CRO report not found"))?;

    if report.project_id != project_id {
        return Err(AppError::not_found("CRO report not found"));
    }

    Ok(Json(report))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{project_id}/cro/report", post(generate_report))
        .route("/projects/{project_id}/cro/reports", get(list_reports))
        .route(
            "/projects/{project_id}/cro/reports/{report_id}",
            get(get_report),
        )
}
