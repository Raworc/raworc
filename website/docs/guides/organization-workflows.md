---
sidebar_position: 3
title: Organization Workflows
---

# Organization-Based Workflows

This guide demonstrates common workflows for managing resources across organizations in Raworc.

## Setting Up Organizations

### Single Organization Setup
For internal use or single-tenant deployments:

```bash
# All resources go to "default" organization
raworc> /api POST agents {"name":"helper","model":"gpt-4"}
# Implicitly creates in "default" namespace
```

### Multi-Organization Setup
For SaaS platforms with multiple customers:

```bash
# Create resources for different organizations
raworc> /api POST agents {"name":"support-bot","namespace":"acme-corp","model":"gpt-4"}
raworc> /api POST agents {"name":"dev-assistant","namespace":"tech-startup","model":"gpt-4"}
raworc> /api POST agents {"name":"analyzer","namespace":"consulting-llc","model":"gpt-4"}
```

## User Access Patterns

### Organization Administrator
Give a user full control of their organization:

```bash
# Make alice the admin of Acme Corp
raworc> /api POST role-bindings {
  "role_name": "admin",
  "principal_name": "alice",
  "principal_type": "ServiceAccount",
  "namespace": "acme-corp"
}

# Alice can now manage all Acme Corp resources
raworc> /api agents?namespace=acme-corp  # See all Acme agents
raworc> /api sessions?namespace=acme-corp  # See all Acme sessions
```

### Multi-Organization Consultant
Grant access to multiple organizations with different roles:

```bash
# Bob is a consultant working with multiple clients
raworc> /api POST role-bindings {
  "role_name": "developer",
  "principal_name": "bob",
  "namespace": "client-a"
}

raworc> /api POST role-bindings {
  "role_name": "developer",
  "principal_name": "bob",
  "namespace": "client-b"
}

raworc> /api POST role-bindings {
  "role_name": "viewer",
  "principal_name": "bob",
  "namespace": "client-c"
}

# Bob can develop for Client A and B, view Client C
```

### Platform Administrator
Grant global access for platform management:

```bash
# Platform admin can manage all organizations
raworc> /api POST role-bindings {
  "role_name": "cluster-admin",
  "principal_name": "platform-admin",
  "principal_type": "ServiceAccount",
  "namespace": null  # null = global access
}

# Platform admin sees everything
raworc> /api agents  # All agents from all orgs
raworc> /api sessions  # All sessions from all orgs
```

## Resource Management

### Creating Organization Resources

```bash
# Create agent for Acme Corp
raworc> /api POST agents {
  "name": "acme-assistant",
  "namespace": "acme-corp",
  "model": "gpt-4",
  "instructions": "You are Acme Corp's helpful assistant",
  "description": "Customer support bot for Acme Corp"
}

# Create session for Tech Startup
raworc> /api POST sessions {
  "name": "code-review-session",
  "namespace": "tech-startup",
  "starting_prompt": "Review the latest PR",
  "agent_ids": ["agent-uuid-here"]
}
```

### Listing Organization Resources

```bash
# List all accessible resources (based on your role bindings)
raworc> /api agents
raworc> /api sessions

# List resources for specific organization (if authorized)
raworc> /api agents?namespace=acme-corp
raworc> /api sessions?namespace=tech-startup

# Filter within organization
raworc> /api sessions?namespace=acme-corp&lifecycle_state=STARTED
```

### Cross-Organization Operations

```bash
# Platform admin can specify any namespace
raworc> /api POST agents {"name":"shared-tool","namespace":"acme-corp",...}
raworc> /api POST agents {"name":"shared-tool","namespace":"tech-startup",...}

# Regular users are restricted to their authorized namespaces
# This will fail if user doesn't have role in other-org:
raworc> /api POST agents {"name":"test","namespace":"other-org",...}
# Error: Forbidden - no access to namespace "other-org"
```

## Common Scenarios

### Onboarding a New Customer Organization

```bash
# 1. Create organization admin account
raworc> /api POST service-accounts {
  "user": "acme-admin",
  "pass": "secure-password",
  "description": "Acme Corp administrator"
}

# 2. Grant admin role for their organization
raworc> /api POST role-bindings {
  "role_name": "org-admin",
  "principal_name": "acme-admin",
  "principal_type": "ServiceAccount",
  "namespace": "acme-corp"
}

# 3. Create initial resources
raworc> /api POST agents {
  "name": "welcome-bot",
  "namespace": "acme-corp",
  "model": "gpt-4",
  "instructions": "Welcome to Acme Corp!"
}
```

### Organization Isolation Verification

```bash
# As acme-admin (only has access to acme-corp)
raworc> /api agents
# Returns only Acme Corp agents

raworc> /api agents?namespace=tech-startup
# Error: Forbidden - no access to namespace "tech-startup"

raworc> /api POST agents {"name":"test","namespace":"tech-startup",...}
# Error: Forbidden - cannot create resources in "tech-startup"
```

### Automation Service Account

```bash
# Create service account for CI/CD
raworc> /api POST service-accounts {
  "user": "acme-ci-bot",
  "pass": "token-here",
  "description": "Acme Corp CI/CD automation"
}

# Grant limited permissions in organization
raworc> /api POST role-bindings {
  "role_name": "deployer",
  "principal_name": "acme-ci-bot",
  "principal_type": "ServiceAccount",
  "namespace": "acme-corp"
}

# Bot can now deploy to acme-corp namespace
```

## Best Practices

### Namespace Naming Conventions
- Use organization slugs: `acme-corp`, `tech-startup`
- Avoid special characters except hyphens
- Keep names short but descriptive
- Consider using domains: `acme-com`, `startup-io`

### Access Control
- Start with least privilege
- Use organization-specific bindings over global
- Regularly audit global role bindings
- Document role purposes

### Resource Organization
- Prefix resource names with purpose: `support-agent`, `dev-assistant`
- Use consistent naming within organizations
- Tag resources with metadata for filtering

### Multi-Tenancy Considerations
- Never share resources between organizations
- Validate namespace access on every operation
- Log cross-organization access attempts
- Implement namespace quotas (future feature)

## Migration from Old Model

If you're migrating from the old model where service accounts were namespaced:

```bash
# Old way (WRONG - namespaced users)
POST /service-accounts {"user":"alice","namespace":"prod"}

# New way (CORRECT - global users, namespaced resources)  
POST /service-accounts {"user":"alice"}  # Global user
POST /role-bindings {  # Grant access to organization
  "role_name":"developer",
  "principal_name":"alice",
  "namespace":"acme-corp"  # Organization access
}
POST /agents {"name":"bot","namespace":"acme-corp"}  # Org resource
```

The new model properly isolates organization resources while allowing users to work across multiple organizations.