use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::server::auth::decode_jwt;
use crate::shared::models::AppState;
use crate::server::rbac::{AuthPrincipal, RbacClaims, Subject, SubjectType};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AuthContext {
    pub principal: AuthPrincipal,
    pub claims: RbacClaims,
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for public endpoints
    let path = request.uri().path();
    if path == "/api/v0/health" || 
       path == "/api/v0/version" || 
       path.starts_with("/api/v0/auth/") ||
       path.starts_with("/swagger-ui") ||
       path == "/api-docs/openapi.json" {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Decode and validate JWT
    let claims = decode_jwt(token, &state.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Get principal from claims
    let principal = match claims.sub_type {
        SubjectType::ServiceAccount => {
            let service_account = state
                .get_service_account(&claims.sub)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::UNAUTHORIZED)?;
            AuthPrincipal::ServiceAccount(service_account)
        }
        SubjectType::Subject => AuthPrincipal::Subject(Subject {
            name: claims.sub.clone(),
        }),
    };

    // Store auth context in request extensions
    let auth_context = AuthContext {
        principal: principal.clone(),
        claims: claims.clone(),
    };
    request.extensions_mut().insert(auth_context);

    // Log the authenticated API request
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user = match &principal {
        AuthPrincipal::Subject(s) => &s.name,
        AuthPrincipal::ServiceAccount(sa) => &sa.user,
    };
    
    info!(
        method = %method,
        path = %uri.path(),
        user = %user,
        workspace = ?claims.workspace,
        "API request"
    );

    Ok(next.run(request).await)
}