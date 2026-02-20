use sqlx::PgPool;
use uuid::Uuid;

use crate::models::feedback::{FeedbackAnalysis, StructuredAnalysis};

#[derive(Clone)]
pub struct FeedbackRepository {
    pool: PgPool,
}

impl FeedbackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find a cached analysis that is less than 24h old and matches the current response count
    pub async fn find_cached(
        &self,
        project_id: Uuid,
        response_count: i32,
    ) -> Result<Option<FeedbackAnalysis>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, project_id, created_at, response_count, analysis, narrative,
                   model_used, input_tokens, output_tokens, duration_ms
            FROM feedback_analyses
            WHERE project_id = $1
              AND response_count = $2
              AND created_at > NOW() - INTERVAL '24 hours'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            project_id,
            response_count,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let analysis: StructuredAnalysis =
                serde_json::from_value(r.analysis).unwrap_or_else(|_| StructuredAnalysis {
                    themes: vec![],
                    sentiment_breakdown: crate::models::feedback::SentimentBreakdown {
                        positive_pct: 0.0,
                        negative_pct: 0.0,
                        neutral_pct: 0.0,
                    },
                    key_issues: vec![],
                    recommendations: vec![],
                });

            FeedbackAnalysis {
                id: r.id,
                project_id: r.project_id,
                created_at: r.created_at,
                analysis,
                narrative: r.narrative,
                model_used: r.model_used,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                duration_ms: r.duration_ms,
            }
        }))
    }

    /// Find the most recent feedback analysis for a project (no TTL filter)
    pub async fn find_latest(
        &self,
        project_id: Uuid,
    ) -> Result<Option<FeedbackAnalysis>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, project_id, created_at, response_count, analysis, narrative,
                   model_used, input_tokens, output_tokens, duration_ms
            FROM feedback_analyses
            WHERE project_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            project_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let analysis: StructuredAnalysis =
                serde_json::from_value(r.analysis).unwrap_or_else(|_| StructuredAnalysis {
                    themes: vec![],
                    sentiment_breakdown: crate::models::feedback::SentimentBreakdown {
                        positive_pct: 0.0,
                        negative_pct: 0.0,
                        neutral_pct: 0.0,
                    },
                    key_issues: vec![],
                    recommendations: vec![],
                });

            FeedbackAnalysis {
                id: r.id,
                project_id: r.project_id,
                created_at: r.created_at,
                analysis,
                narrative: r.narrative,
                model_used: r.model_used,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                duration_ms: r.duration_ms,
            }
        }))
    }

    pub async fn insert(
        &self,
        analysis: &FeedbackAnalysis,
        response_count: i32,
    ) -> Result<(), sqlx::Error> {
        let analysis_json = serde_json::to_value(&analysis.analysis)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization error: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO feedback_analyses (id, project_id, created_at, response_count, analysis, narrative, model_used, input_tokens, output_tokens, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            analysis.id,
            analysis.project_id,
            analysis.created_at,
            response_count,
            analysis_json,
            analysis.narrative,
            analysis.model_used,
            analysis.input_tokens,
            analysis.output_tokens,
            analysis.duration_ms,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
