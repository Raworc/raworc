use axum::{
    extract::{Path, State},
    Extension,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::shared::models::AppState;
use crate::server::rbac::{RoleBinding, SubjectType};
use crate::server::rest::error::{ApiError, ApiResult};
use crate::server::rest::middleware::AuthContext;
use crate::server::rest::rbac_enforcement::{check_api_permission, permissions};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleBindingRequest {
    pub role_name: String,
    pub principal_name: String,
    pub principal_type: SubjectType,
    #[serde(default)]
    pub workspace: Option<String>, // NULL = global, String = specific organization
}


#[derive(Debug, Serialize, ToSchema)]
pub struct RoleBindingResponse {
    pub id: String,
    pub role_name: String,
    pub principal_name: String,
    pub principal_type: SubjectType,
    pub workspace: Option<String>, // NULL = global access, String = specific organization
    pub created_at: String,
}


impl From<RoleBinding> for RoleBindingResponse {
    fn from(rb: RoleBinding) -> Self {
        Self {
            id: rb.id.map(|id| id.to_string()).unwrap_or_default(),
            role_name: rb.role_name,
            principal_name: rb.principal_name,
            principal_type: rb.principal_type,
            workspace: rb.workspace,
            created_at: rb.created_at,
        }
    }
}

pub async fn list_role_bindings(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<RoleBindingResponse>>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_BINDING_LIST, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    let bindings = state.get_all_role_bindings().await?;
    let response: Vec<RoleBindingResponse> = bindings.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_role_binding(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<RoleBindingResponse>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_BINDING_GET, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    // Try to parse as UUID first, otherwise treat as name
    let binding = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        state.get_all_role_bindings().await?
            .into_iter()
            .find(|rb| rb.id == Some(uuid))
    } else {
        state.get_role_binding(&id, None).await?
    };
    
    let binding = binding.ok_or(ApiError::NotFound("Role binding not found".to_string()))?;
    Ok(Json(binding.into()))
}

pub async fn create_role_binding(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoleBindingRequest>,
) -> ApiResult<Json<RoleBindingResponse>> {
    // Check permission - need extra permissions for global bindings
    let target_workspace = req.workspace.as_deref();
    check_api_permission(&auth, &state, &permissions::ROLE_BINDING_CREATE, target_workspace)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    let role_binding = RoleBinding {
        id: None,
        role_name: req.role_name,
        principal_name: req.principal_name,
        principal_type: req.principal_type,
        workspace: req.workspace,
        created_at: Utc::now().to_rfc3339(),
    };
    
    let created_binding = state.create_role_binding(&role_binding).await?;
    Ok(Json(created_binding.into()))
}

pub async fn delete_role_binding(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::ROLE_BINDING_DELETE, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    let deleted = if uuid::Uuid::parse_str(&id).is_ok() {
        // For UUID, we need to find the binding first
        if let Some(binding) = state.get_all_role_bindings().await?
            .into_iter()
            .find(|rb| rb.id == Some(uuid::Uuid::parse_str(&id).unwrap())) {
            state.delete_role_binding(&binding.role_name, binding.workspace.as_deref()).await?
        } else {
            false
        }
    } else {
        state.delete_role_binding(&id, None).await?
    };
    
    if !deleted {
        return Err(ApiError::NotFound("Role binding not found".to_string()));
    }
    
    Ok(())
}