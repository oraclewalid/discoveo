use serde::Serialize;
use serde_json::{json, Value};
use tracing::{info, warn};
use uuid::Uuid;

use crate::infrastructure::feedback_repository::FeedbackRepository;
use crate::infrastructure::funnel_repository::{self, FunnelDimension};
use crate::infrastructure::survey_repository::SurveyRepository;
use crate::services::embedding_service::EmbeddingService;

/// Context needed to execute CRO tools
pub struct ToolContext {
    pub project_id: Uuid,
    pub connector_id: Uuid,
    pub duckdb_base_path: String,
    pub survey_repo: SurveyRepository,
    pub feedback_repo: FeedbackRepository,
    pub embedding_service: EmbeddingService,
}

/// A Bedrock tool definition
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Build all tool definitions for the CRO agent
pub fn build_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "get_funnel_overview".to_string(),
            description: "Get the e-commerce funnel analysis for a date range. Returns stages (Home → PLP → PDP → Cart → Checkout → Shipping → Payment → Confirmation) with user counts, drop-off rates, and conversion percentages.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_date": {
                        "type": "string",
                        "description": "Start date in YYYYMMDD format"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in YYYYMMDD format"
                    },
                    "dimension": {
                        "type": "string",
                        "enum": ["all", "browser", "device_category", "country", "operating_system", "screen_resolution"],
                        "description": "Optional dimension to group by. Default: all"
                    }
                },
                "required": ["start_date", "end_date"]
            }),
        },
        ToolDefinition {
            name: "compare_periods".to_string(),
            description: "Compare funnel performance between two date ranges. Returns both funnels side-by-side so you can identify regressions or improvements.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "period_a_start": {
                        "type": "string",
                        "description": "Period A start date in YYYYMMDD format"
                    },
                    "period_a_end": {
                        "type": "string",
                        "description": "Period A end date in YYYYMMDD format"
                    },
                    "period_b_start": {
                        "type": "string",
                        "description": "Period B start date in YYYYMMDD format"
                    },
                    "period_b_end": {
                        "type": "string",
                        "description": "Period B end date in YYYYMMDD format"
                    },
                    "dimension": {
                        "type": "string",
                        "enum": ["all", "browser", "device_category", "country", "operating_system", "screen_resolution"],
                        "description": "Optional dimension to group by. Default: all"
                    }
                },
                "required": ["period_a_start", "period_a_end", "period_b_start", "period_b_end"]
            }),
        },
        ToolDefinition {
            name: "get_page_paths".to_string(),
            description: "Get page-level analytics: pageviews, users, engagement time per page. Useful to identify high-traffic pages with low engagement.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_date": {
                        "type": "string",
                        "description": "Start date in YYYYMMDD format"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in YYYYMMDD format"
                    }
                },
                "required": ["start_date", "end_date"]
            }),
        },
        ToolDefinition {
            name: "get_drop_off_points".to_string(),
            description: "Identify the biggest funnel drop-off points, sorted by severity. Returns stages where the most users are lost.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_date": {
                        "type": "string",
                        "description": "Start date in YYYYMMDD format"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in YYYYMMDD format"
                    }
                },
                "required": ["start_date", "end_date"]
            }),
        },
        ToolDefinition {
            name: "search_survey_comments".to_string(),
            description: "Search user survey comments by semantic similarity. Use this to find what users say about a specific topic (e.g. 'checkout problem', 'slow loading', 'mobile issue').".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results to return. Default: 10"
                    },
                    "min_similarity": {
                        "type": "number",
                        "description": "Minimum cosine similarity threshold (0-1). Default: 0.3"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "get_survey_by_period".to_string(),
            description: "Get user survey comments filtered by date range. Use this to see what users said during a specific period (e.g. when a funnel drop was detected).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_date": {
                        "type": "string",
                        "description": "Start date in YYYY-MM-DD format"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in YYYY-MM-DD format"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results to return. Default: 50"
                    }
                },
                "required": ["start_date", "end_date"]
            }),
        },
        ToolDefinition {
            name: "get_survey_stats".to_string(),
            description: "Get overall survey statistics: total responses, average rating, date range, number of comments.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "get_feedback_themes".to_string(),
            description: "Get the most recent AI-generated feedback analysis: themes, sentiment breakdown, key issues, and recommendations derived from user comments.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

/// Execute a tool by name with the given input, returning a JSON string result
pub async fn execute_tool(
    name: &str,
    input: &Value,
    ctx: &ToolContext,
) -> String {
    info!(tool = name, "Executing CRO tool");

    let result = match name {
        "get_funnel_overview" => exec_funnel_overview(input, ctx),
        "compare_periods" => exec_compare_periods(input, ctx),
        "get_page_paths" => exec_page_paths(input, ctx),
        "get_drop_off_points" => exec_drop_off_points(input, ctx),
        "search_survey_comments" => exec_search_comments(input, ctx).await,
        "get_survey_by_period" => exec_survey_by_period(input, ctx).await,
        "get_survey_stats" => exec_survey_stats(ctx).await,
        "get_feedback_themes" => exec_feedback_themes(ctx).await,
        _ => Err(format!("Unknown tool: {}", name)),
    };

    match result {
        Ok(json_str) => json_str,
        Err(e) => {
            warn!(tool = name, error = %e, "Tool execution failed");
            json!({ "error": e }).to_string()
        }
    }
}

fn parse_dimension(input: &Value) -> FunnelDimension {
    input
        .get("dimension")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "browser" => FunnelDimension::Browser,
            "device_category" => FunnelDimension::DeviceCategory,
            "country" => FunnelDimension::Country,
            "operating_system" => FunnelDimension::OperatingSystem,
            "screen_resolution" => FunnelDimension::ScreenResolution,
            _ => FunnelDimension::All,
        })
        .unwrap_or(FunnelDimension::All)
}

