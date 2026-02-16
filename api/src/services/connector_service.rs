use duckdb::Connection;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::infrastructure::connector_repository::ConnectorRepository;
use crate::models::connector::Connector;
use super::storage_utils;

#[derive(Clone)]
pub struct ConnectorService {
    repository: ConnectorRepository,
    duckdb_base_path: String,
}

impl ConnectorService {
    pub fn new(repository: ConnectorRepository, duckdb_base_path: String) -> Self {
        Self {
            repository,
            duckdb_base_path,
        }
    }

    /// Delete a connector and drop the GA4 table from DuckDB (keeps the database file)
    pub async fn delete(&self, connector_id: Uuid) -> Result<bool, String> {
        // Get the connector to retrieve project_id
        let connector = self
            .repository
            .find_by_id(connector_id)
            .await
            .map_err(|e| format!("Failed to find connector: {}", e))?;

        let connector = match connector {
            Some(c) => c,
            None => return Ok(false), // Connector doesn't exist
        };

        // Delete from PostgreSQL database
        let deleted = self
            .repository
            .delete(connector_id)
            .await
            .map_err(|e| format!("Failed to delete connector from database: {}", e))?;

        if !deleted {
            return Ok(false);
        }

        // Drop GA4 table from DuckDB (keep the database file)
        self.drop_ga4_table(connector.project_id, connector_id)?;

        info!(
            connector_id = %connector_id,
            project_id = %connector.project_id,
            "Connector deleted and GA4 table dropped successfully"
        );

        Ok(true)
    }

    /// Drop all GA4 tables from DuckDB (keeps the database file)
    fn drop_ga4_table(&self, project_id: Uuid, connector_id: Uuid) -> Result<(), String> {
        let data_dir = storage_utils::get_data_dir(&self.duckdb_base_path, project_id, connector_id);
        let db_path = data_dir.join("ga4.duckdb");

        if !db_path.exists() {
            info!(
                path = %db_path.display(),
                "DuckDB database does not exist, nothing to drop"
            );
            return Ok(());
        }

        let conn = Connection::open(&db_path).map_err(|e| {
            error!(
                path = %db_path.display(),
                error = %e,
                "Failed to open DuckDB database"
            );
            format!("Failed to open DuckDB database: {}", e)
        })?;

        // Drop all GA4 tables
        let tables = vec!["ga4_events", "ga4_page_paths", "ga4_records"];

        for table in tables {
            match conn.execute(&format!("DROP TABLE IF EXISTS {}", table), []) {
                Ok(_) => {
                    info!(
                        path = %db_path.display(),
                        table = table,
                        "GA4 table dropped successfully"
                    );
                }
                Err(e) => {
                    warn!(
                        path = %db_path.display(),
                        table = table,
                        error = %e,
                        "Failed to drop GA4 table (may not exist)"
                    );
                }
            }
        }

        Ok(())
    }

    // Proxy methods to repository for other operations
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Connector>, sqlx::Error> {
        self.repository.find_by_id(id).await
    }
}
