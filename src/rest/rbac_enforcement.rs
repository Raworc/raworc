use axum::http::StatusCode;
use crate::models::AppState;
use crate::rest::middleware::AuthContext;
use crate::rbac::PermissionContext;
use crate::auth::check_permission;

/// Permission requirements for each API endpoint
pub struct PermissionRequirement {
    pub api_group: &'static str,
    pub resource: &'static str,
    pub verb: &'static str,
    pub namespace_scoped: bool,
}

impl PermissionRequirement {
    pub const fn new(api_group: &'static str, resource: &'static str, verb: &'static str, namespace_scoped: bool) -> Self {
        Self {
            api_group,
            resource,
            verb,
            namespace_scoped,
        }
    }
}

/// Check if user has permission for the requested action
pub async fn check_api_permission(
    auth: &AuthContext,
    state: &AppState,
    requirement: &PermissionRequirement,
    target_namespace: Option<&str>,
) -> Result<(), StatusCode> {
    let context = PermissionContext {
        api_group: requirement.api_group.to_string(),
        resource: requirement.resource.to_string(),
        verb: requirement.verb.to_string(),
        resource_name: None,
        namespace: target_namespace.map(|s| s.to_string()),
    };

    let has_permission = check_permission(&auth.principal, state, &context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}

/// Permission definitions for all API endpoints
pub mod permissions {
    use super::PermissionRequirement;

    // Service Account permissions
    pub const SERVICE_ACCOUNT_LIST: PermissionRequirement = 
        PermissionRequirement::new("api", "service-accounts", "list", false);
    pub const SERVICE_ACCOUNT_GET: PermissionRequirement = 
        PermissionRequirement::new("api", "service-accounts", "get", false);
    pub const SERVICE_ACCOUNT_CREATE: PermissionRequirement = 
        PermissionRequirement::new("api", "service-accounts", "create", false);
    pub const SERVICE_ACCOUNT_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "service-accounts", "update", false);
    pub const SERVICE_ACCOUNT_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "service-accounts", "delete", false);

    // Role permissions
    pub const ROLE_LIST: PermissionRequirement = 
        PermissionRequirement::new("api", "roles", "list", false);
    pub const ROLE_GET: PermissionRequirement = 
        PermissionRequirement::new("api", "roles", "get", false);
    pub const ROLE_CREATE: PermissionRequirement = 
        PermissionRequirement::new("api", "roles", "create", false);
    pub const ROLE_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "roles", "update", false);
    pub const ROLE_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "roles", "delete", false);

    // Role Binding permissions
    pub const ROLE_BINDING_LIST: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "list", false);
    pub const ROLE_BINDING_GET: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "get", false);
    pub const ROLE_BINDING_CREATE: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "create", false);
    pub const ROLE_BINDING_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "update", false);
    pub const ROLE_BINDING_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "delete", false);

    // Agent permissions (namespace-scoped)
    pub const AGENT_LIST: PermissionRequirement = 
        PermissionRequirement::new("api", "agents", "list", true);
    pub const AGENT_GET: PermissionRequirement = 
        PermissionRequirement::new("api", "agents", "get", true);
    pub const AGENT_CREATE: PermissionRequirement = 
        PermissionRequirement::new("api", "agents", "create", true);
    pub const AGENT_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "agents", "update", true);
    pub const AGENT_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "agents", "delete", true);

    // Session permissions (namespace-scoped)
    pub const SESSION_LIST: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "list", true);
    pub const SESSION_GET: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "get", true);
    pub const SESSION_CREATE: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "create", true);
    pub const SESSION_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "update", true);
    pub const SESSION_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "delete", true);
    pub const SESSION_LIST_ALL: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "list-all", false);
}

/// Extract namespace from JWT claims
pub fn get_user_namespace(auth: &AuthContext) -> Option<String> {
    auth.claims.namespace.clone()
}

/// Check if user can access a specific namespace
pub async fn check_namespace_access(
    auth: &AuthContext,
    state: &AppState,
    target_namespace: &str,
) -> Result<bool, StatusCode> {
    // Check if user has global access
    let global_context = PermissionContext {
        api_group: "*".to_string(),
        resource: "*".to_string(),
        verb: "*".to_string(),
        resource_name: None,
        namespace: None,
    };

    let has_global = check_permission(&auth.principal, state, &global_context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if has_global {
        return Ok(true);
    }

    // Check if user has access to the specific namespace
    let namespace_context = PermissionContext {
        api_group: "*".to_string(),
        resource: "*".to_string(),
        verb: "*".to_string(),
        resource_name: None,
        namespace: Some(target_namespace.to_string()),
    };

    let has_namespace_access = check_permission(&auth.principal, state, &namespace_context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(has_namespace_access)
}