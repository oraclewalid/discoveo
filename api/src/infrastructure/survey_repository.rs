use chrono::NaiveDateTime;
use pgvector::Vector;
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::survey::{CommentForAnalysis, SimilarComment, SurveyResponse, SurveyStats};

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

    /// Find survey responses with pending embeddings
    pub async fn find_pending_embeddings(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<SurveyResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SurveyRow>(
            r#"
            SELECT id, project_id, date, country, url, device, browser, os,
                   ratings, comments, raw, comment_embedding, embedding_status,
                   embedding_generated_at
            FROM survey_responses
            WHERE project_id = $1
              AND embedding_status = 'pending'
              AND comments IS NOT NULL
              AND comments != ''
            ORDER BY date DESC
            LIMIT 1000
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Update embedding for a survey response
    pub async fn update_embedding(
        &self,
        response_id: Uuid,
        embedding: Vec<f32>,
    ) -> Result<(), sqlx::Error> {
        let vector = Vector::from(embedding);

        sqlx::query(
            r#"
            UPDATE survey_responses
            SET comment_embedding = $1,
                embedding_status = 'completed',
                embedding_generated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(vector)
        .bind(response_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update embedding status (for failed/skipped cases)
    pub async fn update_embedding_status(
        &self,
        response_id: Uuid,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE survey_responses
            SET embedding_status = $1
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(response_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Find comments filtered by date range
    pub async fn find_comments_by_period(
        &self,
        project_id: Uuid,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        limit: i64,
    ) -> Result<Vec<CommentForAnalysis>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CommentRow>(
            r#"
            SELECT comments, ratings, date, country, device, url
            FROM survey_responses
            WHERE project_id = $1
              AND comments IS NOT NULL
              AND comments != ''
              AND date >= $2
              AND date <= $3
            ORDER BY date DESC NULLS LAST
            LIMIT $4
            "#,
        )
        .bind(project_id)
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Find all comments with metadata for LLM analysis (max 500, most recent first)
    pub async fn find_all_comments(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<CommentForAnalysis>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CommentRow>(
            r#"
            SELECT comments, ratings, date, country, device, url
            FROM survey_responses
            WHERE project_id = $1
              AND comments IS NOT NULL
              AND comments != ''
            ORDER BY date DESC NULLS LAST
            LIMIT 500
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Count comments with non-empty text for a project
    pub async fn count_comments(&self, project_id: Uuid) -> Result<i64, sqlx::Error> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM survey_responses
            WHERE project_id = $1
              AND comments IS NOT NULL
              AND comments != ''
            "#,
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Find similar comments using cosine similarity
    pub async fn find_similar_comments(
        &self,
        project_id: Uuid,
        query_embedding: Vec<f32>,
        limit: i64,
        min_similarity: f64,
    ) -> Result<Vec<SimilarComment>, sqlx::Error> {
        let vector = Vector::from(query_embedding);

        let rows = sqlx::query(
            r#"
            SELECT
                id, project_id, date, country, url, device, browser, os,
                ratings, comments, raw, comment_embedding, embedding_status,
                embedding_generated_at,
                1 - (comment_embedding <=> $1) as similarity
            FROM survey_responses
            WHERE project_id = $2
              AND comment_embedding IS NOT NULL
              AND 1 - (comment_embedding <=> $1) >= $3
            ORDER BY comment_embedding <=> $1
            LIMIT $4
            "#,
        )
        .bind(&vector)
        .bind(project_id)
        .bind(min_similarity)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let similarity: f64 = row.try_get("similarity").unwrap_or(0.0);
            let response = SurveyResponse {
                id: row.try_get("id").unwrap(),
                project_id: row.try_get("project_id").unwrap(),
                date: row.try_get("date").ok(),
                country: row.try_get("country").ok(),
                url: row.try_get("url").ok(),
                device: row.try_get("device").ok(),
                browser: row.try_get("browser").ok(),
                os: row.try_get("os").ok(),
                ratings: row.try_get("ratings").ok(),
                comments: row.try_get("comments").ok(),
                raw: row.try_get("raw").unwrap(),
                comment_embedding: row.try_get("comment_embedding").ok(),
                embedding_status: row.try_get("embedding_status").ok(),
                embedding_generated_at: row.try_get("embedding_generated_at").ok(),
            };
            results.push(SimilarComment {
                response,
                similarity,
            });
        }

        Ok(results)
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
    comment_embedding: Option<Vector>,
    embedding_status: Option<String>,
    embedding_generated_at: Option<NaiveDateTime>,
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
            comment_embedding: row.comment_embedding,
            embedding_status: row.embedding_status,
            embedding_generated_at: row.embedding_generated_at,
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

#[derive(sqlx::FromRow)]
struct CommentRow {
    comments: Option<String>,
    ratings: Option<f64>,
    date: Option<NaiveDateTime>,
    country: Option<String>,
    device: Option<String>,
    url: Option<String>,
}

impl From<CommentRow> for CommentForAnalysis {
    fn from(row: CommentRow) -> Self {
        CommentForAnalysis {
            comments: row.comments.unwrap_or_default(),
            ratings: row.ratings,
            date: row.date,
            country: row.country,
            device: row.device,
            url: row.url,
        }
    }
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
