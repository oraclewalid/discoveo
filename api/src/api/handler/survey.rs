use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::api::error::AppError;
use crate::models::survey::{SimilarComment, SurveyResponse, SurveyStats};
use crate::services::embedding_service;
use crate::AppState;

const REQUIRED_COLUMNS: &[&str] = &[
    "Date",
    "Country",
    "URL",
    "Device",
    "Browser",
    "OS",
    "Comments",
];

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub message: String,
    pub row_count: usize,
    pub inserted_count: u64,
    pub columns: Vec<String>,
}

#[instrument(skip(state, multipart), fields(project_id = %project_id))]
async fn upload_survey(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("Receiving survey CSV upload");

    // Verify project exists
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Extract CSV file from multipart form
    let mut csv_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to read multipart field — possible body size limit exceeded");
            AppError::bad_request(format!("Failed to read multipart field: {}", e))
        })?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let content_type = field.content_type().map(|s| s.to_string());
            let file_name = field.file_name().map(|s| s.to_string());
            info!(field_name = %name, ?content_type, ?file_name, "Reading file field");
            let bytes = field
                .bytes()
                .await
                .map_err(|e| {
                    warn!(error = %e, "Failed to read file bytes — possible body size limit exceeded");
                    AppError::bad_request(format!("Failed to read file: {}", e))
                })?;
            info!(file_size_bytes = bytes.len(), "File field read successfully");
            csv_bytes = Some(bytes.to_vec());
        }
    }

    let csv_bytes = csv_bytes.ok_or_else(|| {
        AppError::bad_request("No file field found in the request. Send a multipart form with a 'file' field.")
    })?;

    if csv_bytes.is_empty() {
        return Err(AppError::bad_request("Uploaded file is empty"));
    }

    info!(file_size = csv_bytes.len(), "CSV file received");

    // Parse and validate CSV headers
    let mut reader = csv::Reader::from_reader(csv_bytes.as_slice());

    let headers = reader
        .headers()
        .map_err(|e| AppError::bad_request(format!("Failed to parse CSV headers: {}", e)))?
        .clone();

    let found_columns: Vec<String> = headers.iter().map(|h| h.trim().to_string()).collect();

    // Check for missing required columns
    let missing_columns: Vec<String> = REQUIRED_COLUMNS
        .iter()
        .filter(|required| !found_columns.iter().any(|found| found == **required))
        .map(|s| s.to_string())
        .collect();

    if !missing_columns.is_empty() {
        warn!(
            missing = ?missing_columns,
            found = ?found_columns,
            "CSV is missing required columns"
        );
        return Err(AppError::bad_request(format!(
            "CSV is missing required columns: {}. Found columns: {}",
            missing_columns.join(", "),
            found_columns.join(", ")
        )));
    }

    // Find column indices
    let col_index = |name: &str| -> Option<usize> {
        found_columns.iter().position(|c| c == name)
    };

    let idx_date = col_index("Date");
    let idx_country = col_index("Country");
    let idx_url = col_index("URL");
    let idx_device = col_index("Device");
    let idx_browser = col_index("Browser");
    let idx_os = col_index("OS");
    let idx_ratings = col_index("Ratings");
    let idx_comments = col_index("Comments");

    // Identify extra columns (not in REQUIRED_COLUMNS)
    let extra_columns: Vec<(usize, String)> = found_columns
        .iter()
        .enumerate()
        .filter(|(_, name)| !REQUIRED_COLUMNS.contains(&name.as_str()))
        .map(|(i, name)| (i, name.clone()))
        .collect();

    // Parse rows and build SurveyResponse objects
    let mut responses: Vec<SurveyResponse> = Vec::new();

    for result in reader.records() {
        let record = result
            .map_err(|e| AppError::bad_request(format!("Failed to parse CSV row: {}", e)))?;

        let get = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| record.get(i))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };

        // Parse ratings as f64, supporting both "2.3" and "2,3" formats
        let ratings = get(idx_ratings).and_then(|v| {
            v.replace(',', ".").parse::<f64>().ok()
        });

        // Parse date, trying multiple formats
        let date = get(idx_date).and_then(|v| parse_date(&v));

        // Build raw from extra columns
        let mut raw = Map::new();
        for (idx, col_name) in &extra_columns {
            if let Some(val) = record.get(*idx) {
                let val = val.trim();
                if !val.is_empty() {
                    raw.insert(col_name.clone(), Value::String(val.to_string()));
                }
            }
        }

        responses.push(SurveyResponse {
            id: Uuid::now_v7(),
            project_id,
            date,
            country: get(idx_country),
            url: get(idx_url),
            device: get(idx_device),
            browser: get(idx_browser),
            os: get(idx_os),
            ratings,
            comments: get(idx_comments),
            raw: Value::Object(raw),
            comment_embedding: None,
            embedding_status: None,
            embedding_generated_at: None,
        });
    }

    let row_count = responses.len();

    info!(
        row_count = row_count,
        extra_columns = ?extra_columns.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>(),
        "CSV parsed, inserting into database"
    );

    // Insert into PostgreSQL
    let inserted = state
        .survey_repo
        .insert_batch(&responses)
        .await
        .map_err(AppError::from)?;

    info!(
        inserted = inserted,
        "Survey responses inserted successfully"
    );

    // Spawn background task to generate embeddings
    let project_id_clone = project_id;
    let embedding_service = state.embedding_service.clone();
    let survey_repo = state.survey_repo.clone();

    tokio::spawn(async move {
        embedding_service::generate_embeddings_for_project(
            project_id_clone,
            embedding_service,
            survey_repo,
        )
        .await;
    });

    info!(
        project_id = %project_id,
        "Background embedding generation started"
    );

    Ok(Json(UploadResponse {
        message: "Survey CSV uploaded and saved successfully. Embeddings are being generated in the background.".to_string(),
        row_count,
        inserted_count: inserted,
        columns: found_columns,
    }))
}

