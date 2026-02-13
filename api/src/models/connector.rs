use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectorType {
    Ga4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectorDetails {
    Ga4 {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
        token_type: String,
        property_id: Option<String>,
        property_name: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Connector {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    #[sqlx(rename = "type")]
    pub connector_type: ConnectorType,
    pub config: JsonValue,
}

impl Connector {
    pub fn new(id: Uuid, project_id: Uuid, name: String, connector_type: ConnectorType, config: ConnectorDetails) -> Self {
        Connector {
            id,
            project_id,
            name,
            connector_type,
            config: serde_json::to_value(config).unwrap_or(JsonValue::Null),
        }
    }
}
