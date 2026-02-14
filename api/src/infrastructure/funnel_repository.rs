use duckdb::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

const DATA_DIR: &str = "/tmp/ga4_data";

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

fn db_path(project_id: Uuid, connector_id: Uuid) -> PathBuf {
    PathBuf::from(DATA_DIR)
        .join(project_id.to_string())
        .join(connector_id.to_string())
        .join("ga4.duckdb")
}

pub fn query_funnel(
    project_id: Uuid,
    connector_id: Uuid,
    dimension: FunnelDimension,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<FunnelStage>, String> {
    let path = db_path(project_id, connector_id);
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
            FROM ga4_records
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
