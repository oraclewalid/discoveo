use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub date: Option<NaiveDateTime>,
    pub country: Option<String>,
    pub url: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub ratings: Option<f64>,
    pub comments: Option<String>,
    pub raw: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyStats {
    pub total_responses: i64,
    pub average_rating: Option<f64>,
    pub first_response_date: Option<NaiveDateTime>,
    pub last_response_date: Option<NaiveDateTime>,
    pub responses_with_comments: i64,
}
