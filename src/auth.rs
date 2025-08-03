use crate::models::{AppState, DatabaseError};
use crate::rbac::{
    AuthPrincipal, PermissionContext, RbacAuthz, RbacClaims, ServiceAccount, Subject, SubjectType,
    TokenResponse,
};
use anyhow::Result;
use async_graphql::{Context, ErrorExtensions, Guard};
use axum::http::HeaderMap;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

// RBAC Auth guard for GraphQL fields
pub struct RbacGuard {
    pub api_group: String,
    pub resource: String,
    pub verb: String,
    pub resource_name: Option<String>,
}

impl RbacGuard {
    pub fn new(api_group: &str, resource: &str, verb: &str) -> Self {
        Self {
            api_group: api_group.to_string(),
            resource: resource.to_string(),
            verb: verb.to_string(),
            resource_name: None,
        }
    }

    // Common guards for convenience

    pub fn admin_only() -> Self {
        Self::new("*", "*", "*")
    }

    pub fn read_roles() -> Self {
        Self::new("rbac", "roles", "get")
    }

    pub fn manage_roles() -> Self {
        Self::new("rbac", "roles", "*")
    }

    pub fn manage_role_bindings() -> Self {
        Self::new("rbac", "rolebindings", "*")
    }
}

impl Guard for RbacGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        let headers = ctx.data::<HeaderMap>()?;
        let auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| async_graphql::Error::new("Missing or invalid authorization header"))?;

        let app_state = ctx.data::<AppState>()?;

        // Decode JWT to get principal
        let principal = match decode_rbac_jwt(auth_header, &app_state.jwt_secret) {
            Ok(claims) => {
                match claims.sub_type {
                    SubjectType::Subject => AuthPrincipal::Subject(Subject { name: claims.sub }),
                    SubjectType::ServiceAccount => {
                        // Get service account from database
                        match app_state
                            .get_service_account(&claims.sub, claims.namespace.as_deref())
                            .await
                        {
                            Ok(Some(sa)) => AuthPrincipal::ServiceAccount(sa),
                            Ok(None) => {
                                return Err(async_graphql::Error::new("Service account not found")
                                    .extend_with(|_, e| e.set("code", "UNAUTHORIZED")));
                            }
                            Err(_) => {
                                return Err(async_graphql::Error::new("Database error")
                                    .extend_with(|_, e| e.set("code", "INTERNAL_ERROR")));
                            }
                        }
                    }
                }
            }
            Err(_) => {
                return Err(async_graphql::Error::new("Invalid or expired token")
                    .extend_with(|_, e| e.set("code", "UNAUTHORIZED")));
            }
        };

        // Check permissions using RBAC
        let context = self.to_permission_context();

        match check_permission(&principal, app_state, &context).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(async_graphql::Error::new("Insufficient permissions")
                .extend_with(|_, e| e.set("code", "FORBIDDEN"))),
            Err(_e) => Err(async_graphql::Error::new("Authorization check failed")
                .extend_with(|_, e| e.set("code", "INTERNAL_ERROR"))),
        }
    }
}

impl RbacGuard {
    fn to_permission_context(&self) -> PermissionContext {
        let mut context = PermissionContext::new(&self.api_group, &self.resource, &self.verb);
        if let Some(name) = &self.resource_name {
            context = context.with_resource_name(name);
        }
        context
    }
}

// Legacy Auth guard for backward compatibility during migration
pub struct AuthGuard;

impl AuthGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        // For now, treat any authenticated request as admin during migration
        let headers = ctx.data::<HeaderMap>()?;
        let auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| async_graphql::Error::new("Missing or invalid authorization header"))?;

        let app_state = ctx.data::<AppState>()?;

        match decode_rbac_jwt(auth_header, &app_state.jwt_secret) {
            Ok(_) => Ok(()),
            Err(_) => Err(async_graphql::Error::new("Invalid or expired token")
                .extend_with(|_, e| e.set("code", "UNAUTHORIZED"))),
        }
    }
}

// JWT utility functions for RBAC
pub fn create_service_account_jwt(
    service_account: &ServiceAccount,
    secret: &str,
    duration_hours: i64,
) -> Result<TokenResponse> {
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(duration_hours))
        .expect("valid timestamp");

    let claims = RbacClaims {
        sub: service_account.user.clone(),
        sub_type: SubjectType::ServiceAccount,
        namespace: service_account.namespace.clone(),
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        iss: "raworc-rbac".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(TokenResponse {
        token,
        expires_at: exp.to_rfc3339(),
    })
}

pub fn create_subject_jwt(
    subject_name: &str,
    secret: &str,
    duration_hours: i64,
) -> Result<TokenResponse> {
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(duration_hours))
        .expect("valid timestamp");

    let claims = RbacClaims {
        sub: subject_name.to_string(),
        sub_type: SubjectType::Subject,
        namespace: None,
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        iss: "raworc-rbac".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(TokenResponse {
        token,
        expires_at: exp.to_rfc3339(),
    })
}

pub fn decode_rbac_jwt(token: &str, secret: &str) -> Result<RbacClaims> {
    let token_data: TokenData<RbacClaims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

// Permission checking function
pub async fn check_permission(
    principal: &AuthPrincipal,
    app_state: &AppState,
    context: &PermissionContext,
) -> Result<bool, DatabaseError> {
    // Get all roles and role bindings
    let roles = app_state.get_all_roles().await?;
    let role_bindings = app_state
        .get_role_bindings_for_subject(
            principal.name(),
            principal.subject_type(),
            principal.namespace(),
        )
        .await?;

    // Use RBAC authorization engine
    let result = RbacAuthz::has_permission(principal, &roles, &role_bindings, context);
    Ok(result)
}

// Helper function to extract RBAC claims from context
pub fn get_rbac_claims_from_context(ctx: &Context<'_>) -> async_graphql::Result<RbacClaims> {
    let headers = ctx.data::<HeaderMap>()?;
    let app_state = ctx.data::<AppState>()?;

    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| async_graphql::Error::new("Missing authorization header"))?;

    decode_rbac_jwt(auth_header, &app_state.jwt_secret)
        .map_err(|_| async_graphql::Error::new("Invalid token"))
}

// Helper to get authenticated principal from context
pub async fn get_principal_from_context(ctx: &Context<'_>) -> async_graphql::Result<AuthPrincipal> {
    let claims = get_rbac_claims_from_context(ctx)?;
    let app_state = ctx.data::<AppState>()?;

    match claims.sub_type {
        SubjectType::Subject => Ok(AuthPrincipal::Subject(Subject { name: claims.sub })),
        SubjectType::ServiceAccount => {
            match app_state
                .get_service_account(&claims.sub, claims.namespace.as_deref())
                .await
            {
                Ok(Some(sa)) => Ok(AuthPrincipal::ServiceAccount(sa)),
                Ok(None) => Err(async_graphql::Error::new("Service account not found")),
                Err(_) => Err(async_graphql::Error::new("Database error")),
            }
        }
    }
}

// Authentication functions
pub async fn authenticate_service_account(
    app_state: &AppState,
    user: &str,
    namespace: Option<&str>,
    pass: &str,
) -> Result<Option<ServiceAccount>, DatabaseError> {
    if let Some(service_account) = app_state.get_service_account(user, namespace).await? {
        if service_account.active {
            let is_valid = bcrypt::verify(pass, &service_account.pass_hash).unwrap_or(false);
            if is_valid {
                return Ok(Some(service_account));
            }
        }
    }
    Ok(None)
}
