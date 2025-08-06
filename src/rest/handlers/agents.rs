use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::{Agent, AppState, CreateAgentRequest, UpdateAgentRequest};
use crate::rest::error::{ApiError, ApiResult};

#[derive(Debug, Serialize, ToSchema)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub namespace: String,
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
            namespace: agent.namespace,
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
    pub namespace: Option<String>,
}

pub async fn list_agents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAgentsQuery>,
) -> ApiResult<Json<Vec<AgentResponse>>> {
    let agents = Agent::find_all(&state.db, query.namespace.as_deref())
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list agents: {}", e)))?;
    
    let response: Vec<AgentResponse> = agents.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<AgentResponse>> {
    // Try parsing as UUID first
    let agent = if let Ok(uuid) = Uuid::parse_str(&id) {
        Agent::find_by_id(&state.db, uuid).await
    } else {
        // For name lookups, we need a namespace - default to "default"
        // In a real implementation, you'd get this from the auth context
        Agent::find_by_name(&state.db, &id, "default").await
    }
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch agent: {}", e)))?
    .ok_or(ApiError::NotFound("Agent not found".to_string()))?;

    Ok(Json(agent.into()))
}

pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAgentRequest>,
) -> ApiResult<Json<AgentResponse>> {
    // Check if agent with same name already exists in the namespace
    if let Ok(Some(_)) = Agent::find_by_name(&state.db, &req.name, &req.namespace).await {
        return Err(ApiError::Conflict(format!("Agent '{}' already exists in namespace '{}'", req.name, req.namespace)));
    }

    let agent = Agent::create(&state.db, req)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create agent: {}", e)))?;

    Ok(Json(agent.into()))
}

pub async fn update_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> ApiResult<Json<AgentResponse>> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid agent ID format".to_string()))?;

    // Get the existing agent first to get its namespace
    let existing_agent = Agent::find_by_id(&state.db, uuid)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to fetch agent: {}", e)))?
        .ok_or(ApiError::NotFound("Agent not found".to_string()))?;
    
    // If updating name, check if new name already exists in the same namespace
    if let Some(ref new_name) = req.name {
        if let Ok(Some(existing)) = Agent::find_by_name(&state.db, new_name, &existing_agent.namespace).await {
            if existing.id != uuid {
                return Err(ApiError::Conflict(format!("Agent '{}' already exists in namespace '{}'", new_name, existing_agent.namespace)));
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid agent ID format".to_string()))?;

    let deleted = Agent::delete(&state.db, uuid)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete agent: {}", e)))?;

    if !deleted {
        return Err(ApiError::NotFound("Agent not found".to_string()));
    }

    Ok(())
}