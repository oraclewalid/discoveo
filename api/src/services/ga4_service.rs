use chrono::{DateTime, Duration, NaiveDate, Utc};
use oauth2::{RefreshToken, TokenResponse, basic::BasicClient, reqwest::async_http_client};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

// GA4 API request types
#[derive(Debug, Serialize)]
struct RunReportRequest {
    #[serde(rename = "dateRanges")]
    date_ranges: Vec<DateRange>,
    dimensions: Vec<Dimension>,
    metrics: Vec<Metric>,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Serialize)]
struct DateRange {
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}

#[derive(Debug, Serialize)]
struct Dimension {
    name: String,
}

#[derive(Debug, Serialize)]
struct Metric {
    name: String,
}

// GA4 API response types
#[derive(Debug, Deserialize)]
struct RunReportResponse {
    #[serde(default)]
    rows: Vec<Row>,
    #[serde(rename = "rowCount", default)]
    row_count: i64,
}

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "dimensionValues", default)]
    dimension_values: Vec<Value>,
    #[serde(rename = "metricValues", default)]
    metric_values: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct Value {
    value: String,
}

// Flat record for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GA4Record {
    pub date: String,
    pub country: String,
    pub device_category: String,
    pub event_name: String,
    pub browser: String,
    pub operating_system: String,
    pub screen_resolution: String,
    pub active_users: i64,
    pub sessions: i64,
    pub screen_page_views: i64,
    pub bounce_rate: f64,
    pub average_session_duration: f64,
}

pub struct PullParams {
    pub property_id: String,
    pub access_token: String,
    pub start_date: Option<NaiveDate>,
}

const PAGE_SIZE: i64 = 10000;

pub async fn pull(params: PullParams) -> Result<Vec<GA4Record>, String> {
    let start_date = params
        .start_date
        .unwrap_or_else(|| (Utc::now() - Duration::days(90)).date_naive());
    let end_date = Utc::now().date_naive();

    info!(
        property_id = %params.property_id,
        start_date = %start_date,
        end_date = %end_date,
        "Pulling GA4 data"
    );

    let mut all_records = Vec::new();
    let mut offset: i64 = 0;
    let mut total_rows: Option<i64> = None;

    loop {
        let request = build_request(&start_date, &end_date, offset);
        let response = call_api(&params.property_id, &params.access_token, &request).await?;

        if total_rows.is_none() {
            total_rows = Some(response.row_count);
            info!(total_rows = response.row_count, "Total rows to fetch");
        }

        let page_count = response.rows.len();
        let records = flatten(response);
        all_records.extend(records);

        info!(
            offset = offset,
            page_count = page_count,
            fetched = all_records.len(),
            total = total_rows.unwrap_or(0),
            "Fetched page"
        );

        if page_count < PAGE_SIZE as usize {
            break;
        }
        offset += PAGE_SIZE;
    }

    info!(record_count = all_records.len(), "GA4 data pull complete");
    Ok(all_records)
}

fn build_request(start_date: &NaiveDate, end_date: &NaiveDate, offset: i64) -> RunReportRequest {
    RunReportRequest {
        date_ranges: vec![DateRange {
            start_date: start_date.format("%Y-%m-%d").to_string(),
            end_date: end_date.format("%Y-%m-%d").to_string(),
        }],
        dimensions: vec![
            Dimension { name: "date".to_string() },
            Dimension { name: "country".to_string() },
            Dimension { name: "deviceCategory".to_string() },
            Dimension { name: "eventName".to_string() },
            Dimension { name: "browser".to_string() },
            Dimension { name: "operatingSystem".to_string() },
            Dimension { name: "screenResolution".to_string() },
        ],
        metrics: vec![
            Metric { name: "activeUsers".to_string() },
            Metric { name: "sessions".to_string() },
            Metric { name: "screenPageViews".to_string() },
            Metric { name: "bounceRate".to_string() },
            Metric { name: "averageSessionDuration".to_string() },
        ],
        limit: PAGE_SIZE,
        offset,
    }
}

async fn call_api(
    property_id: &str,
    access_token: &str,
    request: &RunReportRequest,
) -> Result<RunReportResponse, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://analyticsdata.googleapis.com/v1beta/{}:runReport",
        property_id
    );

    debug!("Calling GA4 Data API");

    let response = client
        .post(&url)
        .bearer_auth(access_token)
        .json(request)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to call GA4 API");
            format!("Failed to call GA4 API: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!(status = %status, error = %error_text, "GA4 API error");
        return Err(format!("GA4 API error: {} - {}", status, error_text));
    }

    response.json().await.map_err(|e| {
        error!(error = %e, "Failed to parse GA4 response");
        format!("Failed to parse GA4 response: {}", e)
    })
}

fn flatten(response: RunReportResponse) -> Vec<GA4Record> {
    response
        .rows
        .into_iter()
        .map(|row| {
            let dims = &row.dimension_values;
            let metrics = &row.metric_values;

            GA4Record {
                date: dims.get(0).map(|v| v.value.clone()).unwrap_or_default(),
                country: dims.get(1).map(|v| v.value.clone()).unwrap_or_default(),
                device_category: dims.get(2).map(|v| v.value.clone()).unwrap_or_default(),
                event_name: dims.get(3).map(|v| v.value.clone()).unwrap_or_default(),
                browser: dims.get(4).map(|v| v.value.clone()).unwrap_or_default(),
                operating_system: dims.get(5).map(|v| v.value.clone()).unwrap_or_default(),
                screen_resolution: dims.get(6).map(|v| v.value.clone()).unwrap_or_default(),
                active_users: parse_i64(metrics.get(0)),
                sessions: parse_i64(metrics.get(1)),
                screen_page_views: parse_i64(metrics.get(2)),
                bounce_rate: parse_f64(metrics.get(3)),
                average_session_duration: parse_f64(metrics.get(4)),
            }
        })
        .collect()
}

fn parse_i64(value: Option<&Value>) -> i64 {
    value
        .and_then(|v| v.value.parse().ok())
        .unwrap_or(0)
}

fn parse_f64(value: Option<&Value>) -> f64 {
    value
        .and_then(|v| v.value.parse().ok())
        .unwrap_or(0.0)
}

// Token refresh
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub fn is_token_expired(expires_at: Option<DateTime<Utc>>) -> bool {
    expires_at.map(|exp| exp < Utc::now()).unwrap_or(false)
}

pub async fn refresh_token(
    oauth_client: &BasicClient,
    refresh_token: &str,
) -> Result<TokenInfo, String> {
    warn!("Access token expired, refreshing...");

    let token = oauth_client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to refresh token");
            format!("Failed to refresh token: {}", e)
        })?;

    let expires_at = token
        .expires_in()
        .map(|d| Utc::now() + Duration::seconds(d.as_secs() as i64));

    info!(
        expires_at = ?expires_at,
        "Token refreshed successfully"
    );

    Ok(TokenInfo {
        access_token: token.access_token().secret().clone(),
        refresh_token: token.refresh_token().map(|rt| rt.secret().clone()),
        expires_at,
    })
}
