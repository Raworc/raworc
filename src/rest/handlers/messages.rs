use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use sqlx;

use crate::models::{
    AppState, SessionMessage, CreateMessageRequest, MessageResponse, ListMessagesQuery
};
use crate::rest::error::{ApiError, ApiResult};
use crate::rest::middleware::AuthContext;

pub async fn create_message(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<CreateMessageRequest>,
) -> ApiResult<Json<MessageResponse>> {
    // Validate that agent_id is provided when role is AGENT
    if req.role == crate::models::MessageRole::Agent && req.agent_id.is_none() {
        return Err(ApiError::BadRequest("agent_id is required when role is AGENT".to_string()));
    }
    
    // Verify session exists and user has access
    let session = crate::models::Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Session not found".to_string()))?;
    
    // Check if session is idle and needs reactivation
    if session.state == crate::models::SessionState::Idle {
        tracing::info!("Reactivating idle session {} due to new message", session_id);
        
        // Reactivate the session (this will trigger container restart)
        if let Some(docker) = &state.docker {
            // Reactivate session (transitions from IDLE to READY and restarts container)
            if let Err(e) = docker.reactivate_session(session_id).await {
                tracing::error!("Failed to reactivate session {}: {}", session_id, e);
                return Err(ApiError::Internal(anyhow::anyhow!("Failed to reactivate session: {}", e)));
            }
            
            // Now transition to BUSY for message processing
            sqlx::query(
                "UPDATE sessions SET state = 'BUSY', last_activity_at = CURRENT_TIMESTAMP WHERE id = $1 AND state = 'READY'"
            )
            .bind(session_id)
            .execute(&*state.db)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session state: {}", e)))?;
        }
    } else if session.state == crate::models::SessionState::Ready {
        // Update session to BUSY when processing a message
        sqlx::query(
            "UPDATE sessions SET state = 'BUSY', last_activity_at = CURRENT_TIMESTAMP WHERE id = $1 AND state = 'READY'"
        )
        .bind(session_id)
        .execute(&*state.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session state: {}", e)))?;
    }
    
    // Create the message
    let message = SessionMessage::create(&state.db, session_id, req)
        .await
        .map_err(|e| {
            eprintln!("Database error creating message: {:?}", e);
            ApiError::Internal(anyhow::anyhow!("Failed to create message: {}", e))
        })?;
    
    // Get agent name if applicable
    let agent_name = if let Some(agent_id) = message.agent_id {
        crate::models::Agent::find_by_id(&state.db, agent_id)
            .await
            .ok()
            .flatten()
            .map(|a| a.name)
    } else {
        None
    };
    
    Ok(Json(MessageResponse {
        id: message.id.to_string(),
        session_id: message.session_id.to_string(),
        role: message.role,
        content: message.content,
        agent_id: message.agent_id.map(|id| id.to_string()),
        agent_name,
        metadata: message.metadata,
        created_at: message.created_at.to_rfc3339(),
    }))
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Query(query): Query<ListMessagesQuery>,
    Extension(_auth): Extension<AuthContext>,
) -> ApiResult<Json<Vec<MessageResponse>>> {
    // Verify session exists
    let _session = crate::models::Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Session not found".to_string()))?;
    
    // Get messages with agent info
    let messages = SessionMessage::get_with_agent_info(
        &state.db, 
        session_id, 
        query.limit, 
        query.offset
    )
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch messages: {}", e)))?;
    
    Ok(Json(messages))
}

pub async fn get_message_count(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Extension(_auth): Extension<AuthContext>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify session exists
    let _session = crate::models::Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Session not found".to_string()))?;
    
    let count = SessionMessage::count_by_session(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to count messages: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "count": count,
        "session_id": session_id.to_string()
    })))
}

pub async fn clear_messages(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Extension(_auth): Extension<AuthContext>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify session exists
    let _session = crate::models::Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Session not found".to_string()))?;
    
    let deleted_count = SessionMessage::delete_by_session(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete messages: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "deleted": deleted_count,
        "session_id": session_id.to_string()
    })))
}