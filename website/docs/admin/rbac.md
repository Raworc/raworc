---
sidebar_position: 2
title: RBAC System
---

# Role-Based Access Control (RBAC)

Raworc implements a Kubernetes-style RBAC system for fine-grained access control. This guide covers how to effectively use RBAC to secure your Raworc installation.

## Overview

The RBAC system consists of three main components:

1. **Service Accounts**: Authentication entities with credentials
2. **Roles**: Collections of permissions (what actions can be performed)
3. **Role Bindings**: Connections between roles and principals (who can perform actions)

## Core Concepts

### Service Accounts

Service accounts are the primary authentication mechanism in Raworc:

- Unique username within a namespace
- Password-based authentication
- Receive JWT tokens upon login
- Can be scoped to namespaces
- Support active/inactive states

Example service account:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "user": "deploy-bot",
  "namespace": "production",
  "description": "Deployment automation",
  "active": true
}
```

### Roles

Roles define permissions through rules. Each rule specifies:

- **API Groups**: Which APIs can be accessed
- **Resources**: Which resources can be accessed
- **Verbs**: Which actions can be performed

Example role:
```json
{
  "name": "agent-manager",
  "namespace": null,
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["agents"],
      "verbs": ["get", "list", "create", "update", "delete"]
    }
  ],
  "description": "Can manage agents"
}
```

### Role Bindings

Role bindings connect roles to principals:

- Bind a role to service accounts or external subjects
- Can be namespace-scoped or cluster-wide
- Support multiple principal types

Example binding:
```json
{
  "role_name": "agent-manager",
  "principal_name": "deploy-bot",
  "principal_type": "ServiceAccount",
  "namespace": null
}
```

## Permission Model

### API Groups

| API Group | Description | Resources |
|-----------|-------------|-----------|
| `""` | Core API group | Basic endpoints |
| `api` | Main application APIs | service-accounts, roles, agents |
| `rbac` | RBAC management APIs | role-bindings |
| `*` | All API groups | Admin access |

### Resources

| Resource | Description | API Group |
|----------|-------------|-----------|
| `service-accounts` | Service account management | `api` |
| `roles` | Role management | `api` |
| `role-bindings` | Role binding management | `rbac` |
| `agents` | Agent management | `api` |
| `*` | All resources | - |

### Verbs

| Verb | Description | HTTP Methods |
|------|-------------|--------------|
| `get` | Read a specific resource | GET |
| `list` | List resources | GET |
| `create` | Create new resources | POST |
| `update` | Update existing resources | PUT, PATCH |
| `delete` | Delete resources | DELETE |
| `*` | All verbs | All methods |

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
  }],
  "description": "Full cluster admin"
}
```

## Creating Custom Roles

### Read-Only Role

For users who need to view but not modify:
```bash
raworc> /api POST roles {
  "name": "viewer",
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["*"],
      "verbs": ["get", "list"]
    }
  ],
  "description": "Read-only access to all resources"
}
```

### Agent Operator

For users who manage agents but not RBAC:
```bash
raworc> /api POST roles {
  "name": "agent-operator",
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["agents"],
      "verbs": ["*"]
    },
    {
      "api_groups": ["api"],
      "resources": ["service-accounts", "roles"],
      "verbs": ["get", "list"]
    }
  ],
  "description": "Can manage agents and view RBAC"
}
```

### Namespace Admin

Admin within a specific namespace:
```bash
raworc> /api POST roles {
  "name": "namespace-admin",
  "namespace": "production",
  "rules": [
    {
      "api_groups": ["*"],
      "resources": ["*"],
      "verbs": ["*"]
    }
  ],
  "description": "Admin for production namespace"
}
```

## Common RBAC Patterns

### 1. Separation of Duties

Create separate roles for different responsibilities:

```bash
# Developer role - can manage agents
raworc> /api POST roles {
  "name": "developer",
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["agents"],
      "verbs": ["*"]
    }
  ]
}

# Security role - can manage RBAC
raworc> /api POST roles {
  "name": "security-admin",
  "rules": [
    {
      "api_groups": ["api", "rbac"],
      "resources": ["service-accounts", "roles", "role-bindings"],
      "verbs": ["*"]
    }
  ]
}
```

### 2. Progressive Permissions

Start with minimal permissions and add as needed:

```bash
# Level 1: Read-only
raworc> /api POST roles {
  "name": "junior-dev",
  "rules": [{
    "api_groups": ["api"],
    "resources": ["agents"],
    "verbs": ["get", "list"]
  }]
}

# Level 2: Read and create
raworc> /api POST roles {
  "name": "dev",
  "rules": [{
    "api_groups": ["api"],
    "resources": ["agents"],
    "verbs": ["get", "list", "create"]
  }]
}

# Level 3: Full agent management
raworc> /api POST roles {
  "name": "senior-dev",
  "rules": [{
    "api_groups": ["api"],
    "resources": ["agents"],
    "verbs": ["*"]
  }]
}
```

