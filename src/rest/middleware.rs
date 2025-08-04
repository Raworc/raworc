use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::auth::decode_jwt;
#[allow(unused_imports)]
use crate::auth::check_permission;
use crate::models::AppState;
use crate::rbac::{AuthPrincipal, RbacClaims, Subject, SubjectType};
#[allow(unused_imports)]
use crate::rbac::PermissionContext;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AuthContext {
    pub principal: AuthPrincipal,
    #[allow(dead_code)]
    pub claims: RbacClaims,
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for public endpoints
    let path = request.uri().path();
    if path == "/api/v1/health" || 
       path == "/api/v1/version" || 
       path.starts_with("/api/v1/auth/") ||
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
                .get_service_account(&claims.sub, claims.namespace.as_deref())
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
        namespace = ?claims.namespace,
        "API request"
    );

    Ok(next.run(request).await)
}

#[allow(dead_code)]
pub async fn require_permission(
    api_group: &str,
    resource: &str,
    verb: &str,
    auth_context: &AuthContext,
    state: &AppState,
) -> Result<(), StatusCode> {
    let context = PermissionContext::new(api_group, resource, verb);
    
    let has_permission = check_permission(&auth_context.principal, state, &context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}