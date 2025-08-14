use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "message_role", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageRole {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SessionMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub agent_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub role: MessageRole,
    pub content: String,
    pub agent_id: Option<Uuid>,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,  // Populated from join
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    #[allow(dead_code)]
    pub role: Option<MessageRole>,
    #[allow(dead_code)]
    pub since: Option<DateTime<Utc>>,
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

// Database operations
impl SessionMessage {
    pub async fn create(
        pool: &sqlx::PgPool,
        session_id: Uuid,
        req: CreateMessageRequest,
    ) -> Result<SessionMessage, sqlx::Error> {
        // Note: Database constraint ensures agent_id is set when role is AGENT
        sqlx::query_as::<_, SessionMessage>(
            r#"
            INSERT INTO session_messages (
                session_id, role, content, agent_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, session_id, role, content, agent_id, 
                      metadata, created_at
            "#
        )
        .bind(session_id)
        .bind(req.role)
        .bind(&req.content)
        .bind(req.agent_id)
        .bind(&req.metadata)
        .fetch_one(pool)
        .await
    }

    #[allow(dead_code)]
    pub async fn find_by_session(
        pool: &sqlx::PgPool,
        session_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SessionMessage>, sqlx::Error> {
        let limit = limit.unwrap_or(100).min(1000);  // Max 1000 messages
        let offset = offset.unwrap_or(0);
        
        sqlx::query_as::<_, SessionMessage>(
            r#"
            SELECT id, session_id, role, content, agent_id,
                   metadata, created_at
            FROM session_messages
            WHERE session_id = $1
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    #[allow(dead_code)]
    pub async fn find_by_session_with_filter(
        pool: &sqlx::PgPool,
        session_id: Uuid,
        query: ListMessagesQuery,
    ) -> Result<Vec<SessionMessage>, sqlx::Error> {
        let limit = query.limit.unwrap_or(100).min(1000);
        let offset = query.offset.unwrap_or(0);
        
        let mut sql = String::from(
            r#"
            SELECT id, session_id, role, content, agent_id,
                   metadata, created_at
            FROM session_messages
            WHERE session_id = $1
            "#
        );
        
        let mut param_count = 1;
        
        if query.role.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND role = ${}", param_count));
        }
        
        if query.since.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND created_at > ${}", param_count));
        }
        
        sql.push_str(" ORDER BY created_at ASC");
        param_count += 1;
        sql.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        sql.push_str(&format!(" OFFSET ${}", param_count));
        
        let mut query_builder = sqlx::query_as::<_, SessionMessage>(&sql)
            .bind(session_id);
        
        if let Some(role) = query.role {
            query_builder = query_builder.bind(role);
        }
        
        if let Some(since) = query.since {
            query_builder = query_builder.bind(since);
        }
        
        query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    pub async fn count_by_session(
        pool: &sqlx::PgPool,
        session_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM session_messages WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_one(pool)
        .await?;
        
        Ok(result)
    }

    pub async fn delete_by_session(
        pool: &sqlx::PgPool,
        session_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM session_messages WHERE session_id = $1"
        )
        .bind(session_id)
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected())
    }

    pub async fn get_with_agent_info(
        pool: &sqlx::PgPool,
        session_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<MessageResponse>, sqlx::Error> {
        let limit = limit.unwrap_or(100).min(1000);
        let offset = offset.unwrap_or(0);
        
        let messages = sqlx::query!(
            r#"
            SELECT 
                m.id, m.session_id, m.role as "role: MessageRole", 
                m.content, m.agent_id, 
                m.metadata, m.created_at,
                a.name as "agent_name?"
            FROM session_messages m
            LEFT JOIN agents a ON m.agent_id = a.id
            WHERE m.session_id = $1
            ORDER BY m.created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            session_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;
        
        Ok(messages.into_iter().map(|m| MessageResponse {
            id: m.id.to_string(),
            session_id: m.session_id.to_string(),
            role: m.role,
            content: m.content,
            agent_id: m.agent_id.map(|id| id.to_string()),
            agent_name: m.agent_name,
            metadata: m.metadata.unwrap_or_else(|| serde_json::json!({})),
            created_at: m.created_at.to_rfc3339(),
        }).collect())
    }
}