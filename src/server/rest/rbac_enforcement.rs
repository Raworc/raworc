use axum::http::StatusCode;
use crate::shared::models::AppState;
use crate::server::rest::middleware::AuthContext;
use crate::server::rbac::PermissionContext;
use crate::server::auth::check_permission;

/// Permission requirements for each API endpoint
#[allow(dead_code)]
pub struct PermissionRequirement {
    pub api_group: &'static str,
    pub resource: &'static str,
    pub verb: &'static str,
    pub workspace_scoped: bool,
}

impl PermissionRequirement {
    pub const fn new(api_group: &'static str, resource: &'static str, verb: &'static str, workspace_scoped: bool) -> Self {
        Self {
            api_group,
            resource,
            verb,
            workspace_scoped,
        }
    }
}

/// Check if user has permission for the requested action
pub async fn check_api_permission(
    auth: &AuthContext,
    state: &AppState,
    requirement: &PermissionRequirement,
    target_workspace: Option<&str>,
) -> Result<(), StatusCode> {
    let context = PermissionContext {
        api_group: requirement.api_group.to_string(),
        resource: requirement.resource.to_string(),
        verb: requirement.verb.to_string(),
        resource_name: None,
        workspace: target_workspace.map(|s| s.to_string()),
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub const ROLE_BINDING_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "update", false);
    pub const ROLE_BINDING_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "role-bindings", "delete", false);

    // Agent permissions (workspace-scoped)
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

    // Session permissions (workspace-scoped)
    #[allow(dead_code)]
    pub const SESSION_LIST: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "list", true);
    #[allow(dead_code)]
    pub const SESSION_GET: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "get", true);
    #[allow(dead_code)]
    pub const SESSION_CREATE: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "create", true);
    pub const SESSION_UPDATE: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "update", true);
    pub const SESSION_DELETE: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "delete", true);
    #[allow(dead_code)]
    pub const SESSION_LIST_ALL: PermissionRequirement = 
        PermissionRequirement::new("api", "sessions", "list-all", false);
}

/// Extract workspace from JWT claims
pub fn get_user_workspace(auth: &AuthContext) -> Option<String> {
    auth.claims.workspace.clone()
}

/// Check if user can access a specific workspace
#[allow(dead_code)]
pub async fn check_workspace_access(
    auth: &AuthContext,
    state: &AppState,
    target_workspace: &str,
) -> Result<bool, StatusCode> {
    // Check if user has global access
    let global_context = PermissionContext {
        api_group: "*".to_string(),
        resource: "*".to_string(),
        verb: "*".to_string(),
        resource_name: None,
        workspace: None,
    };

    let has_global = check_permission(&auth.principal, state, &global_context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if has_global {
        return Ok(true);
    }

    // Check if user has access to the specific workspace
    let workspace_context = PermissionContext {
        api_group: "*".to_string(),
        resource: "*".to_string(),
        verb: "*".to_string(),
        resource_name: None,
        workspace: Some(target_workspace.to_string()),
    };

    let has_workspace_access = check_permission(&auth.principal, state, &workspace_context)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(has_workspace_access)
}