fn required_str<'a>(input: &'a Value, field: &str) -> Result<&'a str, String> {
    input
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing required field: {}", field))
}

fn exec_funnel_overview(input: &Value, ctx: &ToolContext) -> Result<String, String> {
    let start_date = required_str(input, "start_date")?;
    let end_date = required_str(input, "end_date")?;
    let dimension = parse_dimension(input);

    let stages = funnel_repository::query_funnel(
        &ctx.duckdb_base_path,
        ctx.project_id,
        ctx.connector_id,
        dimension,
        start_date,
        end_date,
    )?;

    serde_json::to_string(&stages).map_err(|e| format!("Serialization error: {}", e))
}

fn exec_compare_periods(input: &Value, ctx: &ToolContext) -> Result<String, String> {
    let pa_start = required_str(input, "period_a_start")?;
    let pa_end = required_str(input, "period_a_end")?;
    let pb_start = required_str(input, "period_b_start")?;
    let pb_end = required_str(input, "period_b_end")?;
    let dimension = parse_dimension(input);

    let period_a = funnel_repository::query_funnel(
        &ctx.duckdb_base_path,
        ctx.project_id,
        ctx.connector_id,
        dimension,
        pa_start,
        pa_end,
    )?;

    let period_b = funnel_repository::query_funnel(
        &ctx.duckdb_base_path,
        ctx.project_id,
        ctx.connector_id,
        dimension,
        pb_start,
        pb_end,
    )?;

    let result = json!({
        "period_a": { "start": pa_start, "end": pa_end, "funnel": period_a },
        "period_b": { "start": pb_start, "end": pb_end, "funnel": period_b },
    });

    Ok(result.to_string())
}

fn exec_page_paths(input: &Value, ctx: &ToolContext) -> Result<String, String> {
    let start_date = required_str(input, "start_date")?;
    let end_date = required_str(input, "end_date")?;

    let pages = funnel_repository::query_page_paths(
        &ctx.duckdb_base_path,
        ctx.project_id,
        ctx.connector_id,
        start_date,
        end_date,
    )?;

    serde_json::to_string(&pages).map_err(|e| format!("Serialization error: {}", e))
}

