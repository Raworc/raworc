use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::models::AppState;
use crate::rbac::{RoleBinding, RoleBindingSubject, RoleRef, SubjectType};
use crate::rest::error::{ApiError, ApiResult};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleBindingRequest {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    pub role_ref: RoleRefRequest,
    pub subjects: Vec<SubjectRequest>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleRefRequest {
    pub kind: String,
    pub name: String,
    pub api_group: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubjectRequest {
    pub kind: SubjectType,
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleBindingResponse {
    pub id: String,
    pub name: String,
    pub namespace: Option<String>,
    pub role_ref: RoleRefResponse,
    pub subjects: Vec<SubjectResponse>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleRefResponse {
    pub kind: String,
    pub name: String,
    pub api_group: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubjectResponse {
    pub kind: SubjectType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

impl From<RoleBinding> for RoleBindingResponse {
    fn from(rb: RoleBinding) -> Self {
        Self {
            id: rb.id.map(|id| id.to_string()).unwrap_or_default(),
            name: rb.name,
            namespace: rb.namespace,
            role_ref: RoleRefResponse {
                kind: rb.role_ref.kind,
                name: rb.role_ref.name,
                api_group: rb.role_ref.api_group,
            },
            subjects: rb.subjects.into_iter().map(|s| SubjectResponse {
                kind: s.kind,
                name: s.name,
                namespace: s.namespace,
            }).collect(),
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