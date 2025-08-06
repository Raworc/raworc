---
sidebar_position: 1
title: CLI Examples
---

# Raworc CLI API Examples

The enhanced CLI now supports full REST operations with JSON data.

## Basic Syntax

```
/api <endpoint>                    # GET request (shorthand)
/api <METHOD> <endpoint> [json]    # Full syntax
```

## Examples

### GET Requests
```bash
# Shorthand (assumes GET)
raworc> /api version
raworc> /api service-accounts
raworc> /api roles/admin

# Explicit method
raworc> /api GET auth/me
raworc> /api GET role-bindings
```

### POST Requests
```bash
# Create a role
raworc> /api POST roles {"name":"viewer","rules":[{"api_groups":["api"],"resources":["*"],"verbs":["get","list"]}]}

# Create a service account
raworc> /api POST service-accounts {"user":"bot-user","pass":"secure123","description":"Automation bot"}

# Create a role binding
raworc> /api POST role-bindings {"role_name":"viewer","principal_name":"bot-user","principal_type":"ServiceAccount"}
```

### PUT Requests
```bash
# Update a service account description
raworc> /api PUT service-accounts/bot-user {"description":"Updated automation bot"}
```

### DELETE Requests
```bash
# Delete a role
raworc> /api DELETE roles/viewer

# Delete a service account
raworc> /api DELETE service-accounts/bot-user

# Delete a role binding
raworc> /api DELETE role-bindings/some-binding-id
```

### Agent Management
```bash
# List all agents
raworc> /api agents

# Get specific agent by name or ID
raworc> /api agents/assistant

# Create a new agent
raworc> /api POST agents {"name":"code-assistant","instructions":"You are a helpful coding assistant","model":"gpt-4","description":"Helps with code reviews and debugging"}

# Update an agent
raworc> /api PUT agents/code-assistant {"instructions":"You are an expert code reviewer focused on security","model":"gpt-4-turbo"}

# Delete an agent
raworc> /api DELETE agents/code-assistant
```

### Session Management
```bash
# List all sessions
raworc> /api sessions

# List sessions with filters
raworc> /api sessions?lifecycle_state=STARTED
raworc> /api sessions?created_by=john.doe

# Get specific session
raworc> /api sessions/61549530-3095-4cbf-b379-cd32416f626d

# Create a new session
raworc> /api POST sessions {"name":"data-analysis","starting_prompt":"Analyze Q4 sales data","agent_ids":["550e8400-e29b-41d4-a716-446655440000"],"waiting_timeout_seconds":300}

# Update session details
raworc> /api PUT sessions/61549530-3095-4cbf-b379-cd32416f626d {"name":"Q4 Analysis - Updated","metadata":{"priority":"high"}}

# Update session state
raworc> /api PUT sessions/61549530-3095-4cbf-b379-cd32416f626d/state {"lifecycle_state":"STARTED","container_id":"k8s-pod-abc123"}

# Remix a session (create new from existing)
raworc> /api POST sessions/61549530-3095-4cbf-b379-cd32416f626d/remix {"name":"data-analysis-continued","starting_prompt":"Focus on trend analysis"}

# Delete a session
raworc> /api DELETE sessions/61549530-3095-4cbf-b379-cd32416f626d
```

### Service Account Password Management
```bash
# Change password for current user
raworc> /api PUT service-accounts/current-user/password {"current_password":"oldPass123","new_password":"newSecurePass456"}

# Admin changing another user's password
raworc> /api PUT service-accounts/bot-user/password {"new_password":"newBotPassword789"}
```

## Response Format

All commands show:
1. The HTTP method and endpoint
2. The response status code
3. Pretty-printed JSON response (if applicable)

Example:
```
raworc> /api POST roles {"name":"test","rules":[]}
 POST roles → 200 OK
 Response:
  {
    "id": "abc-123",
    "name": "test",
    "rules": [],
    "created_at": "2025-01-01T00:00:00Z"
  }
```