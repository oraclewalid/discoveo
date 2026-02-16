use chrono::NaiveDate;
use duckdb::{Connection, params};
use serde::Serialize;
use tracing::{debug, info};
use uuid::Uuid;

use super::ga4_service::{EventRecord, GA4Record, PagePathRecord, ReportType};
use super::storage_utils;

const LOOKBACK_DAYS: i64 = 2;
const DEFAULT_BACKFILL_DAYS: i64 = 90;

#[derive(Debug, Serialize)]
pub struct StorageResult {
    pub record_count: usize,
    pub inserted_count: usize,
    pub updated_count: usize,
}

pub fn store(
    base_path: &str,
    project_id: Uuid,
    connector_id: Uuid,
    records: Vec<GA4Record>,
    report_type: ReportType,
) -> Result<StorageResult, String> {
    info!(
        project_id = %project_id,
        connector_id = %connector_id,
        report_type = ?report_type,
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

    let dir = storage_utils::get_data_dir(base_path, project_id, connector_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;

    let db_path = dir.join("ga4.duckdb");
    debug!(db_path = %db_path.display(), "Opening DuckDB");

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open DuckDB: {}", e))?;
    debug!("DuckDB connection opened");

    // Create table based on report type
    create_table(&conn, report_type)?;

    // Check if table is empty (first sync)
    let table_name = report_type.table_name();
    let existing_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| row.get(0))
        .unwrap_or(0);

    let (inserted_count, updated_count) = if existing_count == 0 {
        // First sync: use fast bulk appender
        info!(report_type = ?report_type, "First sync detected, using bulk insert");
        bulk_insert(&conn, &records, report_type)?
    } else {
        // Incremental sync: use upsert for deduplication
        info!(
            report_type = ?report_type,
            existing_count = existing_count,
            "Incremental sync, using upsert"
        );
        upsert(&conn, &records, report_type)?
    };

    // Verify count in DuckDB
    let db_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| row.get(0))
        .unwrap_or(-1);

    info!(
        report_type = ?report_type,
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

fn create_table(conn: &Connection, report_type: ReportType) -> Result<(), String> {
    let create_sql = match report_type {
        ReportType::EventReport => {
            r#"
            CREATE TABLE IF NOT EXISTS ga4_events (
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
            "#
        }
        ReportType::PagePathReport => {
            r#"
            CREATE TABLE IF NOT EXISTS ga4_page_paths (
                date VARCHAR,
                page_path VARCHAR,
                screen_page_views BIGINT,
                total_users BIGINT,
                user_engagement_duration DOUBLE,
                PRIMARY KEY (date, page_path)
            );
            "#
        }
    };

    conn.execute_batch(create_sql)
        .map_err(|e| format!("Failed to create table: {}", e))?;
    debug!(report_type = ?report_type, "Table ready");
    Ok(())
}

/// Fast bulk insert using DuckDB appender (for first sync)
fn bulk_insert(conn: &Connection, records: &[GA4Record], report_type: ReportType) -> Result<(usize, usize), String> {
    let table_name = report_type.table_name();
    let mut appender = conn
        .appender(table_name)
        .map_err(|e| format!("Failed to create appender: {}", e))?;

    for record in records {
        match (record, report_type) {
            (GA4Record::EventReport(r), ReportType::EventReport) => {
                append_event_record(&mut appender, r)?;
            }
            (GA4Record::PagePathReport(r), ReportType::PagePathReport) => {
                append_page_path_record(&mut appender, r)?;
            }
            _ => return Err("Record type mismatch with report type".to_string()),
        }
    }

    Ok((records.len(), 0)) // All inserts, no updates
}

fn append_event_record(appender: &mut duckdb::Appender, r: &EventRecord) -> Result<(), String> {
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
        .map_err(|e| format!("Failed to append record: {}", e))
}

fn append_page_path_record(appender: &mut duckdb::Appender, r: &PagePathRecord) -> Result<(), String> {
    appender
        .append_row(params![
            r.date,
            r.page_path,
            r.screen_page_views,
            r.total_users,
            r.user_engagement_duration,
        ])
        .map_err(|e| format!("Failed to append record: {}", e))
}

/// Upsert using staging table for better performance (for incremental sync)
fn upsert(conn: &Connection, records: &[GA4Record], report_type: ReportType) -> Result<(usize, usize), String> {
    let table_name = report_type.table_name();
    let staging_table = format!("{}_staging", table_name);

    // Create staging table (no primary key for fast bulk insert)
    let create_staging_sql = match report_type {
        ReportType::EventReport => {
            format!(
                r#"
                DROP TABLE IF EXISTS {};
                CREATE TABLE {} (
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
                staging_table, staging_table
            )
        }
        ReportType::PagePathReport => {
            format!(
                r#"
                DROP TABLE IF EXISTS {};
                CREATE TABLE {} (
                    date VARCHAR,
                    page_path VARCHAR,
                    screen_page_views BIGINT,
                    total_users BIGINT,
                    user_engagement_duration DOUBLE
                );
                "#,
                staging_table, staging_table
            )
        }
    };

    conn.execute_batch(&create_staging_sql)
        .map_err(|e| format!("Failed to create staging table: {}", e))?;
    debug!(report_type = ?report_type, "Staging table created");

    // Bulk insert into staging using fast appender
    {
        let mut appender = conn
            .appender(&staging_table)
            .map_err(|e| format!("Failed to create staging appender: {}", e))?;

        for record in records {
            match (record, report_type) {
                (GA4Record::EventReport(r), ReportType::EventReport) => {
                    append_event_record(&mut appender, r)?;
                }
                (GA4Record::PagePathReport(r), ReportType::PagePathReport) => {
                    append_page_path_record(&mut appender, r)?;
                }
                _ => return Err("Record type mismatch with report type".to_string()),
            }
        }
    } // appender dropped here, flushes data
    debug!(report_type = ?report_type, records = records.len(), "Bulk inserted into staging");

    // Merge from staging to main table using INSERT OR REPLACE
    let merge_sql = format!(
        r#"
        INSERT OR REPLACE INTO {}
        SELECT * FROM {};
        DROP TABLE {};
        "#,
        table_name, staging_table, staging_table
    );

    conn.execute_batch(&merge_sql)
        .map_err(|e| format!("Failed to merge from staging: {}", e))?;
    debug!(report_type = ?report_type, "Merged staging to main table");

    Ok((records.len(), 0))
}

/// Get the start date for incremental sync.
/// Returns max_date - LOOKBACK_DAYS if data exists, otherwise today - DEFAULT_BACKFILL_DAYS.
pub fn get_incremental_start_date(
    base_path: &str,
    project_id: Uuid,
    connector_id: Uuid,
    report_type: ReportType,
) -> NaiveDate {
    let today = chrono::Utc::now().date_naive();
    let default_start = today - chrono::Duration::days(DEFAULT_BACKFILL_DAYS);

    let db_path = storage_utils::get_data_dir(base_path, project_id, connector_id).join("ga4.duckdb");

    if !db_path.exists() {
        info!(
            report_type = ?report_type,
            "No existing data, using default backfill of {} days",
            DEFAULT_BACKFILL_DAYS
        );
        return default_start;
    }

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            debug!(error = %e, "Failed to open DuckDB, using default start date");
            return default_start;
        }
    };

    let table_name = report_type.table_name();

    // Get max date from existing data (format: "YYYYMMDD")
    let max_date: Option<String> = conn
        .query_row(&format!("SELECT MAX(date) FROM {}", table_name), [], |row| row.get(0))
        .ok();

    match max_date {
        Some(date_str) => {
            // Parse "YYYYMMDD" format
            match NaiveDate::parse_from_str(&date_str, "%Y%m%d") {
                Ok(max_date) => {
                    let start = max_date - chrono::Duration::days(LOOKBACK_DAYS);
                    info!(
                        report_type = ?report_type,
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
            info!(
                report_type = ?report_type,
                "No existing records, using default backfill of {} days",
                DEFAULT_BACKFILL_DAYS
            );
            default_start
        }
    }
}
