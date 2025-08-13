use axum::{
    extract::{Path, Query, State},
    Extension,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::{Agent, AppState, CreateAgentRequest, UpdateAgentRequest};
use crate::rest::error::{ApiError, ApiResult};
use crate::rest::middleware::AuthContext;
use crate::rest::rbac_enforcement::{check_api_permission, permissions, get_user_workspace};

#[derive(Debug, Serialize, ToSchema)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub workspace: String,
    pub description: Option<String>,
    pub instructions: String,
    pub model: String,
    pub tools: serde_json::Value,
    pub routes: serde_json::Value,
    pub guardrails: serde_json::Value,
    pub knowledge_bases: serde_json::Value,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Agent> for AgentResponse {
    fn from(agent: Agent) -> Self {
        Self {
            id: agent.id.to_string(),
            name: agent.name,
            workspace: agent.workspace,
            description: agent.description,
            instructions: agent.instructions,
            model: agent.model,
            tools: agent.tools,
            routes: agent.routes,
            guardrails: agent.guardrails,
            knowledge_bases: agent.knowledge_bases,
            active: agent.active,
            created_at: agent.created_at.to_rfc3339(),
            updated_at: agent.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ListAgentsQuery {
    pub workspace: Option<String>,
}

pub async fn list_agents(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAgentsQuery>,
) -> ApiResult<Json<Vec<AgentResponse>>> {
    // Determine target workspace
    let user_workspace = get_user_workspace(&auth);
    let target_workspace = query.workspace.as_deref()
        .or(user_workspace.as_deref())
        .unwrap_or("default");

    // Check permission for listing agents in the workspace
    check_api_permission(&auth, &state, &permissions::AGENT_LIST, Some(target_workspace))
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    let agents = Agent::find_all(&state.db, Some(target_workspace))
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list agents: {}", e)))?;
    
    let response: Vec<AgentResponse> = agents.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_agent(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<AgentResponse>> {
    // Try parsing as UUID first
    let agent = if let Ok(uuid) = Uuid::parse_str(&id) {
        Agent::find_by_id(&state.db, uuid).await
    } else {
        // For name lookups, use user's workspace from context
        let workspace = get_user_workspace(&auth).unwrap_or_else(|| "default".to_string());
        Agent::find_by_name(&state.db, &id, &workspace).await
    }
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch agent: {}", e)))?
    .ok_or(ApiError::NotFound("Agent not found".to_string()))?;

    // Check permission for the agent's workspace
    check_api_permission(&auth, &state, &permissions::AGENT_GET, Some(&agent.workspace))
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    Ok(Json(agent.into()))
}

pub async fn create_agent(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Json(mut req): Json<CreateAgentRequest>,
) -> ApiResult<Json<AgentResponse>> {
    // Use user's workspace if not specified
    if req.workspace.is_empty() {
        req.workspace = get_user_workspace(&auth).unwrap_or_else(|| "default".to_string());
    }

    // Check permission for creating agents in the workspace
    check_api_permission(&auth, &state, &permissions::AGENT_CREATE, Some(&req.workspace))
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    // Check if agent with same name already exists in the workspace
    if let Ok(Some(_)) = Agent::find_by_name(&state.db, &req.name, &req.workspace).await {
        return Err(ApiError::Conflict(format!("Agent '{}' already exists in workspace '{}'", req.name, req.workspace)));
    }

    let agent = Agent::create(&state.db, req)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create agent: {}", e)))?;

    Ok(Json(agent.into()))
}

pub async fn update_agent(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> ApiResult<Json<AgentResponse>> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid agent ID format".to_string()))?;

    // Get the existing agent first to get its workspace
    let existing_agent = Agent::find_by_id(&state.db, uuid)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch agent: {}", e)))?
        .ok_or(ApiError::NotFound("Agent not found".to_string()))?;

    // Check permission for updating agents in the workspace
    check_api_permission(&auth, &state, &permissions::AGENT_UPDATE, Some(&existing_agent.workspace))
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    
    // If updating name, check if new name already exists in the same workspace
    if let Some(ref new_name) = req.name {
        if let Ok(Some(existing)) = Agent::find_by_name(&state.db, new_name, &existing_agent.workspace).await {
            if existing.id != uuid {
                return Err(ApiError::Conflict(format!("Agent '{}' already exists in workspace '{}'", new_name, existing_agent.workspace)));
            }
        }
    }

    let agent = Agent::update(&state.db, uuid, req)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update agent: {}", e)))?
        .ok_or(ApiError::NotFound("Agent not found".to_string()))?;

    Ok(Json(agent.into()))
}

pub async fn delete_agent(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid agent ID format".to_string()))?;

    // Get the agent to check its workspace
    let agent = Agent::find_by_id(&state.db, uuid)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch agent: {}", e)))?
        .ok_or(ApiError::NotFound("Agent not found".to_string()))?;

    // Check permission for deleting agents in the workspace
    check_api_permission(&auth, &state, &permissions::AGENT_DELETE, Some(&agent.workspace))
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    let deleted = Agent::delete(&state.db, uuid)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete agent: {}", e)))?;

    if !deleted {
        return Err(ApiError::NotFound("Agent not found".to_string()));
    }

    Ok(())
}