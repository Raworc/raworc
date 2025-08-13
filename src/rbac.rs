use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

// RBAC Subject - External user identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String, // External subject identifier (e.g., "user@example.com", "system:serviceaccount:namespace:name")
}

// Service Account - Global account with credentials (can work across organizations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub user: String,
    pub pass_hash: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub active: bool,
    pub last_login_at: Option<String>,
}


// Permission Rule - Fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub api_groups: Vec<String>,             // e.g., ["", "api", "rbac"]
    pub resources: Vec<String>,              // e.g., ["users", "roles", "*"]
    pub verbs: Vec<String>,                  // e.g., ["get", "list", "create", "update", "delete"]
    pub resource_names: Option<Vec<String>>, // Optional specific resource names
}

// Role - Global collection of permissions (can be bound to specific organizations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    pub rules: Vec<Rule>,
    pub description: Option<String>,
    pub created_at: String,
}


// Subject type for role bindings
#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, ToSchema)]
pub enum SubjectType {
    Subject,
    ServiceAccount,
}

// Role Binding Subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBindingSubject {
    pub kind: SubjectType,
    pub name: String,
}

// Role Binding - Links roles to subjects and specifies WHERE they apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBinding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub role_name: String,
    pub principal_name: String,
    pub principal_type: SubjectType,
    pub workspace: Option<String>, // NULL = global access, String = specific organization
    pub created_at: String,
}


// Role Reference for bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRef {
    pub kind: String, // "Role" or "ClusterRole"
    pub name: String,
    pub api_group: String, // API group for permissions, typically "rbac"
}

// Authentication Principal - Represents authenticated entity
#[derive(Debug, Clone)]
pub enum AuthPrincipal {
    Subject(Subject),
    ServiceAccount(ServiceAccount),
}

impl AuthPrincipal {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            AuthPrincipal::Subject(s) => &s.name,
            AuthPrincipal::ServiceAccount(sa) => &sa.user,
        }
    }

    #[allow(dead_code)]
    pub fn workspace(&self) -> Option<&str> {
        // Service accounts are global now, no workspace
        None
    }

    #[allow(dead_code)]
    pub fn subject_type(&self) -> SubjectType {
        match self {
            AuthPrincipal::Subject(_) => SubjectType::Subject,
            AuthPrincipal::ServiceAccount(_) => SubjectType::ServiceAccount,
        }
    }
}

// JWT Claims for RBAC authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacClaims {
    pub sub: String,               // Subject name
    pub sub_type: SubjectType,     // Subject type
    pub workspace: Option<String>, // For service accounts
    pub exp: usize,                // Expiration time
    pub iat: usize,                // Issued at
    pub iss: String,               // Issuer
}

// Input types for API requests
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateServiceAccountInput {
    pub user: String,
    pub workspace: Option<String>,
    pub pass: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateRoleInput {
    pub name: String,
    pub workspace: Option<String>,
    pub rules: Vec<RuleInput>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RuleInput {
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub verbs: Vec<String>,
    pub resource_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateRoleBindingInput {
    pub name: String,
    pub workspace: Option<String>,
    pub role_ref: RoleRefInput,
    pub subjects: Vec<RoleBindingSubjectInput>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RoleRefInput {
    pub kind: String,
    pub name: String,
    pub api_group: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RoleBindingSubjectInput {
    pub kind: SubjectType,
    pub name: String,
    pub workspace: Option<String>,
}

// Token generation response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: String,
}

// Permission check context
#[derive(Debug)]
#[allow(dead_code)]
pub struct PermissionContext {
    pub api_group: String,
    pub resource: String,
    pub verb: String,
    pub resource_name: Option<String>,
    #[allow(dead_code)]
    pub workspace: Option<String>,
}

impl PermissionContext {
    #[allow(dead_code)]
    pub fn new(api_group: &str, resource: &str, verb: &str) -> Self {
        Self {
            api_group: api_group.to_string(),
            resource: resource.to_string(),
            verb: verb.to_string(),
            resource_name: None,
            workspace: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_resource_name(mut self, name: &str) -> Self {
        self.resource_name = Some(name.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_workspace(mut self, workspace: &str) -> Self {
        self.workspace = Some(workspace.to_string());
        self
    }
}

// RBAC Authorization service
#[allow(dead_code)]
pub struct RbacAuthz;

impl RbacAuthz {
    // Check if a principal has permission for a given context
    #[allow(dead_code)]
    pub fn has_permission(
        principal: &AuthPrincipal,
        roles: &[Role],
        role_bindings: &[RoleBinding],
        context: &PermissionContext,
    ) -> bool {
        // Find role bindings that apply to this principal
        let applicable_bindings = Self::get_applicable_bindings(principal, role_bindings);

        // Get all roles bound to this principal
        let bound_roles: Vec<&Role> = applicable_bindings
            .iter()
            .filter_map(|binding| {
                roles.iter().find(|role| role.name == binding.role_name)
            })
            .collect();

        // Check if any bound role grants the required permission
        bound_roles
            .iter()
            .any(|role| Self::role_grants_permission(role, context))
    }

    #[allow(dead_code)]
    fn get_applicable_bindings<'a>(
        principal: &AuthPrincipal,
        role_bindings: &'a [RoleBinding],
    ) -> Vec<&'a RoleBinding> {
        role_bindings
            .iter()
            .filter(|binding| {
                binding.principal_type == principal.subject_type()
                    && binding.principal_name == principal.name()
            })
            .collect()
    }


    #[allow(dead_code)]
    fn role_grants_permission(role: &Role, context: &PermissionContext) -> bool {
        role.rules
            .iter()
            .any(|rule| Self::rule_grants_permission(rule, context))
    }

    #[allow(dead_code)]
    fn rule_grants_permission(rule: &Rule, context: &PermissionContext) -> bool {
        // Check API groups
        let api_group_match = rule.api_groups.contains(&"*".to_string())
            || rule.api_groups.contains(&context.api_group);

        // Check resources
        let resource_match =
            rule.resources.contains(&"*".to_string()) || rule.resources.contains(&context.resource);

        // Check verbs
        let verb_match =
            rule.verbs.contains(&"*".to_string()) || rule.verbs.contains(&context.verb);

        // Check resource names if specified
        let resource_name_match = match (&rule.resource_names, &context.resource_name) {
            (None, _) => true, // No restriction on resource names
            (Some(allowed_names), Some(requested_name)) => {
                allowed_names.contains(&"*".to_string()) || allowed_names.contains(requested_name)
            }
            (Some(_), None) => false, // Rule restricts names but none provided
        };

        api_group_match && resource_match && verb_match && resource_name_match
    }
}

// Pre-defined system roles
pub fn get_admin_role() -> Role {
    Role {
        id: None,
        name: "admin".to_string(),
        rules: vec![Rule {
            api_groups: vec!["*".to_string()],
            resources: vec!["*".to_string()],
            verbs: vec!["*".to_string()],
            resource_names: None,
        }],
        description: Some("Full cluster admin access".to_string()),
        created_at: Utc::now().to_rfc3339(),
    }
}
