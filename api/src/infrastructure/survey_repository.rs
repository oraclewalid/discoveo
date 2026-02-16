use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::survey::{SurveyResponse, SurveyStats};

#[derive(Clone)]
pub struct SurveyRepository {
    pool: PgPool,
}

impl SurveyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_batch(
        &self,
        responses: &[SurveyResponse],
    ) -> Result<u64, sqlx::Error> {
        if responses.is_empty() {
            return Ok(0);
        }

        let mut tx = self.pool.begin().await?;
        let mut inserted: u64 = 0;

        for response in responses {
            sqlx::query(
                r#"
                INSERT INTO survey_responses (id, project_id, date, country, url, device, browser, os, ratings, comments, raw)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(response.id)
            .bind(response.project_id)
            .bind(response.date)
            .bind(&response.country)
            .bind(&response.url)
            .bind(&response.device)
            .bind(&response.browser)
            .bind(&response.os)
            .bind(response.ratings)
            .bind(&response.comments)
            .bind(&response.raw)
            .execute(&mut *tx)
            .await?;
            inserted += 1;
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn find_by_project(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<SurveyResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SurveyRow>(
            r#"
            SELECT id, project_id, date, country, url, device, browser, os, ratings, comments, raw
            FROM survey_responses
            WHERE project_id = $1
            ORDER BY date DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn delete_by_project(&self, project_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM survey_responses WHERE project_id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_stats(&self, project_id: Uuid) -> Result<SurveyStats, sqlx::Error> {
        let stats = sqlx::query_as::<_, SurveyStatsRow>(
            r#"
            SELECT
                COUNT(*) as total_responses,
                AVG(ratings) as average_rating,
                MIN(date) as first_response_date,
                MAX(date) as last_response_date,
                COUNT(CASE WHEN comments IS NOT NULL AND comments != '' THEN 1 END) as responses_with_comments
            FROM survey_responses
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats.into())
    }
}

#[derive(sqlx::FromRow)]
struct SurveyRow {
    id: Uuid,
    project_id: Uuid,
    date: Option<NaiveDateTime>,
    country: Option<String>,
    url: Option<String>,
    device: Option<String>,
    browser: Option<String>,
    os: Option<String>,
    ratings: Option<f64>,
    comments: Option<String>,
    raw: JsonValue,
}

impl From<SurveyRow> for SurveyResponse {
    fn from(row: SurveyRow) -> Self {
        SurveyResponse {
            id: row.id,
            project_id: row.project_id,
            date: row.date,
            country: row.country,
            url: row.url,
            device: row.device,
            browser: row.browser,
            os: row.os,
            ratings: row.ratings,
            comments: row.comments,
            raw: row.raw,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SurveyStatsRow {
    total_responses: Option<i64>,
    average_rating: Option<f64>,
    first_response_date: Option<NaiveDateTime>,
    last_response_date: Option<NaiveDateTime>,
    responses_with_comments: Option<i64>,
}

impl From<SurveyStatsRow> for SurveyStats {
    fn from(row: SurveyStatsRow) -> Self {
        SurveyStats {
            total_responses: row.total_responses.unwrap_or(0),
            average_rating: row.average_rating,
            first_response_date: row.first_response_date,
            last_response_date: row.last_response_date,
            responses_with_comments: row.responses_with_comments.unwrap_or(0),
        }
    }
}
