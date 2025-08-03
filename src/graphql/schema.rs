use crate::auth::{
    authenticate_service_account, create_service_account_jwt, create_subject_jwt,
    get_principal_from_context, AuthGuard, RbacGuard,
};
use crate::models::AppState;
use crate::rbac::{
    get_admin_role, CreateRoleBindingInput, CreateRoleInput, CreateServiceAccountInput, Role,
    RoleBinding, RoleBindingSubject, RoleRef, Rule, ServiceAccount, TokenResponse,
};
use async_graphql::{Context, FieldResult, InputObject, Object};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use tracing::info;

// Service Account Login Input
#[derive(InputObject)]
pub struct ServiceAccountLoginInput {
    pub user: String,
    pub namespace: Option<String>,
    pub pass: String,
}

// Subject Authentication Input
#[derive(InputObject)]
pub struct SubjectLoginInput {
    pub subject: String,
}

// GraphQL Queries
pub struct Query;

#[Object]
impl Query {
    // Public queries
    async fn version(&self) -> &str {
        "0.1.0"
    }

    async fn status(&self) -> &str {
        "Server running"
    }

    // RBAC Queries
    #[graphql(guard = "RbacGuard::new(\"rbac\", \"serviceaccounts\", \"list\")")]
    async fn service_accounts(&self, ctx: &Context<'_>) -> FieldResult<Vec<ServiceAccount>> {
        let state = ctx.data::<AppState>()?;
        let service_accounts = state.get_all_service_accounts().await?;
        Ok(service_accounts)
    }

    #[graphql(guard = "RbacGuard::new(\"rbac\", \"serviceaccounts\", \"get\")")]
    async fn service_account(
        &self,
        ctx: &Context<'_>,
        user: String,
        namespace: Option<String>,
    ) -> FieldResult<Option<ServiceAccount>> {
        let state = ctx.data::<AppState>()?;
        let service_account = state
            .get_service_account(&user, namespace.as_deref())
            .await?;
        Ok(service_account)
    }

    #[graphql(guard = "RbacGuard::read_roles()")]
    async fn roles(&self, ctx: &Context<'_>) -> FieldResult<Vec<Role>> {
        let state = ctx.data::<AppState>()?;
        let roles = state.get_all_roles().await?;
        Ok(roles)
    }

    #[graphql(guard = "RbacGuard::read_roles()")]
    async fn role(
        &self,
        ctx: &Context<'_>,
        name: String,
        namespace: Option<String>,
    ) -> FieldResult<Option<Role>> {
        let state = ctx.data::<AppState>()?;
        let role = state.get_role(&name, namespace.as_deref()).await?;
        Ok(role)
    }

    #[graphql(guard = "RbacGuard::new(\"rbac\", \"rolebindings\", \"list\")")]
    async fn role_bindings(&self, ctx: &Context<'_>) -> FieldResult<Vec<RoleBinding>> {
        let state = ctx.data::<AppState>()?;
        let role_bindings = state.get_all_role_bindings().await?;
        Ok(role_bindings)
    }

    #[graphql(guard = "RbacGuard::new(\"rbac\", \"rolebindings\", \"get\")")]
    async fn role_binding(
        &self,
        ctx: &Context<'_>,
        name: String,
        namespace: Option<String>,
    ) -> FieldResult<Option<RoleBinding>> {
        let state = ctx.data::<AppState>()?;
        let role_binding = state.get_role_binding(&name, namespace.as_deref()).await?;
        Ok(role_binding)
    }

    // Get current principal info
    #[graphql(guard = "AuthGuard::new()")]
    async fn whoami(&self, ctx: &Context<'_>) -> FieldResult<String> {
        let principal = get_principal_from_context(ctx).await?;
        match principal {
            crate::rbac::AuthPrincipal::Subject(s) => Ok(format!("Subject: {name}", name = s.name)),
            crate::rbac::AuthPrincipal::ServiceAccount(sa) => Ok(format!(
                "ServiceAccount: {user}{namespace}",
                user = sa.user,
                namespace = sa
                    .namespace
                    .as_ref()
                    .map(|ns| format!(" (namespace: {ns})"))
                    .unwrap_or_default()
            )),
        }
    }
}

// GraphQL Mutations
pub struct Mutation;

#[Object]
impl Mutation {
    // RBAC Authentication
    async fn generate_service_token(
        &self,
        ctx: &Context<'_>,
        input: ServiceAccountLoginInput,
    ) -> FieldResult<TokenResponse> {
        let state = ctx.data::<AppState>()?;

        let service_account = authenticate_service_account(
            state,
            &input.user,
            input.namespace.as_deref(),
            &input.pass,
        )
        .await?
        .ok_or_else(|| async_graphql::Error::new("Invalid service account credentials"))?;

        let token_response = create_service_account_jwt(&service_account, &state.jwt_secret, 24)?;
        Ok(token_response)
    }

    // Generate token for external subject (requires admin authentication)
    #[graphql(guard = "RbacGuard::admin_only()")]
    async fn generate_external_token(
        &self,
        ctx: &Context<'_>,
        input: SubjectLoginInput,
    ) -> FieldResult<TokenResponse> {
        let state = ctx.data::<AppState>()?;

        // Only authenticated admins can create tokens for external subjects
        // This prevents unauthorized token generation
        let token_response = create_subject_jwt(&input.subject, &state.jwt_secret, 24)?;
        Ok(token_response)
    }

