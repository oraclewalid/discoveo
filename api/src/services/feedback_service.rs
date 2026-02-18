use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::infrastructure::feedback_repository::FeedbackRepository;
use crate::infrastructure::survey_repository::SurveyRepository;
use crate::models::feedback::{FeedbackAnalysis, StructuredAnalysis};
use crate::models::survey::CommentForAnalysis;

const BEDROCK_REGION: &str = "us-east-1";
const DEFAULT_MODEL_ID: &str = "anthropic.claude-sonnet-4-20250514-v1:0";
const CLAUDE_MAX_TOKENS: u32 = 4096;

#[derive(Clone)]
pub struct FeedbackService {
    bearer_token: Option<String>,
    model_id: String,
    http_client: reqwest::Client,
}

impl FeedbackService {
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn has_bearer_token(&self) -> bool {
        self.bearer_token.is_some()
    }
}

pub struct AnalysisResult {
    pub analysis: StructuredAnalysis,
    pub narrative: String,
    pub model_used: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

impl FeedbackService {
    pub fn new(bearer_token: Option<String>, model_id: Option<String>) -> Self {
        Self {
            bearer_token,
            model_id: model_id.unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
            http_client: reqwest::Client::new(),
        }
    }

    /// Orchestrates the full feedback analysis: cache check, data fetch, LLM call, and caching.
    pub async fn generate_feedback(
        &self,
        project_id: Uuid,
        force: bool,
        survey_repo: &SurveyRepository,
        feedback_repo: &FeedbackRepository,
    ) -> Result<FeedbackAnalysis, String> {
        let comment_count = survey_repo
            .count_comments(project_id)
            .await
            .map_err(|e| format!("Database error: {}", e))? as i32;

        if comment_count < 5 {
            return Err("Not enough comments for analysis (minimum 5 required)".to_string());
        }

        // Check cache unless force=true
        if !force {
            if let Ok(Some(cached)) = feedback_repo.find_cached(project_id, comment_count).await {
                info!("Returning cached feedback analysis");
                return Ok(cached);
            }
        }

        // Fetch all comments
        let comments = survey_repo
            .find_all_comments(project_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Call Claude API
        let start = std::time::Instant::now();
        let result = self.call_llm(&comments).await?;
        let duration_ms = start.elapsed().as_millis() as i32;

        // Build and cache the analysis
        let analysis = FeedbackAnalysis {
            id: Uuid::now_v7(),
            project_id,
            created_at: Utc::now().naive_utc(),
            analysis: result.analysis,
            narrative: result.narrative,
            model_used: result.model_used,
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
            duration_ms: Some(duration_ms),
        };

        if let Err(e) = feedback_repo.insert(&analysis, comment_count).await {
            warn!(error = %e, "Failed to cache feedback analysis");
        }

        info!(duration_ms = duration_ms, "Feedback analysis complete and cached");

        Ok(analysis)
    }

    async fn call_llm(
        &self,
        comments: &[CommentForAnalysis],
    ) -> Result<AnalysisResult, String> {
        let token = self
            .bearer_token
            .as_ref()
            .ok_or_else(|| "AWS_BEARER_TOKEN_BEDROCK is not configured".to_string())?;

        let system_prompt = build_system_prompt();
        let user_message = build_user_message(comments);

        info!(
            comment_count = comments.len(),
            user_message_len = user_message.len(),
            "Calling Claude via Bedrock for feedback analysis"
        );

        let url = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            BEDROCK_REGION,
            urlencoding::encode(&self.model_id),
        );

        let request = BedrockRequest {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens: CLAUDE_MAX_TOKENS,
            system: system_prompt,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: user_message,
            }],
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to call Bedrock API: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unable to read response body".to_string());
            return Err(format!("Bedrock API returned {}: {}", status, body));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Bedrock response: {}", e))?;

        let raw_text = claude_response
            .content
            .first()
            .map(|block| block.text.as_str())
            .unwrap_or("");

        let (analysis, narrative) = parse_response(raw_text)?;

        info!(
            input_tokens = claude_response.usage.input_tokens,
            output_tokens = claude_response.usage.output_tokens,
            "Bedrock Claude analysis complete"
        );

        Ok(AnalysisResult {
            analysis,
            narrative,
            model_used: self.model_id.clone(),
            input_tokens: Some(claude_response.usage.input_tokens as i32),
            output_tokens: Some(claude_response.usage.output_tokens as i32),
        })
    }
}

fn build_system_prompt() -> String {
    r#"You are an expert UX researcher analyzing website visitor survey feedback.
Analyze all the comments provided and return a JSON object with this exact structure:
{
  "themes": [
    {
      "name": "short theme name",
      "description": "1-2 sentence description of this theme",
      "sentiment": "positive|negative|mixed|neutral",
      "frequency": "high|medium|low",
      "sample_quotes": ["1-2 verbatim quotes from the comments"]
    }
  ],
  "sentiment_breakdown": {
    "positive_pct": 0,
    "negative_pct": 0,
    "neutral_pct": 0
  },
  "key_issues": [
    {
      "title": "issue title",
      "severity": "critical|major|minor",
      "description": "description of the issue",
      "affected_users_pct": 0
    }
  ],
  "recommendations": [
    {
      "title": "recommendation title",
      "priority": "high|medium|low",
      "description": "what to do",
      "expected_impact": "expected result"
    }
  ],
  "narrative_summary": "A comprehensive free-text summary of all findings, written as a report paragraph."
}

Important rules:
- Respond with ONLY the JSON object, no markdown code fences, no additional text
- Percentages should sum to 100 in sentiment_breakdown
- Base affected_users_pct on the proportion of comments mentioning that issue
- Include 3-8 themes depending on diversity of feedback
- The narrative_summary should be 3-5 sentences synthesizing the key takeaways"#
        .to_string()
}

fn build_user_message(comments: &[CommentForAnalysis]) -> String {
    let mut msg = format!(
        "Survey feedback analysis — {} total comments.\n\nComments:\n",
        comments.len()
    );

    for (i, comment) in comments.iter().enumerate() {
        let rating_str = comment
            .ratings
            .map(|r| format!("{:.1}", r))
            .unwrap_or_else(|| "N/A".to_string());
        let country = comment.country.as_deref().unwrap_or("N/A");
        let device = comment.device.as_deref().unwrap_or("N/A");
        let date = comment
            .date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let url = comment.url.as_deref().unwrap_or("N/A");

        msg.push_str(&format!(
            "{}. \"{}\" [Rating: {}, Country: {}, Device: {}, Date: {}, URL: {}]\n",
            i + 1,
            comment.comments,
            rating_str,
            country,
            device,
            date,
            url,
        ));
    }

    msg.push_str("\nAnalyze all feedback and provide the structured JSON analysis.");
    msg
}

/// Parse Claude's response, extracting StructuredAnalysis and the narrative_summary
fn parse_response(raw: &str) -> Result<(StructuredAnalysis, String), String> {
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let value: serde_json::Value =
        serde_json::from_str(cleaned).map_err(|e| format!("Failed to parse LLM JSON: {}", e))?;

    let narrative = value
        .get("narrative_summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let analysis: StructuredAnalysis = serde_json::from_value(value.clone()).map_err(|e| {
        warn!(error = %e, raw = %cleaned, "Failed to parse structured analysis");
        format!("Failed to parse structured analysis: {}", e)
    })?;

    Ok((analysis, narrative))
}

// Bedrock API types

#[derive(Serialize)]
struct BedrockRequest {
    anthropic_version: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}
