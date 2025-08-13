use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AppState, SessionState};
use crate::rest::error::{ApiError, ApiResult};
use crate::rest::middleware::AuthContext;

/// Mark a session as complete (transition from BUSY to READY)
pub async fn complete_session_processing(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Extension(_auth): Extension<AuthContext>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify session exists
    let session = crate::models::Session::find_by_id(&state.db, session_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Session not found".to_string()))?;
    
    // Check if session is in BUSY state
    if session.state != SessionState::Busy {
        return Err(ApiError::BadRequest(
            format!("Session is not in BUSY state, current state: {:?}", session.state)
        ));
    }
    
    // Update session state from BUSY to READY
    sqlx::query(
        r#"
        UPDATE sessions 
        SET state = 'READY', 
            last_activity_at = CURRENT_TIMESTAMP 
        WHERE id = $1 AND state = 'BUSY'
        "#
    )
    .bind(session_id)
    .execute(&*state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session state: {}", e)))?;
    
    // Sync container state if Docker is enabled
    if let Some(docker) = &state.docker {
        // Docker lifecycle manager handles state transitions automatically
        tracing::debug!("Docker container state will be synchronized for session {}", session_id);
    }
    
    tracing::info!("Session {} processing complete, returned to READY", session_id);
    
    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "state": "READY",
        "message": "Session processing complete"
    })))
}