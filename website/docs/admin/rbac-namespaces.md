---
sidebar_position: 3
title: RBAC and Namespaces
---

# RBAC and Organizations (Namespaces)

## Overview

Raworc implements organization-based multi-tenancy where:
- **Resources** (agents, sessions) belong to organizations (via namespaces)
- **Users and roles** are global (can work across organizations)
- **Role bindings** determine which users can access which organizations

## Key Concepts

### Organizations (Namespaces)
Namespaces represent organizations and provide complete resource isolation:
- Each agent belongs to exactly one organization
- Each session belongs to exactly one organization
- Organizations cannot see or access each other's resources
- Examples: "acme-corp", "tech-startup", "consulting-llc"

### Global RBAC Entities

#### Service Accounts (Users)
- Service accounts are **global** (not tied to organizations)
- One user can work with multiple organizations
- Example: `john.doe` can be an admin at Acme Corp and consultant at Tech Startup

#### Roles
- Roles are **global** permission templates
- Same role can be used across multiple organizations
- Example: `developer` role has same meaning at all organizations

### Role Bindings Define Access

Role bindings connect users to roles and specify **where** those roles apply:

#### Global Binding (All Namespaces)
```json
{
  "role_name": "cluster-admin",
  "principal_name": "admin",
  "principal_type": "ServiceAccount",
  "namespace": null  // null = applies everywhere
}
```

#### Namespace-Specific Binding
```json
{
  "role_name": "developer",
  "principal_name": "alice",
  "principal_type": "ServiceAccount",
  "namespace": "production"  // only in production
}
```

## Access Patterns

### Working with Multiple Organizations
A single user can have different roles at different organizations:

```yaml
User: alice (Consultant)
Bindings:
  - {role: admin, namespace: startup-a}
  - {role: developer, namespace: enterprise-b}
  - {role: viewer, namespace: nonprofit-c}

Result:
  - Full control at Startup A
  - Developer access at Enterprise B
  - Read-only access at Nonprofit C
  - No access to other organizations
```

### Platform Administrator
```yaml
User: platform-admin (SaaS Operator)
Bindings:
  - {role: cluster-admin, namespace: null}

Result:
  - Full access to all customer organizations
  - Can create resources in any organization
  - Can manage customer organization access
  - Typically used by SaaS platform operators
```

## Resource Operations

### Listing Resources
When listing resources, users see a filtered view based on their namespace access:

```bash
GET /api/v0/agents
# Returns agents from all organizations where user has permissions

GET /api/v0/agents?namespace=acme-corp  
# Returns agents from Acme Corp (if authorized)
```

### Creating Resources
Resources are created in a specific namespace:

```json
POST /api/v0/agents
{
  "name": "my-agent",
  "namespace": "acme-corp",  // Create in Acme Corp's organization
  ...
}
```

If namespace is omitted, uses the default organization (usually "default").

### Cross-Namespace Operations

Users can only perform operations in namespaces where they have appropriate role bindings:
- Global bindings (`namespace: null`) allow operations in any namespace
- Namespace-specific bindings restrict operations to that namespace

## Common Scenarios

### Multi-Organization Consultant
```json
// Consultant works with multiple client organizations
[
  {
    "role_name": "developer",
    "principal_name": "consultant",
    "namespace": "client-a"
  },
  {
    "role_name": "developer", 
    "principal_name": "consultant",
    "namespace": "client-b"
  }
]
```

### Organization Admin
```json
// Company admin manages their organization's resources
{
  "role_name": "org-admin",
  "principal_name": "cto",
  "namespace": "acme-corp"
}
```

### Service Account for Organization Automation
```json
// Organization's CI/CD bot
{
  "role_name": "automation",
  "principal_name": "acme-bot",
  "namespace": "acme-corp"
}
```

## Permission Evaluation

When a user attempts an operation:

1. **Authentication**: Verify the service account credentials
2. **Find Bindings**: Get all role bindings for this principal
3. **Check Namespace**: 
   - If binding namespace is `null` → applies to requested namespace
   - If binding namespace matches requested → applies
   - Otherwise → doesn't apply
4. **Evaluate Permissions**: Check if any applicable role grants the permission

## Best Practices

### Organization Design
- Each customer/client gets their own namespace
- Use namespace naming convention (e.g., company slug)
- For internal use, can have single "default" organization
- For SaaS, each tenant is a namespace

### Role Design
- Create reusable, well-defined roles
- Avoid overly broad permissions
- Use principle of least privilege
- Document what each role is intended for

### Binding Management
- Prefer namespace-specific bindings over global
- Regularly audit global bindings
- Use global bindings sparingly (only for true admins)
- Consider time-limited bindings for temporary access

## CLI Examples

### Grant Organization Access
```bash
# Give alice developer role at Tech Startup
raworc> /api POST role-bindings {
  "role_name": "developer",
  "principal_name": "alice",
  "principal_type": "ServiceAccount",
  "namespace": "tech-startup"
}
```

### Grant Global Access
```bash
# Give admin cluster-wide access
raworc> /api POST role-bindings {
  "role_name": "cluster-admin",
  "principal_name": "admin",
  "principal_type": "ServiceAccount",
  "namespace": null
}
```

### List Organization Resources
```bash
# List agents at Acme Corp
raworc> /api agents?namespace=acme-corp

# List sessions at Tech Startup  
raworc> /api sessions?namespace=tech-startup
```

### Create Organization Resource
```bash
# Create agent for Acme Corp
raworc> /api POST agents {
  "name": "support-agent",
  "namespace": "acme-corp",
  "model": "gpt-4",
  "instructions": "..."
}
```

## Migration Notes

If migrating from an older version where RBAC entities were namespaced:
- Service accounts are now global (namespace removed)
- Roles are now global (namespace removed)
- Role bindings namespace field now means "where binding applies"
- Resources (agents, sessions) now have namespace field
- Default namespace is "default" for backward compatibility