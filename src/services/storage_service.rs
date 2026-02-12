use duckdb::{Connection, params};
use serde::Serialize;
use std::path::PathBuf;
use tracing::{debug, info};
use uuid::Uuid;

use super::ga4_service::GA4Record;

const DATA_DIR: &str = "/tmp/ga4_data";

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

    // Drop and recreate table to ensure clean state
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS ga4_records;
        CREATE TABLE ga4_records (
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
    .map_err(|e| format!("Failed to create table: {}", e))?;
    debug!("Table created");

    // Bulk insert using appender (much faster than individual inserts)
    {
        let mut appender = conn
            .appender("ga4_records")
            .map_err(|e| format!("Failed to create appender: {}", e))?;

        for r in &records {
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
    } // appender is flushed and dropped here

    let inserted = records.len();

    // Verify count in DuckDB
    let db_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ga4_records", [], |row| row.get(0))
        .unwrap_or(-1);

    info!(
        incoming_records = records.len(),
        inserted = inserted,
        db_count = db_count,
        "Data stored"
    );

    Ok(StorageResult {
        record_count: records.len(),
        inserted_count: inserted,
        updated_count: 0,
    })
}

fn data_dir(project_id: Uuid, connector_id: Uuid) -> PathBuf {
    PathBuf::from(DATA_DIR)
        .join(project_id.to_string())
        .join(connector_id.to_string())
}
