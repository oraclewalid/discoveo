use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::infrastructure::funnel_repository::{self, FunnelDimension};
use crate::models::connector::ConnectorType;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct FunnelQueryParams {
    pub dimension: FunnelDimension,
    pub start_date: String,
    pub end_date: String,
}

#[instrument(skip(state, params), fields(project_id = %project_id, connector_id = %connector_id))]
async fn funnel(
    State(state): State<AppState>,
    Path((project_id, connector_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<FunnelQueryParams>,
) -> impl IntoResponse {
    info!(
        dimension = ?params.dimension,
        start_date = %params.start_date,
        end_date = %params.end_date,
        "Querying funnel"
    );

    // Verify connector exists and belongs to project
    let connector = state
        .connector_repo
        .find_by_id(connector_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Connector not found"))?;

    if connector.project_id != project_id {
        return Err(AppError::not_found("Connector not found in this project"));
    }

    if connector.connector_type != ConnectorType::Ga4 {
        return Err(AppError::bad_request("Connector is not a GA4 connector"));
    }

    let results = funnel_repository::query_funnel(
        project_id,
        connector_id,
        params.dimension,
        &params.start_date,
        &params.end_date,
    )
    .map_err(AppError::internal)?;

    info!(rows = results.len(), "Funnel query complete");
    Ok(Json(results))
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/projects/{project_id}/connectors/ga4/{connector_id}/funnel",
        get(funnel),
    )
}
