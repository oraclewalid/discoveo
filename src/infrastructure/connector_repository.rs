use sqlx::PgPool;
use uuid::Uuid;

use crate::models::connector::{Connector, ConnectorType};

#[derive(Clone)]
pub struct ConnectorRepository {
    pool: PgPool,
}

impl ConnectorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, connector: &Connector) -> Result<Connector, sqlx::Error> {
        let connector_type_str = connector.connector_type.to_string();
        let row = sqlx::query!(
            r#"
            INSERT INTO connectors (id, project_id, name, type, config)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, project_id, name, type, config
            "#,
            connector.id,
            connector.project_id,
            connector.name,
            connector_type_str,
            connector.config,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Connector {
            id: row.id,
            project_id: row.project_id,
            name: row.name,
            connector_type: row.r#type.parse().unwrap(),
            config: row.config,
        })
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Connector>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, project_id, name, type, config
            FROM connectors
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Connector {
            id: r.id,
            project_id: r.project_id,
            name: r.name,
            connector_type: r.r#type.parse().unwrap(),
            config: r.config,
        }))
    }

    pub async fn find_all(&self) -> Result<Vec<Connector>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, project_id, name, type, config
            FROM connectors
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Connector {
                id: r.id,
                project_id: r.project_id,
                name: r.name,
                connector_type: r.r#type.parse().unwrap(),
                config: r.config,
            })
            .collect())
    }

    pub async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Connector>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, project_id, name, type, config
            FROM connectors
            WHERE project_id = $1
            "#,
            project_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Connector {
                id: r.id,
                project_id: r.project_id,
                name: r.name,
                connector_type: r.r#type.parse().unwrap(),
                config: r.config,
            })
            .collect())
    }

    pub async fn find_by_type(&self, connector_type: ConnectorType) -> Result<Vec<Connector>, sqlx::Error> {
        let connector_type_str = connector_type.to_string();
        let rows = sqlx::query!(
            r#"
            SELECT id, project_id, name, type, config
            FROM connectors
            WHERE type = $1
            "#,
            connector_type_str,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Connector {
                id: r.id,
                project_id: r.project_id,
                name: r.name,
                connector_type: r.r#type.parse().unwrap(),
                config: r.config,
            })
            .collect())
    }

    pub async fn find_by_project_and_type(
        &self,
        project_id: Uuid,
        connector_type: ConnectorType,
    ) -> Result<Vec<Connector>, sqlx::Error> {
        let connector_type_str = connector_type.to_string();
        let rows = sqlx::query!(
            r#"
            SELECT id, project_id, name, type, config
            FROM connectors
            WHERE project_id = $1 AND type = $2
            "#,
            project_id,
            connector_type_str,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Connector {
                id: r.id,
                project_id: r.project_id,
                name: r.name,
                connector_type: r.r#type.parse().unwrap(),
                config: r.config,
            })
            .collect())
    }

    pub async fn update(&self, connector: &Connector) -> Result<Connector, sqlx::Error> {
        let connector_type_str = connector.connector_type.to_string();
        let row = sqlx::query!(
            r#"
            UPDATE connectors
            SET name = $2, type = $3, config = $4
            WHERE id = $1
            RETURNING id, project_id, name, type, config
            "#,
            connector.id,
            connector.name,
            connector_type_str,
            connector.config,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Connector {
            id: row.id,
            project_id: row.project_id,
            name: row.name,
            connector_type: row.r#type.parse().unwrap(),
            config: row.config,
        })
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM connectors WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
