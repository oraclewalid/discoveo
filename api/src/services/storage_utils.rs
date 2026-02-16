use std::path::PathBuf;
use uuid::Uuid;

/// Get the DuckDB data directory for a specific project and connector
pub fn get_data_dir(base_path: &str, project_id: Uuid, connector_id: Uuid) -> PathBuf {
    PathBuf::from(base_path)
        .join(project_id.to_string())
        .join(connector_id.to_string())
}
