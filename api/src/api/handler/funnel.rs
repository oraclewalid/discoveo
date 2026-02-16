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

#[derive(Debug, Deserialize)]
pub struct ScrollQueryParams {
    pub dimension: FunnelDimension,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize)]
pub struct PagePathQueryParams {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Deserialize)]
pub struct DebugQueryParams {
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
        &state.duckdb_base_path,
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

#[instrument(skip(state, params), fields(project_id = %project_id, connector_id = %connector_id))]
async fn scroll_depth(
    State(state): State<AppState>,
    Path((project_id, connector_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<ScrollQueryParams>,
) -> impl IntoResponse {
    info!(
        dimension = ?params.dimension,
        start_date = %params.start_date,
        end_date = %params.end_date,
        "Querying scroll depth"
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

    let results = funnel_repository::query_scroll_depth(
        &state.duckdb_base_path,
        project_id,
        connector_id,
        params.dimension,
        &params.start_date,
        &params.end_date,
    )
    .map_err(AppError::internal)?;

    info!(rows = results.len(), "Scroll depth query complete");
    Ok(Json(results))
}

#[instrument(skip(state, params), fields(project_id = %project_id, connector_id = %connector_id))]
async fn page_paths(
    State(state): State<AppState>,
    Path((project_id, connector_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<PagePathQueryParams>,
) -> impl IntoResponse {
    info!(
        start_date = %params.start_date,
        end_date = %params.end_date,
        "Querying page path analytics"
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

    let results = funnel_repository::query_page_paths(
        &state.duckdb_base_path,
        project_id,
        connector_id,
        &params.start_date,
        &params.end_date,
    )
    .map_err(AppError::internal)?;

    info!(rows = results.len(), "Page path analytics query complete");
    Ok(Json(results))
}

#[instrument(skip(state, params), fields(project_id = %project_id, connector_id = %connector_id))]
async fn debug_events(
    State(state): State<AppState>,
    Path((project_id, connector_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<DebugQueryParams>,
) -> impl IntoResponse {
    info!(
        start_date = %params.start_date,
        end_date = %params.end_date,
        "Querying debug event names"
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

    let results = funnel_repository::query_event_names(
        &state.duckdb_base_path,
        project_id,
        connector_id,
        &params.start_date,
        &params.end_date,
    )
    .map_err(AppError::internal)?;

    info!(event_names = results.len(), "Debug event names query complete");
    Ok(Json(results))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/connectors/ga4/{connector_id}/funnel",
            get(funnel),
        )
        .route(
            "/projects/{project_id}/connectors/ga4/{connector_id}/scroll-depth",
            get(scroll_depth),
        )
        .route(
            "/projects/{project_id}/connectors/ga4/{connector_id}/page-paths",
            get(page_paths),
        )
        .route(
            "/projects/{project_id}/connectors/ga4/{connector_id}/debug/events",
            get(debug_events),
        )
}
