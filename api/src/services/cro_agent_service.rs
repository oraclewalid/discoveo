use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::cro_report::CroReport;
use crate::services::cro_tools::{self, ToolContext, ToolDefinition};

const BEDROCK_REGION: &str = "us-east-1";
const MAX_AGENT_TURNS: usize = 25;
const AGENT_MAX_TOKENS: u32 = 8192;

#[derive(Clone)]
pub struct CroAgentService {
    bearer_token: Option<String>,
    model_id: String,
    http_client: reqwest::Client,
}

impl CroAgentService {
    pub fn new(bearer_token: Option<String>, model_id: Option<String>) -> Self {
        Self {
            bearer_token,
            model_id: model_id.unwrap_or_else(|| "anthropic.claude-sonnet-4-20250514-v1:0".to_string()),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn generate_report(
        &self,
        project_id: Uuid,
        connector_id: Uuid,
        ctx: ToolContext,
    ) -> Result<CroReport, String> {
        let token = self
            .bearer_token
            .as_ref()
            .ok_or_else(|| "AWS_BEARER_TOKEN_BEDROCK is not configured".to_string())?;

        let start = std::time::Instant::now();
        let system_prompt = build_system_prompt();
        let tools = cro_tools::build_tool_definitions();
        let bedrock_tools = build_bedrock_tools(&tools);

        let initial_message = build_initial_message();
        let mut messages: Vec<Message> = vec![Message {
            role: "user".to_string(),
            content: vec![ContentBlock::Text {
                text: initial_message,
            }],
        }];

        let mut total_input_tokens: u32 = 0;
        let mut total_output_tokens: u32 = 0;
        let mut tool_calls_count: i32 = 0;
        let mut final_text = String::new();

        for turn in 0..MAX_AGENT_TURNS {
            info!(turn, "CRO agent turn");

            let response = self
                .call_bedrock(token, &system_prompt, &messages, &bedrock_tools)
                .await?;

            total_input_tokens += response.usage.input_tokens;
            total_output_tokens += response.usage.output_tokens;

            // Collect text blocks and tool_use blocks from response
            let mut assistant_content: Vec<ContentBlock> = Vec::new();
            let mut tool_uses: Vec<(String, String, Value)> = Vec::new(); // (id, name, input)

            for block in &response.content {
                match block {
                    ResponseBlock::Text { text, .. } => {
                        tracing::debug!(text_len = text.len(), "LLM text block received");
                        final_text = text.clone();
                        assistant_content.push(ContentBlock::Text { text: text.clone() });
                    }
                    ResponseBlock::ToolUse { id, name, input, .. } => {
                        tool_calls_count += 1;
                        assistant_content.push(ContentBlock::ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                        tool_uses.push((id.clone(), name.clone(), input.clone()));
                    }
                }
            }

            // Append assistant message
            messages.push(Message {
                role: "assistant".to_string(),
                content: assistant_content,
            });

            // If stop_reason is end_turn or no tool_uses, we're done
            let stop_reason = response.stop_reason.as_deref().unwrap_or("end_turn");
            if stop_reason == "end_turn" || tool_uses.is_empty() {
                info!(turn, stop_reason, "CRO agent finished");
                break;
            }

            // Execute tools and build tool_result blocks
            let mut tool_results: Vec<ContentBlock> = Vec::new();
            for (tool_id, tool_name, tool_input) in &tool_uses {
                let result = cro_tools::execute_tool(tool_name, tool_input, &ctx).await;
                info!(tool = %tool_name, result_len = result.len(), "Tool result");
                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: tool_id.clone(),
                    content: result,
                });
            }

            // Append user message with tool results
            messages.push(Message {
                role: "user".to_string(),
                content: tool_results,
            });
        }

        let duration_ms = start.elapsed().as_millis() as i32;

        info!(
            tool_calls_count,
            total_input_tokens,
            total_output_tokens,
            duration_ms,
            "CRO report generation complete"
        );

        parse_report(
            &final_text,
            project_id,
            connector_id,
            &self.model_id,
            total_input_tokens as i32,
            total_output_tokens as i32,
            tool_calls_count,
            duration_ms,
        )
    }

    async fn call_bedrock(
        &self,
        token: &str,
        system: &str,
        messages: &[Message],
        tools: &[Value],
    ) -> Result<BedrockResponse, String> {
        let url = format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            BEDROCK_REGION,
            urlencoding::encode(&self.model_id),
        );

        let request = BedrockRequest {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens: AGENT_MAX_TOKENS,
            system: system.to_string(),
            messages: messages.to_vec(),
            tools: tools.to_vec(),
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

        response
            .json::<BedrockResponse>()
            .await
            .map_err(|e| format!("Failed to parse Bedrock response: {}", e))
    }
}

fn build_system_prompt() -> String {
    r#"You are an expert CRO (Conversion Rate Optimization) analyst. You have access to tools that let you query GA4 analytics data and user survey feedback for a website.

Your task: Produce a comprehensive CRO audit by investigating the data thoroughly.

## Investigation strategy

Follow these steps IN ORDER. Do NOT skip steps.

### Step 1: Overall funnel (dimension="all")
- Get the funnel overview for the last 90 days with dimension "all"
- Identify which stages have the biggest drop-off rates

### Step 2: Break down by device, country, browser
- Run the funnel with dimension="device_category" to compare mobile vs desktop vs tablet
- Run the funnel with dimension="country" to find geographic differences
- Run the funnel with dimension="browser" to detect browser-specific issues
- Look for: Is mobile drop-off much worse than desktop? Is one country underperforming?

### Step 3: Page-level analysis
- Get page paths data to see time spent on each page (avg_time_per_pageview_sec, avg_time_per_user_sec)
- Look for anomalies: pages where users spend TOO MUCH time (friction/confusion, e.g. checkout, cart) or TOO LITTLE time (not engaging)
- High time on checkout/cart/payment pages = users struggling
- Low time on product pages = users not finding what they need

### Step 4: Trend comparison
- Compare the last 2 weeks vs the previous 2 weeks
- Detect regressions: did any metric get significantly worse?

### Step 5: Qualitative cross-reference
- Get survey statistics and feedback themes
- For each major drop-off found in steps 1-3, search survey comments for related issues
- Example: if mobile checkout drops 60%, search for "mobile checkout", "payment mobile", "phone"
- Get survey comments from the same period as detected regressions

### Step 6: Synthesize
- Combine quantitative + qualitative findings into the final report

## Key metrics to report on
- **active_users**: How many unique users at each funnel stage
- **sessions**: Total sessions (indicates engagement depth)
- **screen_page_views**: Page view volume per page
- **bounce_rate**: Percentage of single-page sessions
- **average_session_duration**: How long users stay
- **avg_time_per_pageview_sec**: Time per page view (from page paths)
- **Device breakdown**: Mobile vs Desktop conversion rates
- **Country breakdown**: Top countries by conversion and drop-off
- **Browser breakdown**: Browser-specific issues (Safari vs Chrome vs Firefox)

## Output format

When you have gathered enough data, output ONLY a JSON object (no text before or after, no markdown fences):
{
  "executive_summary": "2-3 sentence overview of the most critical findings, with key numbers",
  "funnel_analysis": {
    "overview": "Narrative description of funnel performance including device/country/browser breakdowns",
    "critical_drop_offs": [
      {
        "stage": "stage name (e.g. PDP → Cart)",
        "drop_rate": 45.2,
        "severity": "critical|major|minor",
        "correlated_feedback": ["verbatim user quote 1", "verbatim user quote 2"]
      }
    ],
    "period_comparison": {
      "period_a": "YYYYMMDD-YYYYMMDD",
      "period_b": "YYYYMMDD-YYYYMMDD",
      "changes": [
        {
          "metric": "metric name (e.g. mobile_checkout_dropoff, cart_active_users)",
          "before": 100.0,
          "after": 85.0,
          "change_pct": -15.0,
          "interpretation": "what this change means for conversions"
        }
      ]
    }
  },
  "qualitative_insights": {
    "overview": "Summary of user feedback themes correlated with quantitative data",
    "themes_with_data": [
      {
        "theme": "theme name",
        "sentiment": "positive|negative|mixed|neutral",
        "supporting_quotes": ["verbatim quote 1", "verbatim quote 2"],
        "related_metrics": ["mobile checkout drop-off: 62%", "avg time on /checkout: 180s (3x homepage)"]
      }
    ]
  },
  "recommendations": [
    {
      "title": "Short actionable title",
      "priority": "high|medium|low",
      "category": "UX|Performance|Content|Technical",
      "description": "What to do and why, referencing specific numbers",
      "supporting_evidence": ["quant: mobile cart drop-off 62%", "qual: 'checkout freezes on my phone'"],
      "expected_impact": "Expected improvement with estimated impact"
    }
  ]
}

## Rules
- ALWAYS break down the funnel by device_category — mobile vs desktop is the most important CRO dimension
- ALWAYS check page-level time metrics — high time on transactional pages (cart, checkout, payment) signals UX friction
- Back every recommendation with BOTH quantitative data AND user feedback when available
- Be specific: "mobile cart→checkout drop-off is 62% vs 35% on desktop" not "checkout has issues"
- Include actual user quotes in correlated_feedback
- If no survey data is available, produce the report using GA4 data only
- period_comparison can be null if comparison data is not meaningful
- Sort recommendations by priority (high first)
- Output ONLY the JSON object, nothing else"#
        .to_string()
}

fn build_initial_message() -> String {
    "Analyze the website's conversion performance and produce a CRO audit. Start with the overall funnel for the last 90 days, then break down by device_category, country, and browser. Check page-level engagement times. Cross-reference everything with user survey feedback. Use date range 20250101 to 20260218 for the full overview.".to_string()
}

fn build_bedrock_tools(tools: &[ToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema,
            })
        })
        .collect()
}