    // Service Account Management
    #[graphql(guard = "RbacGuard::new(\"rbac\", \"serviceaccounts\", \"create\")")]
    async fn create_service_account(
        &self,
        ctx: &Context<'_>,
        input: CreateServiceAccountInput,
    ) -> FieldResult<ServiceAccount> {
        let state = ctx.data::<AppState>()?;

        // Check if service account already exists
        if let Ok(Some(_)) = state
            .get_service_account(&input.user, input.namespace.as_deref())
            .await
        {
            return Err(async_graphql::Error::new("Service account already exists"));
        }

        let pass_hash = hash(&input.pass, DEFAULT_COST)?;
        let service_account = state
            .create_service_account(&input.user, input.namespace, &pass_hash, input.description)
            .await?;

        info!("Service account created: {}", input.user);
        Ok(service_account)
    }

    #[graphql(guard = "RbacGuard::new(\"rbac\", \"serviceaccounts\", \"delete\")")]
    async fn delete_service_account(
        &self,
        ctx: &Context<'_>,
        user: String,
        namespace: Option<String>,
    ) -> FieldResult<bool> {
        let state = ctx.data::<AppState>()?;
        let deleted = state
            .delete_service_account(&user, namespace.as_deref())
            .await?;
        Ok(deleted)
    }

    #[graphql(guard = "RbacGuard::new(\"rbac\", \"serviceaccounts\", \"delete\")")]
    async fn delete_service_account_by_id(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> FieldResult<bool> {
        let state = ctx.data::<AppState>()?;
        let deleted = state.delete_service_account_by_id(&id).await?;
        Ok(deleted)
    }

    // Role Management
    #[graphql(guard = "RbacGuard::manage_roles()")]
    async fn create_role(&self, ctx: &Context<'_>, input: CreateRoleInput) -> FieldResult<Role> {
        let state = ctx.data::<AppState>()?;

        // Check if role already exists
        if let Ok(Some(_)) = state
            .get_role(&input.name, input.namespace.as_deref())
            .await
        {
            return Err(async_graphql::Error::new("Role already exists"));
        }

        let role = Role {
            id: None,
            name: input.name,
            namespace: input.namespace,
            rules: input
                .rules
                .into_iter()
                .map(|r| Rule {
                    api_groups: r.api_groups,
                    resources: r.resources,
                    verbs: r.verbs,
                    resource_names: r.resource_names,
                })
                .collect(),
            description: input.description,
            created_at: Utc::now().to_rfc3339(),
        };

        let created_role = state.create_role(&role).await?;
        Ok(created_role)
    }

    #[graphql(guard = "RbacGuard::manage_roles()")]
    async fn delete_role(
        &self,
        ctx: &Context<'_>,
        name: String,
        namespace: Option<String>,
    ) -> FieldResult<bool> {
        let state = ctx.data::<AppState>()?;
        let deleted = state.delete_role(&name, namespace.as_deref()).await?;
        Ok(deleted)
    }

    // Role Binding Management
    #[graphql(guard = "RbacGuard::manage_role_bindings()")]
    async fn create_role_binding(
        &self,
        ctx: &Context<'_>,
        input: CreateRoleBindingInput,
    ) -> FieldResult<RoleBinding> {
        let state = ctx.data::<AppState>()?;

        // Check if role binding already exists
        if let Ok(Some(_)) = state
            .get_role_binding(&input.name, input.namespace.as_deref())
            .await
        {
            return Err(async_graphql::Error::new("Role binding already exists"));
        }

        let role_binding = RoleBinding {
            id: None,
            name: input.name,
            namespace: input.namespace,
            role_ref: RoleRef {
                kind: input.role_ref.kind,
                name: input.role_ref.name,
                api_group: input.role_ref.api_group,
            },
            subjects: input
                .subjects
                .into_iter()
                .map(|s| RoleBindingSubject {
                    kind: s.kind,
                    name: s.name,
                    namespace: s.namespace,
                })
                .collect(),
            created_at: Utc::now().to_rfc3339(),
        };

        let created_binding = state.create_role_binding(&role_binding).await?;
        Ok(created_binding)
    }

    #[graphql(guard = "RbacGuard::manage_role_bindings()")]
    async fn delete_role_binding(
        &self,
        ctx: &Context<'_>,
        name: String,
        namespace: Option<String>,
    ) -> FieldResult<bool> {
        let state = ctx.data::<AppState>()?;
        let deleted = state
            .delete_role_binding(&name, namespace.as_deref())
            .await?;
        Ok(deleted)
    }

    // Convenience mutations for setting up common roles
    #[graphql(guard = "RbacGuard::admin_only()")]
    async fn setup_admin_role(&self, ctx: &Context<'_>) -> FieldResult<Role> {
        let state = ctx.data::<AppState>()?;

        // Check if admin role already exists
        if let Ok(Some(existing_role)) = state.get_role("admin", None).await {
            return Ok(existing_role);
        }

        let admin_role = get_admin_role();
        let created_role = state.create_role(&admin_role).await?;
        Ok(created_role)
    }
}

// Placeholder for subscriptions
pub struct Subscription;

#[Object]
impl Subscription {
    // Placeholder for future real-time features
    async fn _placeholder(&self) -> &str {
        "Subscriptions not implemented yet"
    }
}
