use axum::{
    extract::{Path, State},
    Extension,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::models::AppState;
use crate::rbac::{Role, Rule};
use crate::rest::error::{ApiError, ApiResult};
use crate::rest::middleware::AuthContext;
use crate::rest::rbac_enforcement::{check_api_permission, permissions};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    // #[serde(default)]
    // pub workspace: Option<String>, // Roles are global now
    pub rules: Vec<RuleRequest>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RuleRequest {
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub verbs: Vec<String>,
    #[serde(default)]
    pub resource_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    // pub workspace: Option<String>, // Roles are global now
    pub rules: Vec<RuleResponse>,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RuleResponse {
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub verbs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_names: Option<Vec<String>>,
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.map(|id| id.to_string()).unwrap_or_default(),
            name: role.name,
            // workspace: None, // Roles are global now - field removed from struct
            rules: role.rules.into_iter().map(|r| RuleResponse {
                api_groups: r.api_groups,
                resources: r.resources,
                verbs: r.verbs,
                resource_names: r.resource_names,
            }).collect(),
            description: role.description,
            created_at: role.created_at,
        }
    }
}

pub async fn list_roles(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<RoleResponse>>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_LIST, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    let roles = state.get_all_roles().await?;
    let response: Vec<RoleResponse> = roles.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_role(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<RoleResponse>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_GET, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    // Try to parse as UUID first, otherwise treat as name
    let role = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        state.get_all_roles().await?
            .into_iter()
            .find(|r| r.id == Some(uuid))
    } else {
        state.get_role(&id).await?
    };
    
    let role = role.ok_or(ApiError::NotFound("Role not found".to_string()))?;
    Ok(Json(role.into()))
}

pub async fn create_role(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoleRequest>,
) -> ApiResult<Json<RoleResponse>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_CREATE, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    // Check if already exists
    if let Ok(Some(_)) = state.get_role(&req.name).await {
        return Err(ApiError::Conflict("Role already exists".to_string()));
    }
    
    let role = Role {
        id: None,
        name: req.name,
        // workspace: req.workspace, // Roles are global now
        rules: req.rules.into_iter().map(|r| Rule {
            api_groups: r.api_groups,
            resources: r.resources,
            verbs: r.verbs,
            resource_names: r.resource_names,
        }).collect(),
        description: req.description,
        created_at: Utc::now().to_rfc3339(),
    };
    
    let created_role = state.create_role(&role).await?;
    Ok(Json(created_role.into()))
}

pub async fn delete_role(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_DELETE, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    let deleted = if uuid::Uuid::parse_str(&id).is_ok() {
        // For UUID, we need to find the role first
        if let Some(role) = state.get_all_roles().await?
            .into_iter()
            .find(|r| r.id == Some(uuid::Uuid::parse_str(&id).unwrap())) {
            state.delete_role(&role.name).await?
        } else {
            false
        }
    } else {
        state.delete_role(&id).await?
    };
    
    if !deleted {
        return Err(ApiError::NotFound("Role not found".to_string()));
    }
    
    Ok(())
}