/// Try parsing a date/datetime string in multiple common formats
fn parse_date(s: &str) -> Option<NaiveDateTime> {
    // Try datetime formats first
    let datetime_formats = [
        "%Y-%m-%d %H:%M:%S",   // 2024-11-08 17:49:09
        "%Y-%m-%dT%H:%M:%S",   // 2024-11-08T17:49:09
        "%d/%m/%Y %H:%M:%S",   // 08/11/2024 17:49:09
        "%m/%d/%Y %H:%M:%S",   // 11/08/2024 17:49:09
        "%Y-%m-%d %H:%M",      // 2024-11-08 17:49
    ];

    for fmt in &datetime_formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }

    // Fallback to date-only formats (set time to midnight)
    let date_formats = [
        "%Y-%m-%d",       // 2026-02-16
        "%d/%m/%Y",       // 16/02/2026
        "%m/%d/%Y",       // 02/16/2026
        "%Y/%m/%d",       // 2026/02/16
        "%d-%m-%Y",       // 16-02-2026
        "%d.%m.%Y",       // 16.02.2026
        "%Y%m%d",         // 20260216
    ];

    for fmt in &date_formats {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d.and_hms_opt(0, 0, 0).unwrap());
        }
    }

    None
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn get_stats(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<SurveyStats>, AppError> {
    info!("Fetching survey statistics");

    // Verify project exists
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Get stats
    let stats = state
        .survey_repo
        .get_stats(project_id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(stats))
}

#[derive(Debug, Serialize)]
pub struct EmbeddingStatusResponse {
    pub total_responses: i64,
    pub pending: i64,
    pub completed: i64,
    pub failed: i64,
    pub skipped: i64,
}

#[instrument(skip(state), fields(project_id = %project_id))]
async fn get_embedding_status(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<EmbeddingStatusResponse>, AppError> {
    info!("Fetching embedding generation status");

    // Verify project exists
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Query status counts
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE embedding_status = 'pending') as pending,
            COUNT(*) FILTER (WHERE embedding_status = 'completed') as completed,
            COUNT(*) FILTER (WHERE embedding_status = 'failed') as failed,
            COUNT(*) FILTER (WHERE embedding_status = 'skipped') as skipped
        FROM survey_responses
        WHERE project_id = $1
        "#,
    )
    .bind(project_id)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row;
    Ok(Json(EmbeddingStatusResponse {
        total_responses: row.try_get::<i64, _>("total").unwrap_or(0),
        pending: row.try_get::<i64, _>("pending").unwrap_or(0),
        completed: row.try_get::<i64, _>("completed").unwrap_or(0),
        failed: row.try_get::<i64, _>("failed").unwrap_or(0),
        skipped: row.try_get::<i64, _>("skipped").unwrap_or(0),
    }))
}

#[derive(Debug, Deserialize)]
pub struct SimilaritySearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_min_similarity")]
    pub min_similarity: f64,
}

fn default_limit() -> i64 {
    10
}
fn default_min_similarity() -> f64 {
    0.5
}

#[derive(Debug, Serialize)]
pub struct SimilaritySearchResponse {
    pub query: String,
    pub results: Vec<SimilarComment>,
}

#[instrument(skip(state, req), fields(project_id = %project_id))]
async fn search_similar_comments(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<SimilaritySearchRequest>,
) -> Result<Json<SimilaritySearchResponse>, AppError> {
    info!(query = %req.query, "Searching for similar comments");

    // Verify project exists
    state
        .project_repo
        .find_by_id(project_id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Generate embedding for query
    let query_embedding = state
        .embedding_service
        .generate_embedding(&req.query)
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::bad_request("Query text is empty"))?;

    // Search for similar comments
    let results = state
        .survey_repo
        .find_similar_comments(project_id, query_embedding, req.limit, req.min_similarity)
        .await
        .map_err(AppError::from)?;

    info!(result_count = results.len(), "Found similar comments");

    Ok(Json(SimilaritySearchResponse {
        query: req.query,
        results,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/qualitative/surveys",
            post(upload_survey),
        )
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB limit for CSV uploads
        .route(
            "/projects/{project_id}/qualitative/stats",
            get(get_stats),
        )
        .route(
            "/projects/{project_id}/qualitative/embeddings/status",
            get(get_embedding_status),
        )
        .route(
            "/projects/{project_id}/qualitative/comments/search",
            post(search_similar_comments),
        )
}
