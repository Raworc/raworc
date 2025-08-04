# Role-Based Access Control (RBAC)

## Overview

Raworc implements a Kubernetes-style RBAC system for fine-grained access control. The system consists of:

- **Service Accounts**: Internal accounts with credentials for authentication
- **Roles**: Define permissions (what actions can be performed)
- **Role Bindings**: Bind roles to principals (who can perform actions)

## Core Concepts

### Service Accounts

Service accounts are the primary authentication mechanism in Raworc. Each service account:
- Has a unique username
- Can be scoped to a namespace
- Authenticates with username/password
- Receives a JWT token upon successful authentication

### Roles

Roles define a set of permissions through rules. Each rule specifies:
- **API Groups**: Which APIs can be accessed (e.g., "api", "rbac", "*")
- **Resources**: Which resources can be accessed (e.g., "roles", "service-accounts", "*")
- **Verbs**: Which actions can be performed (e.g., "get", "list", "create", "update", "delete", "*")

### Role Bindings

Role bindings connect roles to principals (service accounts or external subjects):
- Bind a role to one or more principals
- Can be namespace-scoped or cluster-wide
- Support both ServiceAccount and Subject principal types

## Permission Model

### API Groups
- `""` - Core API group
- `api` - Main application APIs
- `rbac` - RBAC management APIs
- `*` - All API groups (admin access)

### Resources
- `service-accounts` - Service account management
- `roles` - Role management
- `role-bindings` - Role binding management
- `*` - All resources

### Verbs
- `get` - Read a specific resource
- `list` - List resources
- `create` - Create new resources
- `update` - Update existing resources
- `delete` - Delete resources
- `*` - All verbs

## Examples

### Creating a Read-Only Role
```json
{
  "name": "reader",
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["*"],
      "verbs": ["get", "list"]
    }
  ]
}
```

### Creating a Namespace Admin
```json
{
  "name": "namespace-admin",
  "namespace": "production",
  "rules": [
    {
      "api_groups": ["*"],
      "resources": ["*"],
      "verbs": ["*"]
    }
  ]
}
```

### Binding a Role to a Service Account
```json
{
  "role_name": "reader",
  "principal_name": "monitoring-bot",
  "principal_type": "ServiceAccount"
}
```

## Default Roles

### admin
Full cluster administrator access:
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

## Best Practices

1. **Principle of Least Privilege**: Grant only the minimum permissions required
2. **Use Namespaces**: Scope roles and service accounts to namespaces when possible
3. **Regular Audits**: Review role bindings regularly
4. **Separate Accounts**: Use different service accounts for different applications
5. **Strong Passwords**: Use strong, unique passwords for service accounts

## JWT Token Structure

Tokens include the following claims:
- `sub`: Subject (username)
- `sub_type`: "ServiceAccount" or "Subject"
- `namespace`: Optional namespace
- `exp`: Expiration time
- `iat`: Issued at time
- `iss`: Issuer ("raworc-rbac")

## API Endpoints

See the [REST API Documentation](rest-api.md) for detailed information on RBAC-related endpoints:
- `/auth/login` - Authenticate and get token
- `/service-accounts` - Manage service accounts
- `/roles` - Manage roles
- `/role-bindings` - Manage role bindings