use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct GA4RunReportRequest {
    #[serde(rename = "dateRanges")]
    date_ranges: Vec<DateRange>,
    dimensions: Vec<Dimension>,
    metrics: Vec<Metric>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct GA4ReportResponse {
    #[serde(rename = "dimensionHeaders", default)]
    pub dimension_headers: Vec<Header>,
    #[serde(rename = "metricHeaders", default)]
    pub metric_headers: Vec<Header>,
    #[serde(default)]
    pub rows: Vec<Row>,
    #[serde(rename = "rowCount", default)]
    pub row_count: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Header {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Row {
    #[serde(rename = "dimensionValues", default)]
    pub dimension_values: Vec<Value>,
    #[serde(rename = "metricValues", default)]
    pub metric_values: Vec<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    pub value: String,
}

#[derive(Debug)]
pub struct PullDataParams {
    pub project_id: Uuid,
    pub property_id: String,
    pub access_token: String,
    pub start_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct PullDataResult {
    pub success: bool,
    pub file_path: String,
    pub row_count: i64,
}

fn default_report(start_date: &str, end_date: &str) -> GA4RunReportRequest {
    GA4RunReportRequest {
        date_ranges: vec![DateRange {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
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
    }
}

pub async fn pull_ga4_data(params: PullDataParams) -> Result<PullDataResult, String> {
    let start_date = params
        .start_date
        .unwrap_or_else(|| (Utc::now() - Duration::days(90)).date_naive());
    let end_date = Utc::now().date_naive();

    info!(
        project_id = %params.project_id,
        property_id = %params.property_id,
        start_date = %start_date,
        end_date = %end_date,
        "Starting GA4 data pull"
    );

    let request = default_report(
        &start_date.format("%Y-%m-%d").to_string(),
        &end_date.format("%Y-%m-%d").to_string(),
    );

    debug!(request = ?request, "GA4 report request");

    let client = reqwest::Client::new();
    let url = format!(
        "https://analyticsdata.googleapis.com/v1beta/{}:runReport",
        params.property_id
    );

    let response = client
        .post(&url)
        .bearer_auth(&params.access_token)
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to call GA4 Data API");
            format!("Failed to call GA4 Data API: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!(status = %status, error = %error_text, "GA4 Data API error");
        return Err(format!("GA4 Data API error: {} - {}", status, error_text));
    }

    let report: GA4ReportResponse = response.json().await.map_err(|e| {
        error!(error = %e, "Failed to parse GA4 response");
        format!("Failed to parse GA4 response: {}", e)
    })?;

    let row_count = report.row_count.unwrap_or(report.rows.len() as i64);
    info!(row_count = row_count, "GA4 data fetched successfully");

    // Create output directory
    let output_dir = PathBuf::from(format!("/tmp/{}", params.project_id));
    fs::create_dir_all(&output_dir).await.map_err(|e| {
        error!(error = %e, path = ?output_dir, "Failed to create output directory");
        format!("Failed to create output directory: {}", e)
    })?;

    // Generate filename with timestamp
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("ga4_report_{}.json", timestamp);
    let file_path = output_dir.join(&filename);

    // Write data to file
    let json_data = serde_json::to_string_pretty(&report).map_err(|e| {
        error!(error = %e, "Failed to serialize report");
        format!("Failed to serialize report: {}", e)
    })?;

    fs::write(&file_path, json_data).await.map_err(|e| {
        error!(error = %e, path = ?file_path, "Failed to write report file");
        format!("Failed to write report file: {}", e)
    })?;

    info!(file_path = ?file_path, "Report written successfully");

    Ok(PullDataResult {
        success: true,
        file_path: file_path.to_string_lossy().to_string(),
        row_count,
    })
}
