use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Json, Redirect},
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse, reqwest::async_http_client};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::models::connector::{Connector, ConnectorDetails, ConnectorType};
use crate::services::ga4_service::{self, PullDataParams};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthUrlResponse {
    pub auth_url: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub connector_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CallbackResponse {
    pub connector_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DisconnectResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GA4Property {
    pub name: String,
    pub display_name: String,
    pub property_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SelectPropertyRequest {
    pub property_id: String,
    pub property_name: String,
}

#[derive(Debug, Serialize)]
pub struct SelectPropertyResponse {
    pub connector_id: Uuid,
    pub property_id: String,
    pub property_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PullDataRequest {
    #[serde(default)]
    pub start_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct PullDataResponse {
    pub success: bool,
    pub file_path: String,
    pub row_count: i64,
}

#[derive(Debug, Deserialize)]
struct GoogleAccountSummariesResponse {
    #[serde(rename = "accountSummaries", default)]
    account_summaries: Vec<AccountSummary>,
}

#[derive(Debug, Deserialize)]
struct AccountSummary {
    #[serde(rename = "propertySummaries", default)]
    property_summaries: Vec<PropertySummary>,
}

#[derive(Debug, Deserialize)]
struct PropertySummary {
    #[serde(default)]
    property: String,
    #[serde(rename = "displayName", default)]
    display_name: String,
    #[serde(rename = "propertyType", default)]
    property_type: Option<String>,
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn auth(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Generating GA4 auth URL");

    match state.project_repo.find_by_id(project_id).await {
        Ok(Some(_)) => debug!("Project found"),
        Ok(None) => {
            warn!("Project not found");
            return Err(AppError::not_found("Project not found"));
        }
        Err(e) => {
            error!(error = %e, "Database error");
            return Err(AppError::from(e));
        }
    }

    let (auth_url, _) = state
        .oauth_client
        .authorize_url(|| CsrfToken::new(project_id.to_string()))
        // Admin API (for listing properties)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/analytics.readonly".to_string(),
        ))
        // Data API (for running reports)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/analytics".to_string(),
        ))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .url();

    debug!(auth_url = %auth_url, "Generated auth URL");
    Ok(Json(AuthUrlResponse {
        auth_url: auth_url.to_string(),
    }))
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn auth_redirect(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Redirecting to GA4 auth");

    match state.project_repo.find_by_id(project_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            warn!("Project not found");
            return Err(AppError::not_found("Project not found"));
        }
        Err(e) => {
            error!(error = %e, "Database error");
            return Err(AppError::from(e));
        }
    }

    let (auth_url, _) = state
        .oauth_client
        .authorize_url(|| CsrfToken::new(project_id.to_string()))
        // Admin API (for listing properties)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/analytics.readonly".to_string(),
        ))
        // Data API (for running reports)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/analytics".to_string(),
        ))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .url();

    debug!(auth_url = %auth_url, "Redirecting to Google OAuth");
    Ok(Redirect::temporary(auth_url.as_str()))
}

