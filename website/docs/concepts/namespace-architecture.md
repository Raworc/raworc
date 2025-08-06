---
sidebar_position: 4
title: Namespace Architecture
---

# Namespace Architecture

## Overview

Raworc uses namespaces to provide multi-tenant resource isolation by organization. Namespaces represent **organizations** (companies, teams, or business units) and segregate their **resources** (agents, sessions) while **RBAC entities** (service accounts, roles) are global.

## Design Principles

### 1. Resources Belong to Organizations (Namespaces)
- **Agents** belong to organizations via namespaces
- **Sessions** belong to organizations via namespaces
- Future resources will belong to organizations
- Each organization's resources are completely isolated

### 2. RBAC Entities are Global
- **Service Accounts** are global (not namespaced)
- **Roles** are global (define permissions that can be used anywhere)
- **Role Bindings** connect roles to users and specify WHERE they apply

### 3. Role Bindings Define Namespace Scope

Role bindings can be:

#### Global Bindings
```json
{
  "role": "cluster-admin",
  "principal": "admin",
  "principal_type": "ServiceAccount",
  "namespace": null  // Applies to ALL namespaces
}
```

#### Namespace-Specific Bindings
```json
{
  "role": "editor",
  "principal": "john.doe",
  "principal_type": "ServiceAccount", 
  "namespace": "production"  // Only applies to production namespace
}
```

## How It Works

### Resource Access
1. User authenticates (global service account)
2. System checks role bindings for that user
3. User can only see/modify resources in namespaces where they have roles
4. Global bindings (namespace=null) grant access to all namespaces

### Example Scenarios

**Multi-Organization Access:**
```yaml
User: john.doe
Bindings:
  - role: admin, namespace: acme-corp
  - role: viewer, namespace: tech-startup
  - role: developer, namespace: consulting-llc
  
Result:
  - Full access to Acme Corp's resources
  - Read-only access to Tech Startup's resources
  - Developer access to Consulting LLC's resources
  - No access to other organizations
```

**Platform Administrator:**
```yaml
User: platform-admin
Bindings:
  - role: cluster-admin, namespace: null
  
Result:
  - Full access to ALL organizations' resources
  - Can create/manage resources in any organization
  - Typically reserved for SaaS platform operators
```

## API Behavior

### Resource Endpoints

**List Resources:**
- `GET /api/v0/agents` - Returns agents from accessible namespaces
- `GET /api/v0/agents?namespace=production` - Filter to specific namespace (if authorized)

**Create Resources:**
```json
POST /api/v0/agents
{
  "name": "helper",
  "namespace": "acme-corp",  // Organization that will own this agent
  ...
}
```

**Cross-Namespace Operations:**
- Users with global roles can specify any namespace
- Users with namespace-specific roles are restricted to their namespaces

### Role Binding Examples

**Grant organization-specific access:**
```json
POST /api/v0/role-bindings
{
  "role_name": "developer",
  "principal_name": "alice",
  "principal_type": "ServiceAccount",
  "namespace": "tech-startup"  // Alice gets developer role at Tech Startup
}
```

**Grant global access:**
```json
POST /api/v0/role-bindings
{
  "role_name": "cluster-admin",
  "principal_name": "admin",
  "principal_type": "ServiceAccount",
  "namespace": null  // Admin gets cluster-admin role everywhere
}
```

## Default Organization

- If no namespace specified when creating resources → uses "default"
- New users initially have no organization access until role binding created
- For single-organization deployments, can use "default" as the organization
- For multi-tenant SaaS, each customer gets their own namespace (e.g., "acme-corp", "tech-startup")

## Benefits

1. **True Multi-Tenancy**: Each organization's resources are completely isolated
2. **Flexible Permissions**: Consultants/contractors can work with multiple organizations
3. **Platform Administration**: SaaS operators can manage all customer organizations
4. **Clean Separation**: Users and roles are global, organization resources are isolated
5. **Enterprise Ready**: Natural mapping to real-world organizational boundaries

## Migration from Old Model

The old model had it backwards:
- ❌ Old: Service accounts, roles were namespaced (wrong)
- ✅ New: Resources (agents, sessions) are namespaced (correct)

This change enables proper resource isolation while maintaining flexible RBAC.