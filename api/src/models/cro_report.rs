use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CroReport {
    pub id: Uuid,
    pub project_id: Uuid,
    pub connector_id: Uuid,
    pub created_at: NaiveDateTime,
    pub executive_summary: String,
    pub funnel_analysis: FunnelAnalysis,
    pub qualitative_insights: QualitativeInsights,
    pub recommendations: Vec<CroRecommendation>,
    pub model_used: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub tool_calls_count: i32,
    pub duration_ms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelAnalysis {
    pub overview: String,
    pub critical_drop_offs: Vec<DropOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_comparison: Option<PeriodComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropOff {
    pub stage: String,
    pub drop_rate: f64,
    pub severity: String,
    pub correlated_feedback: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub period_a: String,
    pub period_b: String,
    pub changes: Vec<MetricChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricChange {
    pub metric: String,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub change_pct: Option<f64>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitativeInsights {
    pub overview: String,
    pub themes_with_data: Vec<ThemeWithData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeWithData {
    pub theme: String,
    pub sentiment: String,
    pub supporting_quotes: Vec<String>,
    pub related_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CroRecommendation {
    pub title: String,
    pub priority: String,
    pub category: String,
    pub description: String,
    pub supporting_evidence: Vec<String>,
    pub expected_impact: String,
}
