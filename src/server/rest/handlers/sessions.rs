use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::shared::models::{AppState, Session, SessionState, CreateSessionRequest, RemixSessionRequest, UpdateSessionStateRequest, UpdateSessionRequest};
use crate::server::rest::error::{ApiError, ApiResult};
use crate::server::rest::middleware::AuthContext;
use crate::server::rest::rbac_enforcement::{check_api_permission, permissions};

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub name: String,
    pub workspace: String,
    pub starting_prompt: String,
    pub state: SessionState,
    pub waiting_timeout_seconds: Option<i32>,
    pub container_id: Option<String>,
    pub persistent_volume_id: Option<String>,
    pub created_by: String,
    pub parent_session_id: Option<String>,
    pub agents: Vec<SessionAgentInfo>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub terminated_at: Option<String>,
    pub termination_reason: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionAgentInfo {
    pub id: String,
    pub name: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub workspace: Option<String>,
    pub created_by: Option<String>,
    pub state: Option<SessionState>,
}

impl SessionResponse {
    async fn from_session(session: Session, pool: &sqlx::PgPool) -> Result<Self, ApiError> {
        let agents = Session::get_agents(pool, session.id)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch session agents: {}", e)))?
            .into_iter()
            .map(|agent| SessionAgentInfo {
                id: agent.id.to_string(),
                name: agent.name,
                model: agent.model,
            })
            .collect();

        Ok(Self {
            id: session.id.to_string(),
            name: session.name,
            workspace: session.workspace,
            starting_prompt: session.starting_prompt,
            state: session.state,
            waiting_timeout_seconds: session.waiting_timeout_seconds,
            container_id: session.container_id,
            persistent_volume_id: session.persistent_volume_id,
            created_by: session.created_by,
            parent_session_id: session.parent_session_id.map(|id| id.to_string()),
            agents,
            created_at: session.created_at.to_rfc3339(),
            started_at: session.started_at.map(|dt| dt.to_rfc3339()),
            last_activity_at: session.last_activity_at.map(|dt| dt.to_rfc3339()),
            terminated_at: session.terminated_at.map(|dt| dt.to_rfc3339()),
            termination_reason: session.termination_reason,
            metadata: session.metadata,
        })
    }
}

pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSessionsQuery>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<Json<Vec<SessionResponse>>> {
    use crate::server::rbac::AuthPrincipal;
    
    // Get username from auth context
    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };

    // If created_by is specified and doesn't match current user, check admin permission
    let filter_user = if let Some(ref requested_user) = query.created_by {
        if requested_user != username {
            // Check if user has admin permissions to view other users' sessions
            let is_admin = crate::server::auth::check_permission(
                &auth.principal,
                &state,
                &crate::server::rbac::PermissionContext::new("api", "sessions", "list-all"),
            )
            .await
            .unwrap_or(false);

            if !is_admin {
                return Err(ApiError::Forbidden("Cannot view other users' sessions".to_string()));
            }
        }
        Some(requested_user.as_str())
    } else {
        // Default to current user's sessions
        Some(username.as_str())
    };

    let mut sessions = Session::find_all(&state.db, query.workspace.as_deref(), filter_user)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list sessions: {}", e)))?;

    // Filter by state if provided
    if let Some(state_filter) = query.state {
        sessions.retain(|s| s.state == state_filter);
    }

    let mut response = Vec::new();
    for session in sessions {
        response.push(SessionResponse::from_session(session, &state.db).await?);
    }

    Ok(Json(response))
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<Json<SessionResponse>> {
    use crate::server::rbac::AuthPrincipal;
    
    let session_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid session ID format".to_string()))?;

    let session = Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch session: {}", e)))?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    // Check if user owns the session or is admin
    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };

    if &session.created_by != username {
        let is_admin = crate::server::auth::check_permission(
            &auth.principal,
            &state,
            &crate::server::rbac::PermissionContext::new("api", "sessions", "get-all"),
        )
        .await
        .unwrap_or(false);

        if !is_admin {
            return Err(ApiError::Forbidden("Cannot access other users' sessions".to_string()));
        }
    }

    Ok(Json(SessionResponse::from_session(session, &state.db).await?))
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateSessionRequest>,
) -> ApiResult<Json<SessionResponse>> {
    use crate::server::rbac::AuthPrincipal;
    
    tracing::info!("Creating session: {:?}", req);
    
    // Validate agent IDs exist
    for agent_id in &req.agent_ids {
        let agent_exists = sqlx::query(
            "SELECT id FROM agents WHERE id = $1 AND active = true"
        )
        .bind(agent_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to validate agent: {}", e)))?;

        if agent_exists.is_none() {
            return Err(ApiError::BadRequest(format!("Agent {} not found or inactive", agent_id)));
        }
    }

    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => s.name.clone(),
        AuthPrincipal::ServiceAccount(sa) => sa.user.clone(),
    };

    let session = Session::create(&state.db, req.clone(), username.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session: {:?}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create session: {}", e))
        })?;

    // Add task to queue for session manager to create container
    sqlx::query(
        r#"
        INSERT INTO session_tasks (session_id, task_type, payload, status)
        VALUES ($1, 'create_session', $2, 'pending')
        "#
    )
    .bind(session.id)
    .bind(serde_json::json!({
        "user_id": username,
        "agent_ids": req.agent_ids
    }))
    .execute(&*state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create session task: {}", e)))?;
    
    tracing::info!("Created session task for session {}", session.id);

    Ok(Json(SessionResponse::from_session(session, &state.db).await?))
}

pub async fn remix_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<RemixSessionRequest>,
) -> ApiResult<Json<SessionResponse>> {
    use crate::server::rbac::AuthPrincipal;
    
    let parent_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid session ID format".to_string()))?;

    // Check if parent session exists and user has access
    let parent = Session::find_by_id(&state.db, parent_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch parent session: {}", e)))?
        .ok_or(ApiError::NotFound("Parent session not found".to_string()))?;

    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };

    if &parent.created_by != username {
        let is_admin = crate::server::auth::check_permission(
            &auth.principal,
            &state,
            &crate::server::rbac::PermissionContext::new("api", "sessions", "remix-all"),
        )
        .await
        .unwrap_or(false);

        if !is_admin {
            return Err(ApiError::Forbidden("Cannot remix other users' sessions".to_string()));
        }
    }

    // Validate new agent IDs if provided
    if let Some(ref agent_ids) = req.agent_ids {
        for agent_id in agent_ids {
            let agent_exists = sqlx::query(
                "SELECT id FROM agents WHERE id = $1 AND active = true"
            )
            .bind(agent_id)
            .fetch_optional(&*state.db)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to validate agent: {}", e)))?;

            if agent_exists.is_none() {
                return Err(ApiError::BadRequest(format!("Agent {} not found or inactive", agent_id)));
            }
        }
    }

    let session = Session::remix(&state.db, parent_id, req, username.to_string())
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to remix session: {}", e)))?;

    Ok(Json(SessionResponse::from_session(session, &state.db).await?))
}

