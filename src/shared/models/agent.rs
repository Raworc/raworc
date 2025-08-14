use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub workspace: String, // Organization that owns this agent
    pub description: Option<String>,
    pub instructions: String,
    pub model: String,
    pub tools: serde_json::Value,
    pub routes: serde_json::Value,
    pub guardrails: serde_json::Value,
    pub knowledge_bases: serde_json::Value,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAgentRequest {
    pub name: String,
    #[serde(default = "default_workspace")]
    pub workspace: String, // Organization for this agent
    pub description: Option<String>,
    pub instructions: String,
    pub model: String,
    #[serde(default = "default_json_array")]
    pub tools: serde_json::Value,
    #[serde(default = "default_json_array")]
    pub routes: serde_json::Value,
    #[serde(default = "default_json_array")]
    pub guardrails: serde_json::Value,
    #[serde(default = "default_json_array")]
    pub knowledge_bases: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub model: Option<String>,
    pub tools: Option<serde_json::Value>,
    pub routes: Option<serde_json::Value>,
    pub guardrails: Option<serde_json::Value>,
    pub knowledge_bases: Option<serde_json::Value>,
    pub active: Option<bool>,
}

fn default_json_array() -> serde_json::Value {
    serde_json::json!([])
}

fn default_workspace() -> String {
    "default".to_string()
}

// Database queries
impl Agent {
    pub async fn find_all(pool: &sqlx::PgPool, workspace: Option<&str>) -> Result<Vec<Agent>, sqlx::Error> {
        let query = if let Some(ns) = workspace {
            sqlx::query_as::<_, Agent>(
                r#"
                SELECT id, name, workspace, description, instructions, model, 
                       tools, routes, guardrails, knowledge_bases,
                       active, created_at, updated_at
                FROM agents
                WHERE active = true AND workspace = $1
                ORDER BY name ASC
                "#
            )
            .bind(ns)
        } else {
            sqlx::query_as::<_, Agent>(
                r#"
                SELECT id, name, workspace, description, instructions, model, 
                       tools, routes, guardrails, knowledge_bases,
                       active, created_at, updated_at
                FROM agents
                WHERE active = true
                ORDER BY name ASC
                "#
            )
        };
        
        query.fetch_all(pool).await
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Agent>, sqlx::Error> {
        sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, name, workspace, description, instructions, model,
                   tools, routes, guardrails, knowledge_bases,
                   active, created_at, updated_at
            FROM agents
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_name(pool: &sqlx::PgPool, name: &str, workspace: &str) -> Result<Option<Agent>, sqlx::Error> {
        sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, name, workspace, description, instructions, model,
                   tools, routes, guardrails, knowledge_bases,
                   active, created_at, updated_at
            FROM agents
            WHERE name = $1 AND workspace = $2
            "#
        )
        .bind(name)
        .bind(workspace)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &sqlx::PgPool, req: CreateAgentRequest) -> Result<Agent, sqlx::Error> {
        sqlx::query_as::<_, Agent>(
            r#"
            INSERT INTO agents (name, workspace, description, instructions, model, tools, routes, guardrails, knowledge_bases)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, name, workspace, description, instructions, model,
                      tools, routes, guardrails, knowledge_bases,
                      active, created_at, updated_at
            "#
        )
        .bind(req.name)
        .bind(req.workspace)
        .bind(req.description)
        .bind(req.instructions)
        .bind(req.model)
        .bind(req.tools)
        .bind(req.routes)
        .bind(req.guardrails)
        .bind(req.knowledge_bases)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &sqlx::PgPool, id: Uuid, req: UpdateAgentRequest) -> Result<Option<Agent>, sqlx::Error> {
        // Build dynamic update query based on provided fields
        let result = sqlx::query_as::<_, Agent>(
            r#"
            UPDATE agents
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                instructions = COALESCE($4, instructions),
                model = COALESCE($5, model),
                tools = COALESCE($6, tools),
                routes = COALESCE($7, routes),
                guardrails = COALESCE($8, guardrails),
                knowledge_bases = COALESCE($9, knowledge_bases),
                active = COALESCE($10, active)
            WHERE id = $1
            RETURNING id, name, workspace, description, instructions, model,
                      tools, routes, guardrails, knowledge_bases,
                      active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(req.name)
        .bind(req.description)
        .bind(req.instructions)
        .bind(req.model)
        .bind(req.tools)
        .bind(req.routes)
        .bind(req.guardrails)
        .bind(req.knowledge_bases)
        .bind(req.active)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE agents
            SET active = false
            WHERE id = $1 AND active = true
            "#
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}