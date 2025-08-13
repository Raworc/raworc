use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "session_state", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    Init,
    Ready,
    Idle,
    Busy,
    Error,
}

impl SessionState {
    pub fn can_transition_to(&self, target: &SessionState) -> bool {
        match (self, target) {
            // From INIT
            (SessionState::Init, SessionState::Ready) => true,
            (SessionState::Init, SessionState::Error) => true,
            
            // From READY
            (SessionState::Ready, SessionState::Idle) => true,  // After timeout
            (SessionState::Ready, SessionState::Busy) => true,  // Processing request
            (SessionState::Ready, SessionState::Error) => true,
            
            // From IDLE (container terminated, waiting for reactivation)
            (SessionState::Idle, SessionState::Ready) => true,  // User returns, restart container
            (SessionState::Idle, SessionState::Error) => true,
            
            // From BUSY (actively processing)
            (SessionState::Busy, SessionState::Ready) => true,  // Processing complete
            (SessionState::Busy, SessionState::Error) => true,
            
            // From ERROR
            (SessionState::Error, SessionState::Init) => true,  // Reset
            (SessionState::Error, SessionState::Ready) => true,  // Recovery
            
            // Cannot transition to same state
            _ => false,
        }
    }
    
    /// Check if this state indicates the container should be running
    pub fn requires_container(&self) -> bool {
        match self {
            SessionState::Ready | SessionState::Busy => true,  // Container needed
            SessionState::Init | SessionState::Idle | SessionState::Error => false,  // No container
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub workspace: String, // Organization that owns this session
    pub starting_prompt: String,
    pub state: SessionState,
    pub waiting_timeout_seconds: Option<i32>,
    pub container_id: Option<String>,
    pub persistent_volume_id: Option<String>,
    pub created_by: String,
    pub parent_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub termination_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    pub name: String,
    #[serde(default = "default_workspace")]
    pub workspace: String, // Organization for this session
    pub starting_prompt: String,
    #[serde(default)]
    pub agent_ids: Vec<Uuid>,
    #[serde(default = "default_timeout")]
    pub waiting_timeout_seconds: i32,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RemixSessionRequest {
    pub name: String,
    #[serde(default)]
    pub starting_prompt: Option<String>,
    #[serde(default)]
    pub agent_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub waiting_timeout_seconds: Option<i32>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSessionStateRequest {
    pub state: SessionState,
    #[serde(default)]
    pub container_id: Option<String>,
    #[serde(default)]
    pub persistent_volume_id: Option<String>,
    #[serde(default)]
    pub termination_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSessionRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub waiting_timeout_seconds: Option<i32>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SessionAgent {
    pub session_id: Uuid,
    pub agent_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub configuration: serde_json::Value,
}

fn default_timeout() -> i32 {
    300 // 5 minutes
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

fn default_workspace() -> String {
    "default".to_string()
}

// Database queries
impl Session {
    pub async fn find_all(pool: &sqlx::PgPool, workspace: Option<&str>, created_by: Option<&str>) -> Result<Vec<Session>, sqlx::Error> {
        let query = match (workspace, created_by) {
            (Some(ns), Some(user)) => {
                sqlx::query_as::<_, Session>(
                    r#"
                    SELECT id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                           container_id, persistent_volume_id, created_by, parent_session_id,
                           created_at, started_at, last_activity_at, terminated_at,
                           termination_reason, metadata, deleted_at
                    FROM sessions
                    WHERE workspace = $1 AND created_by = $2 AND deleted_at IS NULL
                    ORDER BY created_at DESC
                    "#
                )
                .bind(ns)
                .bind(user)
            },
            (Some(ns), None) => {
                sqlx::query_as::<_, Session>(
                    r#"
                    SELECT id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                           container_id, persistent_volume_id, created_by, parent_session_id,
                           created_at, started_at, last_activity_at, terminated_at,
                           termination_reason, metadata, deleted_at
                    FROM sessions
                    WHERE workspace = $1 AND deleted_at IS NULL
                    ORDER BY created_at DESC
                    "#
                )
                .bind(ns)
            },
            (None, Some(user)) => {
                sqlx::query_as::<_, Session>(
                    r#"
                    SELECT id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                           container_id, persistent_volume_id, created_by, parent_session_id,
                           created_at, started_at, last_activity_at, terminated_at,
                           termination_reason, metadata, deleted_at
                    FROM sessions
                    WHERE created_by = $1 AND deleted_at IS NULL
                    ORDER BY created_at DESC
                    "#
                )
                .bind(user)
            },
            (None, None) => {
                sqlx::query_as::<_, Session>(
                    r#"
                    SELECT id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                           container_id, persistent_volume_id, created_by, parent_session_id,
                           created_at, started_at, last_activity_at, terminated_at,
                           termination_reason, metadata, deleted_at
                    FROM sessions
                    WHERE deleted_at IS NULL
                    ORDER BY created_at DESC
                    "#
                )
            }
        };
        
        query.fetch_all(pool).await
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Session>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"
            SELECT id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                   container_id, persistent_volume_id, created_by, parent_session_id,
                   created_at, started_at, last_activity_at, terminated_at,
                   termination_reason, metadata, deleted_at
            FROM sessions
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        req: CreateSessionRequest,
        created_by: String,
    ) -> Result<Session, sqlx::Error> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (name, workspace, starting_prompt, waiting_timeout_seconds, created_by, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                      container_id, persistent_volume_id, created_by, parent_session_id,
                      created_at, started_at, last_activity_at, terminated_at,
                      termination_reason, metadata, deleted_at
            "#
        )
        .bind(&req.name)
        .bind(&req.workspace)
        .bind(&req.starting_prompt)
        .bind(req.waiting_timeout_seconds)
        .bind(&created_by)
        .bind(&req.metadata)
        .fetch_one(pool)
        .await?;

        // Assign agents if provided
        if !req.agent_ids.is_empty() {
            Self::assign_agents(pool, session.id, &req.agent_ids).await?;
        }

        Ok(session)
    }

    pub async fn remix(
        pool: &sqlx::PgPool,
        parent_id: Uuid,
        req: RemixSessionRequest,
        created_by: String,
    ) -> Result<Session, sqlx::Error> {
        // Get parent session
        let parent = Self::find_by_id(pool, parent_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        // Create new session based on parent
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (
                name, workspace, starting_prompt, waiting_timeout_seconds, 
                created_by, parent_session_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, workspace, starting_prompt, state, waiting_timeout_seconds,
                      container_id, persistent_volume_id, created_by, parent_session_id,
                      created_at, started_at, last_activity_at, terminated_at,
                      termination_reason, metadata, deleted_at
            "#
        )
        .bind(&req.name)
        .bind(&parent.workspace) // Inherit workspace from parent
        .bind(req.starting_prompt.as_ref().unwrap_or(&parent.starting_prompt))
        .bind(req.waiting_timeout_seconds.unwrap_or(parent.waiting_timeout_seconds.unwrap_or(300)))
        .bind(&created_by)
        .bind(parent_id)
        .bind(req.metadata.as_ref().unwrap_or(&parent.metadata))
        .fetch_one(pool)
        .await?;

        // Assign agents - use provided or copy from parent
        if let Some(agent_ids) = req.agent_ids {
            if !agent_ids.is_empty() {
                Self::assign_agents(pool, session.id, &agent_ids).await?;
            }
        } else {
            // Copy agents from parent session
            Self::copy_agents_from_parent(pool, session.id, parent_id).await?;
        }

        Ok(session)
    }

    pub async fn update_state(
        pool: &sqlx::PgPool,
        id: Uuid,
        req: UpdateSessionStateRequest,
    ) -> Result<Option<Session>, sqlx::Error> {
        // Check current state and validate transition
        let current = Self::find_by_id(pool, id).await?;
        if let Some(session) = current {
            if !session.state.can_transition_to(&req.state) {
                return Err(sqlx::Error::Protocol(format!(
                    "Invalid state transition from {:?} to {:?}",
                    session.state, req.state
                )));
            }
        } else {
            return Ok(None);
        }

        let now = Utc::now();
        let mut query_builder = String::from("UPDATE sessions SET state = $1, last_activity_at = $2");
        let mut param_count = 2;

        // Add optional fields based on state transition
        if req.state == SessionState::Ready {
            param_count += 1;
            query_builder.push_str(&format!(", started_at = ${}", param_count));
        }

        if req.state == SessionState::Error {
            param_count += 1;
            query_builder.push_str(&format!(", terminated_at = ${}", param_count));
            if req.termination_reason.is_some() {
                param_count += 1;
                query_builder.push_str(&format!(", termination_reason = ${}", param_count));
            }
        }

        if req.container_id.is_some() {
            param_count += 1;
            query_builder.push_str(&format!(", container_id = ${}", param_count));
        }

        if req.persistent_volume_id.is_some() {
            param_count += 1;
            query_builder.push_str(&format!(", persistent_volume_id = ${}", param_count));
        }

        query_builder.push_str(" WHERE id = $");
        param_count += 1;
        query_builder.push_str(&param_count.to_string());
        query_builder.push_str(" RETURNING id, name, workspace, starting_prompt, state, waiting_timeout_seconds, container_id, persistent_volume_id, created_by, parent_session_id, created_at, started_at, last_activity_at, terminated_at, termination_reason, metadata, deleted_at");

        // Build and execute query
        let mut query = sqlx::query_as::<_, Session>(&query_builder)
            .bind(req.state)
            .bind(now);

        if req.state == SessionState::Ready {
            query = query.bind(now);
        }

        if req.state == SessionState::Error {
            query = query.bind(now);
            if let Some(reason) = req.termination_reason {
                query = query.bind(reason);
            }
        }

        if let Some(container_id) = req.container_id {
            query = query.bind(container_id);
        }

        if let Some(pv_id) = req.persistent_volume_id {
            query = query.bind(pv_id);
        }

        query = query.bind(id);

        query.fetch_optional(pool).await
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        id: Uuid,
        req: UpdateSessionRequest,
    ) -> Result<Option<Session>, sqlx::Error> {
        let mut query_builder = String::from("UPDATE sessions SET");
        let mut updates = Vec::new();
        let mut param_count = 0;

        if let Some(_name) = &req.name {
            param_count += 1;
            updates.push(format!(" name = ${}", param_count));
        }

        if let Some(_timeout) = req.waiting_timeout_seconds {
            param_count += 1;
            updates.push(format!(" waiting_timeout_seconds = ${}", param_count));
        }

        if let Some(_metadata) = &req.metadata {
            param_count += 1;
            updates.push(format!(" metadata = ${}", param_count));
        }

        if updates.is_empty() {
            return Err(sqlx::Error::Protocol("No fields to update".to_string()));
        }

        query_builder.push_str(&updates.join(","));
        query_builder.push_str(" WHERE id = $");
        param_count += 1;
        query_builder.push_str(&param_count.to_string());
        query_builder.push_str(" AND deleted_at IS NULL");
        query_builder.push_str(" RETURNING id, name, workspace, starting_prompt, state, waiting_timeout_seconds, container_id, persistent_volume_id, created_by, parent_session_id, created_at, started_at, last_activity_at, terminated_at, termination_reason, metadata, deleted_at");

        let mut query = sqlx::query_as::<_, Session>(&query_builder);

        if let Some(name) = req.name {
            query = query.bind(name);
        }

        if let Some(timeout) = req.waiting_timeout_seconds {
            query = query.bind(timeout);
        }

        if let Some(metadata) = req.metadata {
            query = query.bind(metadata);
        }

        query = query.bind(id);

        query.fetch_optional(pool).await
    }

    pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE sessions SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_agents(pool: &sqlx::PgPool, session_id: Uuid) -> Result<Vec<crate::models::Agent>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::Agent>(
            r#"
            SELECT a.id, a.name, a.workspace, a.description, a.instructions, a.model,
                   a.tools, a.routes, a.guardrails, a.knowledge_bases,
                   a.active, a.created_at, a.updated_at, a.deleted_at
            FROM agents a
            JOIN session_agents sa ON a.id = sa.agent_id
            WHERE sa.session_id = $1
            ORDER BY sa.assigned_at
            "#
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    async fn assign_agents(pool: &sqlx::PgPool, session_id: Uuid, agent_ids: &[Uuid]) -> Result<(), sqlx::Error> {
        for agent_id in agent_ids {
            sqlx::query(
                "INSERT INTO session_agents (session_id, agent_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(session_id)
            .bind(agent_id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    async fn copy_agents_from_parent(pool: &sqlx::PgPool, session_id: Uuid, parent_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO session_agents (session_id, agent_id, configuration)
            SELECT $1, agent_id, configuration
            FROM session_agents
            WHERE session_id = $2
            "#
        )
        .bind(session_id)
        .bind(parent_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_waiting_sessions_to_timeout(pool: &sqlx::PgPool) -> Result<Vec<Session>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"
            SELECT id, name, starting_prompt, state, waiting_timeout_seconds,
                   container_id, persistent_volume_id, created_by, parent_session_id,
                   created_at, started_at, last_activity_at, terminated_at,
                   termination_reason, metadata, deleted_at
            FROM sessions
            WHERE state = 'READY'
              AND waiting_timeout_seconds IS NOT NULL
              AND last_activity_at IS NOT NULL
              AND last_activity_at + (waiting_timeout_seconds || ' seconds')::interval < NOW()
              AND deleted_at IS NULL
            "#
        )
        .fetch_all(pool)
        .await
    }
}