#[instrument(skip(state, params), fields(has_code = params.code.len() > 0, has_state = params.state.is_some()))]
async fn callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
) -> impl IntoResponse {
    info!("Processing GA4 OAuth callback");

    let project_id = params
        .state
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            error!("Invalid or missing state parameter");
            AppError::bad_request("Invalid or missing state parameter (project_id)")
        })?;

    debug!(project_id = %project_id, "Extracted project_id from state");

    match state.project_repo.find_by_id(project_id).await {
        Ok(Some(_)) => debug!("Project verified"),
        Ok(None) => {
            warn!(project_id = %project_id, "Project not found");
            return Err(AppError::not_found("Project not found"));
        }
        Err(e) => {
            error!(error = %e, "Database error");
            return Err(AppError::from(e));
        }
    }

    debug!("Exchanging authorization code for tokens");
    let token = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to exchange code for tokens");
            AppError::bad_request(format!("Failed to exchange code: {}", e))
        })?;

    let expires_at = token
        .expires_in()
        .map(|d| Utc::now() + chrono::Duration::seconds(d.as_secs() as i64));

    debug!(
        expires_at = ?expires_at,
        has_refresh_token = token.refresh_token().is_some(),
        "Token exchange successful"
    );

    let config = ConnectorDetails::Ga4 {
        access_token: token.access_token().secret().clone(),
        refresh_token: token.refresh_token().map(|rt| rt.secret().clone()),
        expires_at,
        token_type: "Bearer".to_string(),
        property_id: None,
        property_name: None,
    };

    let connector = Connector {
        id: Uuid::now_v7(),
        project_id,
        name: "GA4 Connector".to_string(),
        connector_type: ConnectorType::Ga4,
        config: serde_json::to_value(&config).unwrap(),
    };

    debug!(connector_id = %connector.id, "Creating connector");
    state
        .connector_repo
        .create(&connector)
        .await
        .map(|c| {
            info!(connector_id = %c.id, "GA4 connector created successfully");
            Json(CallbackResponse {
                connector_id: c.id,
                message: "Successfully connected to GA4".to_string(),
            })
        })
        .map_err(|e| {
            error!(error = %e, "Failed to create connector");
            AppError::from(e)
        })
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn status(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<StatusResponse>, AppError> {
    debug!("Checking GA4 connection status");

    let connectors = state
        .connector_repo
        .find_by_project_and_type(project_id, ConnectorType::Ga4)
        .await?;

    let Some(connector) = connectors.first() else {
        debug!("No GA4 connector found");
        return Ok(Json(StatusResponse {
            connected: false,
            connector_id: None,
            expires_at: None,
        }));
    };

    let config: ConnectorDetails = serde_json::from_value(connector.config.clone())
        .map_err(|_| AppError::internal("Invalid connector config"))?;

    let ConnectorDetails::Ga4 { expires_at, .. } = config;
    let is_expired = expires_at.map(|exp| exp < Utc::now()).unwrap_or(false);

    debug!(
        connector_id = %connector.id,
        is_expired = is_expired,
        expires_at = ?expires_at,
        "Status check complete"
    );

    Ok(Json(StatusResponse {
        connected: !is_expired,
        connector_id: Some(connector.id),
        expires_at,
    }))
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn disconnect(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<DisconnectResponse>, AppError> {
    info!("Disconnecting GA4");

    let connectors = state
        .connector_repo
        .find_by_project_and_type(project_id, ConnectorType::Ga4)
        .await?;

    let Some(connector) = connectors.first() else {
        debug!("No GA4 connector found");
        return Ok(Json(DisconnectResponse {
            message: "No GA4 connector found for this project".to_string(),
        }));
    };

    debug!(connector_id = %connector.id, "Deleting connector");
    state.connector_repo.delete(connector.id).await?;

    info!(connector_id = %connector.id, "GA4 disconnected successfully");
    Ok(Json(DisconnectResponse {
        message: "Successfully disconnected from GA4".to_string(),
    }))
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn properties(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Fetching GA4 properties");

    let connectors = state
        .connector_repo
        .find_by_project_and_type(project_id, ConnectorType::Ga4)
        .await
        .map_err(AppError::from)?;

    let connector = connectors
        .first()
        .ok_or_else(|| {
            warn!("No GA4 connector found");
            AppError::unauthorized("Not connected to GA4. Please authenticate first.")
        })?;

    debug!(connector_id = %connector.id, "Found GA4 connector");

    let config: ConnectorDetails = serde_json::from_value(connector.config.clone())
        .map_err(|_| AppError::internal("Invalid connector config"))?;

    let ConnectorDetails::Ga4 { access_token, expires_at, .. } = config;

    if let Some(exp) = expires_at {
        if exp < Utc::now() {
            warn!(expires_at = ?exp, "Token expired");
            return Err(AppError::unauthorized("Token expired. Please re-authenticate."));
        }
    }

    debug!("Calling Google Analytics Admin API");
    let client = reqwest::Client::new();
    let response = client
        .get("https://analyticsadmin.googleapis.com/v1beta/accountSummaries")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to GA4 API");
            AppError::internal(format!("Failed to connect to GA4 API: {}", e))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!(status = %status, error = %error_text, "GA4 API error");
        return Err(AppError::internal(format!("GA4 API error: {} - {}", status, error_text)));
    }

    let data: GoogleAccountSummariesResponse = response
        .json()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to parse GA4 response");
            AppError::internal(format!("Failed to parse GA4 response: {}", e))
        })?;

    let properties: Vec<GA4Property> = data
        .account_summaries
        .into_iter()
        .flat_map(|account| {
            account.property_summaries.into_iter().map(|prop| GA4Property {
                name: prop.property,
                display_name: prop.display_name,
                property_type: prop.property_type,
            })
        })
        .collect();

    info!(count = properties.len(), "Fetched GA4 properties");
    Ok(Json(properties))
}

#[instrument(skip(state, payload), fields(project_id = %project_id, connector_id = %connector_id))]
async fn select_property(
    State(state): State<AppState>,
    Path((project_id, connector_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<SelectPropertyRequest>,
) -> impl IntoResponse {
    info!(
        property_id = %payload.property_id,
        property_name = %payload.property_name,
        "Selecting GA4 property"
    );

    let connector = match state.connector_repo.find_by_id(connector_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!("Connector not found");
            return Err(AppError::not_found("Connector not found"));
        }
        Err(e) => {
            error!(error = %e, "Database error");
            return Err(AppError::from(e));
        }
    };

    if connector.project_id != project_id {
        warn!("Connector belongs to different project");
        return Err(AppError::not_found("Connector not found in this project"));
    }

    let config: ConnectorDetails = serde_json::from_value(connector.config.clone())
        .map_err(|_| AppError::internal("Invalid connector config"))?;

    let ConnectorDetails::Ga4 { access_token, refresh_token, expires_at, token_type, .. } = config;

    let updated_config = ConnectorDetails::Ga4 {
        access_token,
        refresh_token,
        expires_at,
        token_type,
        property_id: Some(payload.property_id.clone()),
        property_name: Some(payload.property_name.clone()),
    };

    let updated_connector = Connector {
        id: connector.id,
        project_id: connector.project_id,
        name: connector.name,
        connector_type: connector.connector_type,
        config: serde_json::to_value(&updated_config).unwrap(),
    };

    state
        .connector_repo
        .update(&updated_connector)
        .await
        .map_err(AppError::from)?;

    info!("Property selected successfully");
    Ok(Json(SelectPropertyResponse {
        connector_id,
        property_id: payload.property_id,
        property_name: payload.property_name,
    }))
}

#[instrument(skip(state, payload), fields(project_id = %project_id, connector_id = %connector_id))]
async fn pull_data(
    State(state): State<AppState>,
    Path((project_id, connector_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<PullDataRequest>,
) -> impl IntoResponse {
    info!("Starting GA4 data pull");

    // Get the specific connector
    let connector = state
        .connector_repo
        .find_by_id(connector_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| {
            warn!("Connector not found");
            AppError::not_found("Connector not found")
        })?;

    // Verify connector belongs to project
    if connector.project_id != project_id {
        warn!("Connector belongs to different project");
        return Err(AppError::not_found("Connector not found in this project"));
    }

    // Verify it's a GA4 connector
    if connector.connector_type != ConnectorType::Ga4 {
        warn!("Connector is not GA4 type");
        return Err(AppError::bad_request("Connector is not a GA4 connector"));
    }

    let config: ConnectorDetails = serde_json::from_value(connector.config.clone())
        .map_err(|_| AppError::internal("Invalid connector config"))?;

    let ConnectorDetails::Ga4 { access_token, property_id, expires_at, .. } = config;

    // Check token expiration
    if let Some(exp) = expires_at {
        if exp < Utc::now() {
            warn!(expires_at = ?exp, "Token expired");
            return Err(AppError::unauthorized("Token expired. Please re-authenticate."));
        }
    }

    // Check property is selected
    let property_id = property_id.ok_or_else(|| {
        warn!("No property selected");
        AppError::bad_request("No GA4 property selected. Please select a property first.")
    })?;

    debug!(property_id = %property_id, "Pulling data for property");

    // Call the service
    let params = PullDataParams {
        project_id,
        property_id,
        access_token,
        start_date: payload.start_date,
    };

    let result = ga4_service::pull_ga4_data(params)
        .await
        .map_err(AppError::internal)?;

    info!(
        file_path = %result.file_path,
        row_count = result.row_count,
        "Data pull completed"
    );

    Ok(Json(PullDataResponse {
        success: result.success,
        file_path: result.file_path,
        row_count: result.row_count,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{project_id}/connectors/ga4/auth", get(auth))
        .route("/projects/{project_id}/connectors/ga4/auth/redirect", get(auth_redirect))
        .route("/projects/{project_id}/connectors/ga4/status", get(status))
        .route("/projects/{project_id}/connectors/ga4/disconnect", get(disconnect))
        .route("/projects/{project_id}/connectors/ga4/properties", get(properties))
        .route("/projects/{project_id}/connectors/ga4/{connector_id}/property", put(select_property))
        .route("/projects/{project_id}/connectors/ga4/{connector_id}/pull", post(pull_data))
        .route("/connectors/ga4/callback", get(callback))
}
