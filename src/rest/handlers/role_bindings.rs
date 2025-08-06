use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::models::AppState;
use crate::rbac::{RoleBinding, SubjectType};
use crate::rest::error::{ApiError, ApiResult};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleBindingRequest {
    pub role_name: String,
    pub principal_name: String,
    pub principal_type: SubjectType,
    #[serde(default)]
    pub namespace: Option<String>, // NULL = global, String = specific organization
}


#[derive(Debug, Serialize, ToSchema)]
pub struct RoleBindingResponse {
    pub id: String,
    pub role_name: String,
    pub principal_name: String,
    pub principal_type: SubjectType,
    pub namespace: Option<String>, // NULL = global access, String = specific organization
    pub created_at: String,
}


impl From<RoleBinding> for RoleBindingResponse {
    fn from(rb: RoleBinding) -> Self {
        Self {
            id: rb.id.map(|id| id.to_string()).unwrap_or_default(),
            role_name: rb.role_name,
            principal_name: rb.principal_name,
            principal_type: rb.principal_type,
            namespace: rb.namespace,
            created_at: rb.created_at,
        }
    }
}

pub async fn list_role_bindings(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<RoleBindingResponse>>> {
    let bindings = state.get_all_role_bindings().await?;
    let response: Vec<RoleBindingResponse> = bindings.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_role_binding(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<RoleBindingResponse>> {
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoleBindingRequest>,
) -> ApiResult<Json<RoleBindingResponse>> {
    // Check if already exists
    if let Ok(Some(_)) = state.get_role_binding(&req.name, req.namespace.as_deref()).await {
        return Err(ApiError::Conflict("Role binding already exists".to_string()));
    }
    
    let role_binding = RoleBinding {
        id: None,
        name: req.name,
        namespace: req.namespace,
        role_ref: RoleRef {
            kind: req.role_ref.kind,
            name: req.role_ref.name,
            api_group: req.role_ref.api_group,
        },
        subjects: req.subjects.into_iter().map(|s| RoleBindingSubject {
            kind: s.kind,
            name: s.name,
            namespace: s.namespace,
        }).collect(),
        created_at: Utc::now().to_rfc3339(),
    };
    
    let created_binding = state.create_role_binding(&role_binding).await?;
    Ok(Json(created_binding.into()))
}

pub async fn delete_role_binding(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    let deleted = if uuid::Uuid::parse_str(&id).is_ok() {
        // For UUID, we need to find the binding first
        if let Some(binding) = state.get_all_role_bindings().await?
            .into_iter()
            .find(|rb| rb.id == Some(uuid::Uuid::parse_str(&id).unwrap())) {
            state.delete_role_binding(&binding.name, binding.namespace.as_deref()).await?
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