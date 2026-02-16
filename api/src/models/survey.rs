use chrono::NaiveDateTime;
use pgvector::Vector;
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

    // Embedding fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_embedding: Option<Vector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_generated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyStats {
    pub total_responses: i64,
    pub average_rating: Option<f64>,
    pub first_response_date: Option<NaiveDateTime>,
    pub last_response_date: Option<NaiveDateTime>,
    pub responses_with_comments: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarComment {
    pub response: SurveyResponse,
    pub similarity: f64,
}
