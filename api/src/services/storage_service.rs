use chrono::NaiveDate;
use duckdb::{Connection, params};
use serde::Serialize;
use std::path::PathBuf;
use tracing::{debug, info};
use uuid::Uuid;

use super::ga4_service::GA4Record;

const DATA_DIR: &str = "/tmp/ga4_data";
const LOOKBACK_DAYS: i64 = 2;
const DEFAULT_BACKFILL_DAYS: i64 = 30;

#[derive(Debug, Serialize)]
pub struct StorageResult {
    pub record_count: usize,
    pub inserted_count: usize,
    pub updated_count: usize,
}

pub fn store(
    project_id: Uuid,
    connector_id: Uuid,
    records: Vec<GA4Record>,
) -> Result<StorageResult, String> {
    info!(
        project_id = %project_id,
        connector_id = %connector_id,
        incoming_records = records.len(),
        "Starting storage"
    );

    if records.is_empty() {
        info!("No records to store, skipping");
        return Ok(StorageResult {
            record_count: 0,
            inserted_count: 0,
            updated_count: 0,
        });
    }

    let dir = data_dir(project_id, connector_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    let db_path = dir.join("ga4.duckdb");
    debug!(db_path = %db_path.display(), "Opening DuckDB");

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open DuckDB: {}", e))?;
    debug!("DuckDB connection opened");

    // Create table if not exists with primary key for deduplication
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS ga4_records (
            date VARCHAR,
            country VARCHAR,
            device_category VARCHAR,
            event_name VARCHAR,
            browser VARCHAR,
            operating_system VARCHAR,
            screen_resolution VARCHAR,
            active_users BIGINT,
            sessions BIGINT,
            screen_page_views BIGINT,
            bounce_rate DOUBLE,
            average_session_duration DOUBLE,
            PRIMARY KEY (date, country, device_category, event_name, browser, operating_system, screen_resolution)
        );
        "#,
    )
    .map_err(|e| format!("Failed to create table: {}", e))?;
    debug!("Table ready");

    // Check if table is empty (first sync)
    let existing_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ga4_records", [], |row| row.get(0))
        .unwrap_or(0);

    let (inserted_count, updated_count) = if existing_count == 0 {
        // First sync: use fast bulk appender
        info!("First sync detected, using bulk insert");
        bulk_insert(&conn, &records)?
    } else {
        // Incremental sync: use upsert for deduplication
        info!(existing_count = existing_count, "Incremental sync, using upsert");
        upsert(&conn, &records)?
    };

    // Verify count in DuckDB
    let db_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ga4_records", [], |row| row.get(0))
        .unwrap_or(-1);

    info!(
        incoming_records = records.len(),
        inserted = inserted_count,
        updated = updated_count,
        db_count = db_count,
        "Data stored"
    );

    Ok(StorageResult {
        record_count: records.len(),
        inserted_count,
        updated_count,
    })
}

fn data_dir(project_id: Uuid, connector_id: Uuid) -> PathBuf {
    PathBuf::from(DATA_DIR)
        .join(project_id.to_string())
        .join(connector_id.to_string())
}

/// Fast bulk insert using DuckDB appender (for first sync)
fn bulk_insert(conn: &Connection, records: &[GA4Record]) -> Result<(usize, usize), String> {
    let mut appender = conn
        .appender("ga4_records")
        .map_err(|e| format!("Failed to create appender: {}", e))?;

    for r in records {
        appender
            .append_row(params![
                r.date,
                r.country,
                r.device_category,
                r.event_name,
                r.browser,
                r.operating_system,
                r.screen_resolution,
                r.active_users,
                r.sessions,
                r.screen_page_views,
                r.bounce_rate,
                r.average_session_duration,
            ])
            .map_err(|e| format!("Failed to append record: {}", e))?;
    }

    Ok((records.len(), 0)) // All inserts, no updates
}

/// Upsert using staging table for better performance (for incremental sync)
/// 1. Bulk insert into staging table (fast appender, no constraints)
/// 2. Single INSERT OR REPLACE from staging to main table
/// 3. Drop staging table
fn upsert(conn: &Connection, records: &[GA4Record]) -> Result<(usize, usize), String> {
    // Create staging table (no primary key for fast bulk insert)
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS ga4_staging;
        CREATE TABLE ga4_staging (
            date VARCHAR,
            country VARCHAR,
            device_category VARCHAR,
            event_name VARCHAR,
            browser VARCHAR,
            operating_system VARCHAR,
            screen_resolution VARCHAR,
            active_users BIGINT,
            sessions BIGINT,
            screen_page_views BIGINT,
            bounce_rate DOUBLE,
            average_session_duration DOUBLE
        );
        "#,
    )
    .map_err(|e| format!("Failed to create staging table: {}", e))?;
    debug!("Staging table created");

    // Bulk insert into staging using fast appender
    {
        let mut appender = conn
            .appender("ga4_staging")
            .map_err(|e| format!("Failed to create staging appender: {}", e))?;

        for r in records {
            appender
                .append_row(params![
                    r.date,
                    r.country,
                    r.device_category,
                    r.event_name,
                    r.browser,
                    r.operating_system,
                    r.screen_resolution,
                    r.active_users,
                    r.sessions,
                    r.screen_page_views,
                    r.bounce_rate,
                    r.average_session_duration,
                ])
                .map_err(|e| format!("Failed to append to staging: {}", e))?;
        }
    } // appender dropped here, flushes data
    debug!(records = records.len(), "Bulk inserted into staging");

    // Merge from staging to main table using INSERT OR REPLACE
    conn.execute_batch(
        r#"
        INSERT OR REPLACE INTO ga4_records
        SELECT * FROM ga4_staging;
        DROP TABLE ga4_staging;
        "#,
    )
    .map_err(|e| format!("Failed to merge from staging: {}", e))?;
    debug!("Merged staging to main table");

    Ok((records.len(), 0))
}

/// Get the start date for incremental sync.
/// Returns max_date - LOOKBACK_DAYS if data exists, otherwise today - DEFAULT_BACKFILL_DAYS.
pub fn get_incremental_start_date(
    project_id: Uuid,
    connector_id: Uuid,
) -> NaiveDate {
    let today = chrono::Utc::now().date_naive();
    let default_start = today - chrono::Duration::days(DEFAULT_BACKFILL_DAYS);

    let db_path = data_dir(project_id, connector_id).join("ga4.duckdb");

    if !db_path.exists() {
        info!("No existing data, using default backfill of {} days", DEFAULT_BACKFILL_DAYS);
        return default_start;
    }

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            debug!(error = %e, "Failed to open DuckDB, using default start date");
            return default_start;
        }
    };

    // Get max date from existing data (format: "YYYYMMDD")
    let max_date: Option<String> = conn
        .query_row("SELECT MAX(date) FROM ga4_records", [], |row| row.get(0))
        .ok();

    match max_date {
        Some(date_str) => {
            // Parse "YYYYMMDD" format
            match NaiveDate::parse_from_str(&date_str, "%Y%m%d") {
                Ok(max_date) => {
                    let start = max_date - chrono::Duration::days(LOOKBACK_DAYS);
                    info!(
                        max_date = %date_str,
                        start_date = %start.format("%Y%m%d"),
                        lookback_days = LOOKBACK_DAYS,
                        "Incremental sync from existing data"
                    );
                    start
                }
                Err(e) => {
                    debug!(error = %e, date_str = %date_str, "Failed to parse max date, using default");
                    default_start
                }
            }
        }
        None => {
            info!("No existing records, using default backfill of {} days", DEFAULT_BACKFILL_DAYS);
            default_start
        }
    }
}
