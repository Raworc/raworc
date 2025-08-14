use axum::{
    extract::{Path, State},
    Extension,
    Json,
};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::shared::models::AppState;
use crate::server::rbac::ServiceAccount;
use crate::server::rest::error::{ApiError, ApiResult};
use crate::server::rest::middleware::AuthContext;
use crate::server::rest::rbac_enforcement::{check_api_permission, permissions};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateServiceAccountRequest {
    pub user: String,
    pub pass: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateServiceAccountRequest {
    pub workspace: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceAccountResponse {
    pub id: String,
    pub user: String,
    pub workspace: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
}

impl From<ServiceAccount> for ServiceAccountResponse {
    fn from(sa: ServiceAccount) -> Self {
        Self {
            id: sa.id.map(|id| id.to_string()).unwrap_or_default(),
            user: sa.user,
            workspace: None, // Service accounts are global now
            description: sa.description,
            active: sa.active,
            created_at: sa.created_at,
            updated_at: sa.updated_at,
            last_login_at: sa.last_login_at,
        }
    }
}

pub async fn list_service_accounts(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ServiceAccountResponse>>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::SERVICE_ACCOUNT_LIST, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;

    let accounts = state.get_all_service_accounts().await?;
    let response: Vec<ServiceAccountResponse> = accounts.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn get_service_account(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<ServiceAccountResponse>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::SERVICE_ACCOUNT_GET, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    // Try to parse as UUID first, otherwise treat as username
    let account = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        state.get_all_service_accounts().await?
            .into_iter()
            .find(|sa| sa.id == Some(uuid))
    } else {
        state.get_service_account(&id).await?
    };
    
    let account = account.ok_or(ApiError::NotFound("Service account not found".to_string()))?;
    Ok(Json(account.into()))
}

pub async fn create_service_account(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateServiceAccountRequest>,
) -> ApiResult<Json<ServiceAccountResponse>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::SERVICE_ACCOUNT_CREATE, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    // Check if already exists
    if let Ok(Some(_)) = state.get_service_account(&req.user).await {
        return Err(ApiError::Conflict("Service account already exists".to_string()));
    }
    
    let pass_hash = hash(&req.pass, DEFAULT_COST)?;
    let account = state.create_service_account(
        &req.user,
        None, // Service accounts are global now
        &pass_hash,
        req.description,
    ).await?;
    
    Ok(Json(account.into()))
}

pub async fn delete_service_account(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::SERVICE_ACCOUNT_DELETE, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    let deleted = if uuid::Uuid::parse_str(&id).is_ok() {
        state.delete_service_account_by_id(&id).await?
    } else {
        state.delete_service_account(&id).await?
    };
    
    if !deleted {
        return Err(ApiError::NotFound("Service account not found".to_string()));
    }
    
    Ok(())
}

pub async fn update_service_account_password(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePasswordRequest>,
) -> ApiResult<()> {
    // Check permission - users can update their own password, admins can update any
    let is_self = match &auth.principal {
        crate::server::rbac::AuthPrincipal::ServiceAccount(sa) => sa.user == id || sa.id.map(|uuid| uuid.to_string()) == Some(id.clone()),
        _ => false,
    };

    if !is_self {
        check_api_permission(&auth, &state, &permissions::SERVICE_ACCOUNT_UPDATE, None)
            .await
            .map_err(|e| match e {
                axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
                _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
            })?;
    }
    use bcrypt::verify;
    
    // Get the service account first
    let account = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        state.get_all_service_accounts().await?
            .into_iter()
            .find(|sa| sa.id == Some(uuid))
    } else {
        state.get_service_account(&id).await?
    };
    
    let account = account.ok_or(ApiError::NotFound("Service account not found".to_string()))?;
    
    // Verify current password
    if !verify(&req.current_password, &account.pass_hash)? {
        return Err(ApiError::Unauthorized);
    }
    
    // Hash new password
    let new_pass_hash = hash(&req.new_password, DEFAULT_COST)?;
    
    // Update password
    let updated = if let Some(id) = account.id {
        state.update_service_account_password_by_id(&id.to_string(), &new_pass_hash).await?
    } else {
        state.update_service_account_password(&account.user, &new_pass_hash).await?
    };
    
    if !updated {
        return Err(ApiError::NotFound("Service account not found".to_string()));
    }
    
    Ok(())
}

pub async fn update_service_account(
    Extension(auth): Extension<AuthContext>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateServiceAccountRequest>,
) -> ApiResult<Json<ServiceAccountResponse>> {
    // Check permission
    check_api_permission(&auth, &state, &permissions::SERVICE_ACCOUNT_UPDATE, None)
        .await
        .map_err(|e| match e {
            axum::http::StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
            _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
        })?;
    // Check if service account exists
    let account = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        state.get_all_service_accounts().await?
            .into_iter()
            .find(|sa| sa.id == Some(uuid))
    } else {
        state.get_service_account(&id).await?
    };
    
    let account = account.ok_or(ApiError::NotFound("Service account not found".to_string()))?;
    
    // Update the service account
    let updated = state.update_service_account(
        &account.id.unwrap().to_string(),
        req.workspace,
        req.description,
        req.active,
    ).await?;
    
    if !updated {
        return Err(ApiError::NotFound("Service account not found".to_string()));
    }
    
    // Fetch the updated account
    let updated_account = state.get_all_service_accounts().await?
        .into_iter()
        .find(|sa| sa.id == account.id)
        .ok_or(ApiError::NotFound("Service account not found".to_string()))?;
    
    Ok(Json(updated_account.into()))
}