use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::Project;

#[derive(Clone)]
pub struct ProjectRepository {
    pool: PgPool,
}

impl ProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, project: &Project) -> Result<Project, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            INSERT INTO projects (id, name, description)
            VALUES ($1, $2, $3)
            RETURNING id, name, description
            "#,
            project.id,
            project.name,
            project.description,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Project {
            id: row.id,
            name: row.name,
            description: row.description,
        })
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, description
            FROM projects
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Project {
            id: r.id,
            name: r.name,
            description: r.description,
        }))
    }

    pub async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, description
            FROM projects
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Project {
                id: r.id,
                name: r.name,
                description: r.description,
            })
            .collect())
    }

    pub async fn update(&self, project: &Project) -> Result<Project, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            UPDATE projects
            SET name = $2, description = $3
            WHERE id = $1
            RETURNING id, name, description
            "#,
            project.id,
            project.name,
            project.description,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Project {
            id: row.id,
            name: row.name,
            description: row.description,
        })
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM projects WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn has_connectors(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM connectors
            WHERE project_id = $1
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0) > 0)
    }
}
