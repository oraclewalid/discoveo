use sqlx::PgPool;
use uuid::Uuid;

use crate::models::cro_report::{
    CroReport, FunnelAnalysis, QualitativeInsights, CroRecommendation,
};

#[derive(Clone)]
pub struct CroRepository {
    pool: PgPool,
}

impl CroRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, report: &CroReport) -> Result<(), sqlx::Error> {
        let funnel_json = serde_json::to_value(&report.funnel_analysis)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON error: {}", e)))?;
        let qual_json = serde_json::to_value(&report.qualitative_insights)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON error: {}", e)))?;
        let recs_json = serde_json::to_value(&report.recommendations)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON error: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO cro_reports (
                id, project_id, connector_id, created_at,
                executive_summary, funnel_analysis, qualitative_insights, recommendations,
                model_used, input_tokens, output_tokens, tool_calls_count, duration_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            report.id,
            report.project_id,
            report.connector_id,
            report.created_at,
            report.executive_summary,
            funnel_json,
            qual_json,
            recs_json,
            report.model_used,
            report.input_tokens,
            report.output_tokens,
            report.tool_calls_count,
            report.duration_ms,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<CroReport>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, project_id, connector_id, created_at,
                   executive_summary, funnel_analysis, qualitative_insights, recommendations,
                   model_used, input_tokens, output_tokens, tool_calls_count, duration_ms
            FROM cro_reports
            WHERE project_id = $1
            ORDER BY created_at DESC
            "#,
            project_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let funnel_analysis: FunnelAnalysis =
                    serde_json::from_value(r.funnel_analysis).ok()?;
                let qualitative_insights: QualitativeInsights =
                    serde_json::from_value(r.qualitative_insights).ok()?;
                let recommendations: Vec<CroRecommendation> =
                    serde_json::from_value(r.recommendations).ok()?;

                Some(CroReport {
                    id: r.id,
                    project_id: r.project_id,
                    connector_id: r.connector_id,
                    created_at: r.created_at,
                    executive_summary: r.executive_summary,
                    funnel_analysis,
                    qualitative_insights,
                    recommendations,
                    model_used: r.model_used,
                    input_tokens: r.input_tokens,
                    output_tokens: r.output_tokens,
                    tool_calls_count: r.tool_calls_count,
                    duration_ms: r.duration_ms,
                })
            })
            .collect())
    }

    pub async fn find_by_id(
        &self,
        report_id: Uuid,
    ) -> Result<Option<CroReport>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, project_id, connector_id, created_at,
                   executive_summary, funnel_analysis, qualitative_insights, recommendations,
                   model_used, input_tokens, output_tokens, tool_calls_count, duration_ms
            FROM cro_reports
            WHERE id = $1
            "#,
            report_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| {
            let funnel_analysis: FunnelAnalysis =
                serde_json::from_value(r.funnel_analysis).ok()?;
            let qualitative_insights: QualitativeInsights =
                serde_json::from_value(r.qualitative_insights).ok()?;
            let recommendations: Vec<CroRecommendation> =
                serde_json::from_value(r.recommendations).ok()?;

            Some(CroReport {
                id: r.id,
                project_id: r.project_id,
                connector_id: r.connector_id,
                created_at: r.created_at,
                executive_summary: r.executive_summary,
                funnel_analysis,
                qualitative_insights,
                recommendations,
                model_used: r.model_used,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                tool_calls_count: r.tool_calls_count,
                duration_ms: r.duration_ms,
            })
        }))
    }
}
