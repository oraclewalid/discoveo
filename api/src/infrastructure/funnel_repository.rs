use duckdb::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunnelDimension {
    Browser,
    DeviceCategory,
    Country,
    OperatingSystem,
    ScreenResolution,
    All,
}

impl FunnelDimension {
    /// Returns a SQL expression for the dimension column.
    /// Safe from injection since all values are hardcoded.
    fn to_sql_expr(&self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::DeviceCategory => "device_category",
            Self::Country => "country",
            Self::OperatingSystem => "operating_system",
            Self::ScreenResolution => "screen_resolution",
            Self::All => "'ALL'",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FunnelStage {
    pub stage_order: i32,
    pub dimension: String,
    pub funnel_stage: String,
    pub total_users: i64,
    pub total_interactions: i64,
    pub prev_stage_users: Option<i64>,
    pub users_dropped: Option<i64>,
    pub dropoff_pct: Option<f64>,
    pub conversion_from_start_pct: Option<f64>,
    pub stage_conversion_pct: Option<f64>,
    pub ranking: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScrollDepthData {
    pub dimension: String,
    pub scroll_depth: String,
    pub events: i64,
    pub users: i64,
    pub prev_stage_users: Option<i64>,
    pub drop_off_pct: Option<f64>,
    pub users_lost: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PagePathAnalytics {
    pub page_path: String,
    pub total_pageviews: i64,
    pub total_users: i64,
    pub total_engagement_seconds: f64,
    pub avg_time_per_pageview_sec: Option<f64>,
    pub avg_time_per_user_sec: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventNameDebug {
    pub event_name: String,
    pub total_events: i64,
    pub total_users: i64,
}

fn db_path(base_path: &str, project_id: Uuid, connector_id: Uuid) -> PathBuf {
    PathBuf::from(base_path)
        .join(project_id.to_string())
        .join(connector_id.to_string())
        .join("ga4.duckdb")
}

pub fn query_funnel(
    base_path: &str,
    project_id: Uuid,
    connector_id: Uuid,
    dimension: FunnelDimension,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<FunnelStage>, String> {
    let path = db_path(base_path, project_id, connector_id);
    if !path.exists() {
        return Err("No data available. Pull GA4 data first.".to_string());
    }

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open DuckDB: {}", e))?;

    let dim_expr = dimension.to_sql_expr();

    let sql = format!(
        r#"
        WITH event_funnel AS (
            SELECT
                {dim_expr} AS dimension,
                CASE event_name
                    WHEN 'session_start' THEN 'Home'
                    WHEN 'view_item_list' THEN 'PLP'
                    WHEN 'view_item' THEN 'PDP'
                    WHEN 'view_cart' THEN 'Cart'
                    WHEN 'begin_checkout' THEN 'Checkout'
                    WHEN 'add_shipping_info' THEN 'Shipping'
                    WHEN 'add_payment_info' THEN 'Payment'
                    WHEN 'purchase' THEN 'Confirmation'
                    ELSE NULL
                END AS funnel_stage,
                active_users AS users,
                sessions AS interactions
            FROM ga4_events
            WHERE date >= ? AND date <= ?
        ),
        stage_aggregated AS (
            SELECT
                funnel_stage,
                dimension,
                CAST(SUM(users) AS BIGINT) AS total_users,
                CAST(SUM(interactions) AS BIGINT) AS total_interactions,
                CASE funnel_stage
                    WHEN 'Home' THEN 1
                    WHEN 'PLP' THEN 2
                    WHEN 'PDP' THEN 3
                    WHEN 'Cart' THEN 4
                    WHEN 'Checkout' THEN 5
                    WHEN 'Shipping' THEN 6
                    WHEN 'Payment' THEN 7
                    WHEN 'Confirmation' THEN 8
                END AS stage_order
            FROM event_funnel
            WHERE funnel_stage IS NOT NULL
            GROUP BY funnel_stage, dimension
        )
        SELECT * FROM (
            SELECT
                CAST(stage_order AS INTEGER) AS stage_order,
                dimension,
                funnel_stage,
                total_users,
                total_interactions,
                CAST(LAG(total_users) OVER w AS BIGINT) AS prev_stage_users,
                CAST(LAG(total_users) OVER w - total_users AS BIGINT) AS users_dropped,
                ROUND(
                    100.0 * (LAG(total_users) OVER w - total_users)
                    / NULLIF(LAG(total_users) OVER w, 0), 2
                ) AS dropoff_pct,
                ROUND(
                    100.0 * total_users
                    / NULLIF(FIRST_VALUE(total_users) OVER w, 0), 2
                ) AS conversion_from_start_pct,
                ROUND(
                    100.0 * total_users
                    / NULLIF(LAG(total_users) OVER w, 0), 2
                ) AS stage_conversion_pct,
                CAST(RANK() OVER (PARTITION BY stage_order ORDER BY total_users DESC) AS BIGINT) AS ranking
            FROM stage_aggregated
            WHERE stage_order IS NOT NULL
            WINDOW w AS (PARTITION BY dimension ORDER BY stage_order)
        ) ranked
        WHERE ranking <= 10
        ORDER BY stage_order, total_users DESC
        "#
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(FunnelStage {
                stage_order: row.get(0)?,
                dimension: row.get(1)?,
                funnel_stage: row.get(2)?,
                total_users: row.get(3)?,
                total_interactions: row.get(4)?,
                prev_stage_users: row.get(5)?,
                users_dropped: row.get(6)?,
                dropoff_pct: row.get(7)?,
                conversion_from_start_pct: row.get(8)?,
                stage_conversion_pct: row.get(9)?,
                ranking: row.get(10)?,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(results)
}

pub fn query_scroll_depth(
    base_path: &str,
    project_id: Uuid,
    connector_id: Uuid,
    dimension: FunnelDimension,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<ScrollDepthData>, String> {
    let path = db_path(base_path, project_id, connector_id);
    if !path.exists() {
        return Err("No data available. Pull GA4 data first.".to_string());
    }

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open DuckDB: {}", e))?;

    let dim_expr = dimension.to_sql_expr();

    // Adapted query for DuckDB using ga4_events table
    let sql = format!(
        r#"
        WITH scroll_data AS (
            SELECT
                {dim_expr} AS dimension,
                event_name as scroll_depth,
                CAST(SUM(sessions) AS BIGINT) as events,
                CAST(SUM(active_users) AS BIGINT) as users
            FROM ga4_events
            WHERE event_name IN ('scroll_25', 'scroll_50', 'scroll_75', 'scroll_90', '25', '50', '75', '90')
                AND date >= ? AND date <= ?
            GROUP BY dimension, event_name
        ),
        scroll_with_lag AS (
            SELECT
                dimension,
                scroll_depth,
                events,
                users,
                CAST(LAG(users) OVER (PARTITION BY dimension ORDER BY
                    CAST(REPLACE(REPLACE(scroll_depth, 'scroll_', ''), '%', '') AS INTEGER)
                ) AS BIGINT) as prev_stage_users
            FROM scroll_data
        )
        SELECT
            dimension,
            scroll_depth,
            events,
            users,
            prev_stage_users,
            CASE
                WHEN prev_stage_users IS NULL THEN NULL
                ELSE ROUND(CAST((prev_stage_users - users) AS DOUBLE) / prev_stage_users * 100.0, 1)
            END as drop_off_pct,
            CASE
                WHEN prev_stage_users IS NULL THEN NULL
                ELSE CAST(prev_stage_users - users AS BIGINT)
            END as users_lost
        FROM scroll_with_lag
        ORDER BY dimension, CAST(REPLACE(REPLACE(scroll_depth, 'scroll_', ''), '%', '') AS INTEGER)
    "#
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(ScrollDepthData {
                dimension: row.get(0)?,
                scroll_depth: row.get(1)?,
                events: row.get(2)?,
                users: row.get(3)?,
                prev_stage_users: row.get(4)?,
                drop_off_pct: row.get(5)?,
                users_lost: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(results)
}

pub fn query_page_paths(
    base_path: &str,
    project_id: Uuid,
    connector_id: Uuid,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<PagePathAnalytics>, String> {
    let path = db_path(base_path, project_id, connector_id);
    if !path.exists() {
        return Err("No data available. Pull GA4 page path data first.".to_string());
    }

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open DuckDB: {}", e))?;

    let sql = r#"
        SELECT
            page_path,
            SUM(screen_page_views) as total_pageviews,
            SUM(total_users) as total_users,
            SUM(user_engagement_duration) as total_engagement_seconds,
            ROUND(SUM(user_engagement_duration) / NULLIF(SUM(screen_page_views), 0), 2) as avg_time_per_pageview_sec,
            ROUND(SUM(user_engagement_duration) / NULLIF(SUM(total_users), 0), 2) as avg_time_per_user_sec
        FROM ga4_page_paths
        WHERE date >= ? AND date <= ?
        GROUP BY page_path
        ORDER BY total_pageviews DESC
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(PagePathAnalytics {
                page_path: row.get(0)?,
                total_pageviews: row.get(1)?,
                total_users: row.get(2)?,
                total_engagement_seconds: row.get(3)?,
                avg_time_per_pageview_sec: row.get(4)?,
                avg_time_per_user_sec: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(results)
}

pub fn query_event_names(
    base_path: &str,
    project_id: Uuid,
    connector_id: Uuid,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<EventNameDebug>, String> {
    let path = db_path(base_path, project_id, connector_id);
    if !path.exists() {
        return Err("No data available. Pull GA4 data first.".to_string());
    }

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open DuckDB: {}", e))?;

    let sql = r#"
        SELECT
            event_name,
            CAST(SUM(sessions) AS BIGINT) as total_events,
            CAST(SUM(active_users) AS BIGINT) as total_users
        FROM ga4_events
        WHERE date >= ? AND date <= ?
        GROUP BY event_name
        ORDER BY total_events DESC
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(EventNameDebug {
                event_name: row.get(0)?,
                total_events: row.get(1)?,
                total_users: row.get(2)?,
            })
        })
        .map_err(|e| format!("Failed to execute query: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(results)
}