/// Extract a JSON object from text that may contain narrative around it
fn extract_json(raw: &str) -> Option<&str> {
    // Try to find the outermost { ... } block
    let start = raw.find('{')?;
    let mut depth = 0;
    let mut end = None;
    for (i, ch) in raw[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(start + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    end.map(|e| &raw[start..e])
}

fn parse_report(
    raw: &str,
    project_id: Uuid,
    connector_id: Uuid,
    model_used: &str,
    input_tokens: i32,
    output_tokens: i32,
    tool_calls_count: i32,
    duration_ms: i32,
) -> Result<CroReport, String> {
    info!(raw_len = raw.len(), "Parsing CRO report from LLM response");
    tracing::debug!(raw_response = %raw, "Raw LLM response");

    // First try: clean markdown fences and parse directly
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let value: Value = serde_json::from_str(cleaned)
        .or_else(|_| {
            // Second try: extract JSON object from mixed text
            extract_json(raw)
                .ok_or_else(|| "No JSON object found in response".to_string())
                .and_then(|json_str| {
                    serde_json::from_str(json_str)
                        .map_err(|e| format!("Failed to parse extracted JSON: {}", e))
                })
        })
        .map_err(|e| {
            warn!(error = %e, raw_len = raw.len(), raw_start = %&raw[..raw.len().min(500)], "Failed to parse CRO report JSON");
            format!("Failed to parse CRO report from LLM response: {}", e)
        })?;

    let executive_summary = value
        .get("executive_summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let funnel_analysis = serde_json::from_value(
        value.get("funnel_analysis").cloned().unwrap_or_default(),
    )
    .map_err(|e| format!("Failed to parse funnel_analysis: {}", e))?;

    let qualitative_insights = serde_json::from_value(
        value.get("qualitative_insights").cloned().unwrap_or_default(),
    )
    .map_err(|e| format!("Failed to parse qualitative_insights: {}", e))?;

    let recommendations = serde_json::from_value(
        value.get("recommendations").cloned().unwrap_or_default(),
    )
    .map_err(|e| format!("Failed to parse recommendations: {}", e))?;

    Ok(CroReport {
        id: Uuid::now_v7(),
        project_id,
        connector_id,
        created_at: Utc::now().naive_utc(),
        executive_summary,
        funnel_analysis,
        qualitative_insights,
        recommendations,
        model_used: model_used.to_string(),
        input_tokens,
        output_tokens,
        tool_calls_count,
        duration_ms,
    })
}

// --- Bedrock API types for tool_use ---

#[derive(Debug, Clone, Serialize)]
struct BedrockRequest {
    anthropic_version: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Deserialize)]
struct BedrockResponse {
    content: Vec<ResponseBlock>,
    usage: Usage,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ResponseBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}
