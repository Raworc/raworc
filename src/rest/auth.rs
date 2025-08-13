use axum::{
    extract::{Extension, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::auth::{authenticate_service_account, create_service_account_jwt, create_subject_jwt};
use crate::models::AppState;
use crate::rbac::TokenResponse;
use crate::rest::error::{ApiError, ApiResult};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub user: String,
    pub pass: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub workspace: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExternalLoginRequest {
    pub subject: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_at: String,
}

impl From<TokenResponse> for LoginResponse {
    fn from(token: TokenResponse) -> Self {
        Self {
            token: token.token,
            token_type: "Bearer".to_string(),
            expires_at: token.expires_at,
        }
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let service_account = authenticate_service_account(
        &state,
        &req.user,
        &req.pass,
    )
    .await?
    .ok_or(ApiError::Unauthorized)?;

    // Update last login timestamp
    let _ = state.update_last_login(&req.user).await;

    let token_response = create_service_account_jwt(&service_account, &state.jwt_secret, 24)?;
    
    Ok(Json(token_response.into()))
}

pub async fn external_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExternalLoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    // This endpoint requires admin authentication - checked by middleware
    let token_response = create_subject_jwt(&req.subject, &state.jwt_secret, 24)?;
    
    Ok(Json(token_response.into()))
}

pub async fn me(
    Extension(auth): Extension<crate::rest::middleware::AuthContext>,
) -> ApiResult<Json<serde_json::Value>> {
    use crate::rbac::AuthPrincipal;
    
    let (user, namespace, principal_type) = match &auth.principal {
        AuthPrincipal::Subject(s) => (&s.name, None::<String>, "Subject"),
        AuthPrincipal::ServiceAccount(sa) => (&sa.user, None::<String>, "ServiceAccount"),
    };
    
    Ok(Json(serde_json::json!({
        "user": user,
        "namespace": namespace,
        "type": principal_type
    })))
}