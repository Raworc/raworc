use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

use crate::rest::{
    auth::{LoginRequest, LoginResponse, ExternalLoginRequest},
    handlers::{
        service_accounts::{CreateServiceAccountRequest, ServiceAccountResponse},
        roles::{CreateRoleRequest, RoleResponse, RuleRequest, RuleResponse},
        role_bindings::{CreateRoleBindingRequest, RoleBindingResponse, RoleRefRequest, SubjectRequest},
        agents::AgentResponse,
    },
    error::ErrorResponse,
};
use crate::models::{CreateAgentRequest, UpdateAgentRequest};
use crate::rbac::SubjectType;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::rest::openapi::health,
        crate::rest::openapi::version,
        crate::rest::openapi::login,
        crate::rest::openapi::external_login,
        crate::rest::openapi::me,
        crate::rest::openapi::list_service_accounts,
        crate::rest::openapi::get_service_account,
        crate::rest::openapi::create_service_account,
        crate::rest::openapi::delete_service_account,
        crate::rest::openapi::list_roles,
        crate::rest::openapi::get_role,
        crate::rest::openapi::create_role,
        crate::rest::openapi::delete_role,
        crate::rest::openapi::list_role_bindings,
        crate::rest::openapi::get_role_binding,
        crate::rest::openapi::create_role_binding,
        crate::rest::openapi::delete_role_binding,
        crate::rest::openapi::list_agents,
        crate::rest::openapi::get_agent,
        crate::rest::openapi::create_agent,
        crate::rest::openapi::update_agent,
        crate::rest::openapi::delete_agent,
    ),
    components(
        schemas(
            LoginRequest,
            LoginResponse,
            ExternalLoginRequest,
            CreateServiceAccountRequest,
            ServiceAccountResponse,
            CreateRoleRequest,
            RoleResponse,
            RuleRequest,
            RuleResponse,
            CreateRoleBindingRequest,
            RoleBindingResponse,
            RoleRefRequest,
            SubjectRequest,
            SubjectType,
            ErrorResponse,
            crate::rest::error::ErrorDetails,
            AgentResponse,
            CreateAgentRequest,
            UpdateAgentRequest,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Service Accounts", description = "Service account management"),
        (name = "Roles", description = "Role management"),
        (name = "Role Bindings", description = "Role binding management"),
        (name = "Agents", description = "Agent management"),
    ),
    info(
        title = "Raworc REST API",
        version = "1.0.0",
        description = "Remote Agent Work Orchestration REST API with RBAC",
        license(name = "MIT"),
    ),
    servers(
        (url = "/", description = "Current server"),
    ),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}

// Health endpoints
#[utoipa::path(
    get,
    path = "/api/v0/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy"),
    ),
)]
#[allow(dead_code)]
pub async fn health() {}

#[utoipa::path(
    get,
    path = "/api/v0/version",
    tag = "Health",
    responses(
        (status = 200, description = "API version", body = String),
    ),
)]
#[allow(dead_code)]
pub async fn version() {}

// Auth endpoints
#[utoipa::path(
    post,
    path = "/api/v0/auth/internal",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn login() {}

#[utoipa::path(
    post,
    path = "/api/v0/auth/external",
    tag = "Auth",
    request_body = ExternalLoginRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "External login successful", body = LoginResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Admin access required", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn external_login() {}

#[utoipa::path(
    get,
    path = "/api/v0/auth/me",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user info", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn me() {}

// Service Account endpoints
#[utoipa::path(
    get,
    path = "/api/v0/service-accounts",
    tag = "Service Accounts",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of service accounts", body = Vec<ServiceAccountResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn list_service_accounts() {}

#[utoipa::path(
    get,
    path = "/api/v0/service-accounts/{id}",
    tag = "Service Accounts",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Service account ID or username"),
    ),
    responses(
        (status = 200, description = "Service account details", body = ServiceAccountResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Service account not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn get_service_account() {}

#[utoipa::path(
    post,
    path = "/api/v0/service-accounts",
    tag = "Service Accounts",
    request_body = CreateServiceAccountRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Service account created", body = ServiceAccountResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 409, description = "Service account already exists", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn create_service_account() {}

#[utoipa::path(
    delete,
    path = "/api/v0/service-accounts/{id}",
    tag = "Service Accounts",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Service account ID or username"),
    ),
    responses(
        (status = 204, description = "Service account deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Service account not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn delete_service_account() {}

// Role endpoints
#[utoipa::path(
    get,
    path = "/api/v0/roles",
    tag = "Roles",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of roles", body = Vec<RoleResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn list_roles() {}

#[utoipa::path(
    get,
    path = "/api/v0/roles/{id}",
    tag = "Roles",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Role ID or name"),
    ),
    responses(
        (status = 200, description = "Role details", body = RoleResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn get_role() {}

#[utoipa::path(
    post,
    path = "/api/v0/roles",
    tag = "Roles",
    request_body = CreateRoleRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Role created", body = RoleResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 409, description = "Role already exists", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn create_role() {}

#[utoipa::path(
    delete,
    path = "/api/v0/roles/{id}",
    tag = "Roles",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Role ID or name"),
    ),
    responses(
        (status = 204, description = "Role deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Role not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn delete_role() {}

// Role Binding endpoints
#[utoipa::path(
    get,
    path = "/api/v0/role-bindings",
    tag = "Role Bindings",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of role bindings", body = Vec<RoleBindingResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn list_role_bindings() {}

#[utoipa::path(
    get,
    path = "/api/v0/role-bindings/{id}",
    tag = "Role Bindings",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Role binding ID or name"),
    ),
    responses(
        (status = 200, description = "Role binding details", body = RoleBindingResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Role binding not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn get_role_binding() {}

#[utoipa::path(
    post,
    path = "/api/v0/role-bindings",
    tag = "Role Bindings",
    request_body = CreateRoleBindingRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Role binding created", body = RoleBindingResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 409, description = "Role binding already exists", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn create_role_binding() {}

#[utoipa::path(
    delete,
    path = "/api/v0/role-bindings/{id}",
    tag = "Role Bindings",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Role binding ID or name"),
    ),
    responses(
        (status = 204, description = "Role binding deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Role binding not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn delete_role_binding() {}

// Agent endpoints
#[utoipa::path(
    get,
    path = "/api/v0/agents",
    tag = "Agents",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of agents", body = Vec<AgentResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn list_agents() {}

#[utoipa::path(
    get,
    path = "/api/v0/agents/{id}",
    tag = "Agents",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Agent ID or name"),
    ),
    responses(
        (status = 200, description = "Agent details", body = AgentResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn get_agent() {}

#[utoipa::path(
    post,
    path = "/api/v0/agents",
    tag = "Agents",
    request_body = CreateAgentRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Agent created", body = AgentResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 409, description = "Agent already exists", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn create_agent() {}

#[utoipa::path(
    put,
    path = "/api/v0/agents/{id}",
    tag = "Agents",
    request_body = UpdateAgentRequest,
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Agent ID"),
    ),
    responses(
        (status = 200, description = "Agent updated", body = AgentResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
        (status = 409, description = "Agent name conflict", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn update_agent() {}

#[utoipa::path(
    delete,
    path = "/api/v0/agents/{id}",
    tag = "Agents",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "Agent ID"),
    ),
    responses(
        (status = 204, description = "Agent deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 404, description = "Agent not found", body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn delete_agent() {}