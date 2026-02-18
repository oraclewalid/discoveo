use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAnalysis {
    pub id: Uuid,
    pub project_id: Uuid,
    pub created_at: NaiveDateTime,
    pub analysis: StructuredAnalysis,
    pub narrative: String,
    pub model_used: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub duration_ms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredAnalysis {
    pub themes: Vec<Theme>,
    pub sentiment_breakdown: SentimentBreakdown,
    pub key_issues: Vec<KeyIssue>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub sentiment: String,
    pub frequency: String,
    pub sample_quotes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentBreakdown {
    pub positive_pct: f64,
    pub negative_pct: f64,
    pub neutral_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyIssue {
    pub title: String,
    pub severity: String,
    pub description: String,
    pub affected_users_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub title: String,
    pub priority: String,
    pub description: String,
    pub expected_impact: String,
}
