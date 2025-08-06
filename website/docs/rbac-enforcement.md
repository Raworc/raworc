# RBAC Enforcement in Raworc

## Overview

Raworc implements a comprehensive Role-Based Access Control (RBAC) system that enforces permissions across all API endpoints. The system follows a Kubernetes-style RBAC model with roles, role bindings, and namespace-based resource isolation.

## Architecture

### Key Components

1. **Service Accounts**: Global entities that represent users or automated systems
2. **Roles**: Define sets of permissions (api_groups, resources, verbs)
3. **Role Bindings**: Connect roles to principals (service accounts) and specify scope
4. **Namespaces**: Provide resource isolation for multi-tenant environments

### Namespace Model

- **Service Accounts** and **Roles** are global entities
- **Resources** (Agents, Sessions) are namespace-scoped
- **Role Bindings** can be:
  - Global (namespace = NULL): Permissions apply across all namespaces
  - Namespace-scoped: Permissions apply only within specified namespace

## Permission Model

### Permission Structure

```rust
pub struct PermissionContext {
    pub api_group: String,      // e.g., "api", "*"
    pub resource: String,        // e.g., "sessions", "agents", "*"
    pub verb: String,           // e.g., "create", "list", "get", "*"
    pub resource_name: Option<String>,  // Specific resource name
    pub namespace: Option<String>,      // Target namespace
}
```

### API Endpoint Permissions

#### Service Accounts (Global)
- `api/service-accounts/list` - List all service accounts
- `api/service-accounts/get` - Get specific service account
- `api/service-accounts/create` - Create new service account
- `api/service-accounts/update` - Update service account
- `api/service-accounts/delete` - Delete service account

#### Roles (Global)
- `api/roles/list` - List all roles
- `api/roles/get` - Get specific role
- `api/roles/create` - Create new role
- `api/roles/delete` - Delete role

#### Role Bindings (Global)
- `api/role-bindings/list` - List all role bindings
- `api/role-bindings/get` - Get specific role binding
- `api/role-bindings/create` - Create new role binding
- `api/role-bindings/delete` - Delete role binding

#### Agents (Namespace-scoped)
- `api/agents/list` - List agents in namespace
- `api/agents/get` - Get specific agent
- `api/agents/create` - Create agent in namespace
- `api/agents/update` - Update agent
- `api/agents/delete` - Delete agent

#### Sessions (Namespace-scoped)
- `api/sessions/list` - List sessions in namespace
- `api/sessions/get` - Get specific session
- `api/sessions/create` - Create session in namespace
- `api/sessions/update` - Update session
- `api/sessions/delete` - Delete session
- `api/sessions/list-all` - List all users' sessions (admin)

## Implementation Details

### Middleware

The authentication middleware (`auth_middleware`) validates JWT tokens and extracts the authenticated principal. It skips authentication for public endpoints like health checks and authentication endpoints.

### Permission Checking

Each API handler performs permission checks using the `check_api_permission` function:

```rust
check_api_permission(&auth, &state, &permissions::AGENT_CREATE, Some(&namespace))
    .await
    .map_err(|e| match e {
        StatusCode::FORBIDDEN => ApiError::Forbidden("Insufficient permissions".to_string()),
        _ => ApiError::Internal(anyhow::anyhow!("Permission check failed")),
    })?;
```

### Namespace Resolution

For namespace-scoped resources, the system:
1. Uses explicitly provided namespace if available
2. Falls back to user's default namespace from JWT claims
3. Defaults to "default" namespace if neither is specified

### Special Cases

#### Password Updates
Users can update their own password without admin permissions. Admin permission is required to update other users' passwords.

#### Session Access
Users can always access their own sessions. Admin permission or appropriate namespace access is required to view other users' sessions.

## Default Roles

### Admin Role
The default admin role has full access to all resources:

```json
{
  "name": "admin",
  "rules": [{
    "api_groups": ["*"],
    "resources": ["*"],
    "verbs": ["*"]
  }]
}
```

### Viewer Role
Read-only access to resources:

```json
{
  "name": "viewer",
  "rules": [{
    "api_groups": ["api"],
    "resources": ["*"],
    "verbs": ["get", "list"]
  }]
}
```

## Testing RBAC

### Create a Test User
```bash
# As admin, create a new service account
curl -X POST http://localhost:9000/api/v0/service-accounts \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user": "testuser", "pass": "test123"}'
```

### Test Without Permissions
```bash
# Login as testuser
TOKEN=$(curl -X POST http://localhost:9000/api/v0/auth/internal \
  -H "Content-Type: application/json" \
  -d '{"user": "testuser", "pass": "test123"}' | jq -r '.token')

# Try to list service accounts (should fail with 403)
curl -X GET http://localhost:9000/api/v0/service-accounts \
  -H "Authorization: Bearer $TOKEN"
```

### Grant Permissions
```bash
# As admin, create a role binding for testuser
curl -X POST http://localhost:9000/api/v0/role-bindings \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_name": "viewer",
    "principal_name": "testuser",
    "principal_type": "ServiceAccount",
    "namespace": "default"
  }'
```

### Test With Permissions
```bash
# Now testuser can list resources in the default namespace
curl -X GET http://localhost:9000/api/v0/agents?namespace=default \
  -H "Authorization: Bearer $TOKEN"
```

## Security Considerations

1. **Default Deny**: All endpoints require authentication except public health checks
2. **Principle of Least Privilege**: Users start with no permissions
3. **Namespace Isolation**: Resources are isolated by namespace
4. **Audit Logging**: All API requests are logged with user and permission context
5. **Token Security**: JWT tokens have expiration and are validated on each request

## Future Enhancements

1. **Resource-level permissions**: Support for specific resource names in permission checks
2. **Dynamic role creation**: API for creating custom roles
3. **Permission aggregation**: Support for ClusterRoles and aggregated permissions
4. **Audit events**: Detailed audit log for permission grants/revokes
5. **Service account impersonation**: Allow admins to impersonate other users for debugging