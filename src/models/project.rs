use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl Project {
    pub fn new(id: Uuid, name: String, description: Option<String>) -> Self {
        Project {
            id,
            name,
            description,
        }
    }
}