pub async fn update_session_state(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<UpdateSessionStateRequest>,
) -> ApiResult<Json<SessionResponse>> {
    use crate::server::rbac::AuthPrincipal;
    
    let session_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid session ID format".to_string()))?;

    // Check if session exists and user has access
    let session = Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch session: {}", e)))?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };

    // Check permission for updating sessions in the workspace
    let can_update = check_api_permission(&auth, &state, &permissions::SESSION_UPDATE, Some(&session.workspace))
        .await
        .is_ok();
    
    if !can_update && &session.created_by != username {
        return Err(ApiError::Forbidden("Cannot update other users' sessions".to_string()));
    }

    // Store old state for comparison
    let old_state = session.state;
    let new_state = req.state;
    
    let updated_session = Session::update_state(&state.db, session_id, req)
        .await
        .map_err(|e| {
            if e.to_string().contains("Invalid state transition") {
                ApiError::BadRequest(e.to_string())
            } else {
                ApiError::Internal(anyhow::anyhow!("Failed to update session state: {}", e))
            }
        })?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    // Add tasks for container state transitions
    match (old_state, new_state) {
        (SessionState::Init, SessionState::Ready) => {
            // Container should be created by this point
            tracing::debug!("Session {} transitioned to Ready", session_id);
        }
        (SessionState::Ready, SessionState::Idle) => {
            // Add task to stop container
            sqlx::query(
                r#"
                INSERT INTO session_tasks (session_id, task_type, payload, status)
                VALUES ($1, 'stop_session', '{}', 'pending')
                "#
            )
            .bind(session_id)
            .execute(&*state.db)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create stop task: {}", e)))?;
        }
        (SessionState::Idle, SessionState::Ready) => {
            // Add task to reactivate container
            sqlx::query(
                r#"
                INSERT INTO session_tasks (session_id, task_type, payload, status)
                VALUES ($1, 'reactivate_session', '{}', 'pending')
                "#
            )
            .bind(session_id)
            .execute(&*state.db)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create reactivate task: {}", e)))?;
        }
        _ => {
            tracing::debug!("Session {} state transition {:?} -> {:?}", session_id, old_state, new_state);
        }
    }

    Ok(Json(SessionResponse::from_session(updated_session, &state.db).await?))
}

pub async fn update_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<UpdateSessionRequest>,
) -> ApiResult<Json<SessionResponse>> {
    use crate::server::rbac::AuthPrincipal;
    
    let session_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid session ID format".to_string()))?;

    // Check if session exists and user has access
    let session = Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch session: {}", e)))?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };

    // Check permission for updating sessions in the workspace
    let can_update = check_api_permission(&auth, &state, &permissions::SESSION_UPDATE, Some(&session.workspace))
        .await
        .is_ok();
    
    if !can_update && &session.created_by != username {
        return Err(ApiError::Forbidden("Cannot update other users' sessions".to_string()));
    }

    let updated_session = Session::update(&state.db, session_id, req)
        .await
        .map_err(|e| {
            if e.to_string().contains("No fields to update") {
                ApiError::BadRequest(e.to_string())
            } else {
                ApiError::Internal(anyhow::anyhow!("Failed to update session: {}", e))
            }
        })?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    Ok(Json(SessionResponse::from_session(updated_session, &state.db).await?))
}

pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<()> {
    use crate::server::rbac::AuthPrincipal;
    
    let session_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid session ID format".to_string()))?;

    // Check if session exists and user has access
    let session = Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch session: {}", e)))?
        .ok_or(ApiError::NotFound("Session not found".to_string()))?;

    let username = match &auth.principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };

    // Check permission for deleting sessions in the workspace
    let can_delete = check_api_permission(&auth, &state, &permissions::SESSION_DELETE, Some(&session.workspace))
        .await
        .is_ok();
    
    if !can_delete && &session.created_by != username {
        return Err(ApiError::Forbidden("Cannot delete other users' sessions".to_string()));
    }

    // Sessions can be soft deleted in any state

    // Add task to queue for session manager to destroy container
    sqlx::query(
        r#"
        INSERT INTO session_tasks (session_id, task_type, payload, status)
        VALUES ($1, 'destroy_session', '{}', 'pending')
        "#
    )
    .bind(session_id)
    .execute(&*state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create destroy task: {}", e)))?;
    
    tracing::info!("Created destroy task for session {}", session_id);

    let deleted = Session::delete(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete session: {}", e)))?;

    if !deleted {
        return Err(ApiError::NotFound("Session not found".to_string()));
    }

    Ok(())
}