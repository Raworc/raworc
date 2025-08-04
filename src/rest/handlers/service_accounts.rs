use axum::{
    extract::{Path, State},
    Json,
};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::models::AppState;
use crate::rbac::ServiceAccount;
use crate::rest::error::{ApiError, ApiResult};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateServiceAccountRequest {
    pub user: String,
    pub pass: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceAccountResponse {
    pub id: String,
    pub user: String,
    pub namespace: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: String,
}

impl From<ServiceAccount> for ServiceAccountResponse {
    fn from(sa: ServiceAccount) -> Self {
        Self {
            id: sa.id.map(|id| id.to_string()).unwrap_or_default(),
            user: sa.user,
            namespace: sa.namespace,
            description: sa.description,
            active: sa.active,
            created_at: sa.created_at,
        }
    }
}

pub async fn list_service_accounts(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ServiceAccountResponse>>> {
    let accounts = state.get_all_service_accounts().await?;
    let response: Vec<ServiceAccountResponse> = accounts.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_service_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<ServiceAccountResponse>> {
    // Try to parse as UUID first, otherwise treat as username
    let account = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        state.get_all_service_accounts().await?
            .into_iter()
            .find(|sa| sa.id == Some(uuid))
    } else {
        state.get_service_account(&id, None).await?
    };
    
    let account = account.ok_or(ApiError::NotFound("Service account not found".to_string()))?;
    Ok(Json(account.into()))
}

pub async fn create_service_account(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateServiceAccountRequest>,
) -> ApiResult<Json<ServiceAccountResponse>> {
    // Check if already exists
    if let Ok(Some(_)) = state.get_service_account(&req.user, req.namespace.as_deref()).await {
        return Err(ApiError::Conflict("Service account already exists".to_string()));
    }
    
    let pass_hash = hash(&req.pass, DEFAULT_COST)?;
    let account = state.create_service_account(
        &req.user,
        req.namespace,
        &pass_hash,
        req.description,
    ).await?;
    
    Ok(Json(account.into()))
}

pub async fn delete_service_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    let deleted = if uuid::Uuid::parse_str(&id).is_ok() {
        state.delete_service_account_by_id(&id).await?
    } else {
        state.delete_service_account(&id, None).await?
    };
    
    if !deleted {
        return Err(ApiError::NotFound("Service account not found".to_string()));
    }
    
    Ok(())
}