### 3. Service Account Per Application

Create dedicated accounts for each application:

```bash
# CI/CD Pipeline
raworc> /api POST service-accounts {
  "user": "ci-pipeline",
  "pass": "secure-pass",
  "description": "CI/CD deployment pipeline"
}

# Monitoring System
raworc> /api POST service-accounts {
  "user": "monitoring",
  "pass": "secure-pass",
  "description": "Metrics collection"
}

# Bind appropriate roles
raworc> /api POST role-bindings {
  "role_name": "agent-operator",
  "principal_name": "ci-pipeline",
  "principal_type": "ServiceAccount"
}
```

## Managing RBAC

### Listing Permissions

Check current RBAC setup:
```bash
# List all roles
raworc> /api roles

# List all bindings
raworc> /api role-bindings

# Check specific user's permissions
raworc> /api role-bindings | grep "username"
```

### Auditing Access

Regular auditing checklist:

1. **Review Admin Access**
   ```bash
   raworc> /api role-bindings | grep '"role_name":"admin"'
   ```

2. **Check Inactive Accounts**
   ```bash
   raworc> /api service-accounts | grep '"active":false'
   ```

3. **Verify Least Privilege**
   - Review each role's permissions
   - Ensure users have only necessary access
   - Remove unused bindings

### Rotating Credentials

Best practice for credential rotation:

```bash
# 1. Create new service account
raworc> /api POST service-accounts {
  "user": "app-v2",
  "pass": "new-secure-pass",
  "description": "Updated app credentials"
}

# 2. Copy role bindings
raworc> /api POST role-bindings {
  "role_name": "app-role",
  "principal_name": "app-v2",
  "principal_type": "ServiceAccount"
}

# 3. Update application config
# 4. Test new credentials
# 5. Delete old account
raworc> /api DELETE service-accounts/app-v1
```

## Security Best Practices

### 1. Principle of Least Privilege

- Grant only the minimum permissions required
- Start restrictive and add permissions as needed
- Regularly review and remove unnecessary permissions

### 2. Use Namespaces

- Isolate different environments (dev, staging, prod)
- Scope permissions to specific namespaces
- Prevent accidental cross-environment access

### 3. Strong Authentication

- Use strong, unique passwords for service accounts
- Rotate passwords regularly
- Never share service account credentials
- Use different accounts for different applications

### 4. Regular Audits

- Weekly: Review active sessions and recent changes
- Monthly: Audit all role bindings
- Quarterly: Review all roles and permissions
- Annually: Complete security review

### 5. Monitoring and Alerting

- Log all authentication attempts
- Alert on privilege escalation
- Monitor for unusual access patterns
- Track permission changes

## Troubleshooting

### Permission Denied Errors

1. **Check current user**:
   ```bash
   raworc> /api auth/me
   ```

2. **List user's role bindings**:
   ```bash
   raworc> /api role-bindings | grep "your-username"
   ```

3. **Verify role permissions**:
   ```bash
   raworc> /api roles/your-role-name
   ```

### Cannot Create Resources

Common issues:
- Missing verb in role (need `create`)
- Wrong API group specified
- Namespace mismatch
- Role binding not active

### Lost Admin Access

If all admin accounts are locked out:
1. Access database directly
2. Create emergency admin account
3. Review what caused the lockout
4. Implement safeguards

## RBAC Recipes

### Multi-Environment Setup

```bash
# Create namespace-specific roles
for env in dev staging prod; do
  raworc> /api POST roles {
    "name": "${env}-admin",
    "namespace": "${env}",
    "rules": [{
      "api_groups": ["*"],
      "resources": ["*"],
      "verbs": ["*"]
    }]
  }
done

# Create environment-specific accounts
for env in dev staging prod; do
  raworc> /api POST service-accounts {
    "user": "${env}-deployer",
    "pass": "secure-${env}-pass",
    "namespace": "${env}"
  }
done
```

### Read-Only Monitoring

```bash
# Create monitoring role
raworc> /api POST roles {
  "name": "monitoring",
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["*"],
      "verbs": ["get", "list"]
    }
  ]
}

# Create service account for each monitoring tool
for tool in prometheus datadog newrelic; do
  raworc> /api POST service-accounts {
    "user": "${tool}-collector",
    "pass": "secure-pass",
    "description": "${tool} metrics collector"
  }
  
  raworc> /api POST role-bindings {
    "role_name": "monitoring",
    "principal_name": "${tool}-collector",
    "principal_type": "ServiceAccount"
  }
done
```

## Future Enhancements

Planned RBAC features:
- External authentication providers (OIDC, SAML)
- Time-based access (temporary permissions)
- Attribute-based access control (ABAC)
- Permission inheritance
- Audit logging with compliance reports