fn exec_drop_off_points(input: &Value, ctx: &ToolContext) -> Result<String, String> {
    let start_date = required_str(input, "start_date")?;
    let end_date = required_str(input, "end_date")?;

    let stages = funnel_repository::query_funnel(
        &ctx.duckdb_base_path,
        ctx.project_id,
        ctx.connector_id,
        FunnelDimension::All,
        start_date,
        end_date,
    )?;

    // Filter to stages with drop-offs and sort by dropoff_pct descending
    let mut drop_offs: Vec<_> = stages
        .into_iter()
        .filter(|s| s.dropoff_pct.is_some() && s.dropoff_pct.unwrap() > 0.0)
        .collect();

    drop_offs.sort_by(|a, b| {
        b.dropoff_pct
            .unwrap_or(0.0)
            .partial_cmp(&a.dropoff_pct.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    serde_json::to_string(&drop_offs).map_err(|e| format!("Serialization error: {}", e))
}

async fn exec_search_comments(input: &Value, ctx: &ToolContext) -> Result<String, String> {
    let query = required_str(input, "query")?;
    let limit = input.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
    let min_similarity = input
        .get("min_similarity")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.3);

    let embedding = ctx
        .embedding_service
        .generate_embedding(query)
        .map_err(|e| format!("Embedding generation failed: {}", e))?
        .ok_or_else(|| "Empty query produced no embedding".to_string())?;

    let results = ctx
        .survey_repo
        .find_similar_comments(ctx.project_id, embedding, limit, min_similarity)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    // Return simplified results (no embedding vectors)
    let simplified: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "comment": r.response.comments,
                "similarity": r.similarity,
                "rating": r.response.ratings,
                "date": r.response.date.map(|d| d.format("%Y-%m-%d").to_string()),
                "country": r.response.country,
                "device": r.response.device,
                "url": r.response.url,
            })
        })
        .collect();

    serde_json::to_string(&simplified).map_err(|e| format!("Serialization error: {}", e))
}

async fn exec_survey_by_period(input: &Value, ctx: &ToolContext) -> Result<String, String> {
    let start_date_str = required_str(input, "start_date")?;
    let end_date_str = required_str(input, "end_date")?;
    let limit = input.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);

    let start_date = chrono::NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "Invalid start date".to_string())?;

    let end_date = chrono::NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(23, 59, 59)
        .ok_or_else(|| "Invalid end date".to_string())?;

    let comments = ctx
        .survey_repo
        .find_comments_by_period(ctx.project_id, start_date, end_date, limit)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let simplified: Vec<Value> = comments
        .iter()
        .map(|c| {
            json!({
                "comment": c.comments,
                "rating": c.ratings,
                "date": c.date.map(|d| d.format("%Y-%m-%d").to_string()),
                "country": c.country,
                "device": c.device,
                "url": c.url,
            })
        })
        .collect();

    serde_json::to_string(&simplified).map_err(|e| format!("Serialization error: {}", e))
}

async fn exec_survey_stats(ctx: &ToolContext) -> Result<String, String> {
    let stats = ctx
        .survey_repo
        .get_stats(ctx.project_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    serde_json::to_string(&stats).map_err(|e| format!("Serialization error: {}", e))
}

async fn exec_feedback_themes(ctx: &ToolContext) -> Result<String, String> {
    let analysis = ctx
        .feedback_repo
        .find_latest(ctx.project_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    match analysis {
        Some(a) => {
            let result = json!({
                "themes": a.analysis.themes,
                "sentiment_breakdown": a.analysis.sentiment_breakdown,
                "key_issues": a.analysis.key_issues,
                "recommendations": a.analysis.recommendations,
                "narrative": a.narrative,
                "created_at": a.created_at.format("%Y-%m-%d %H:%M").to_string(),
            });
            Ok(result.to_string())
        }
        None => Ok(json!({ "message": "No feedback analysis available. Survey comments have not been analyzed yet." }).to_string()),
    }
}
