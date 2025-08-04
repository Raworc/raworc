---
sidebar_position: 1
title: CLI Examples
---

# CLI Examples

The Raworc CLI provides an interactive interface for managing the platform. This guide shows common usage patterns and examples.

## Basic Commands

### Starting the Server

```bash
# Start in foreground mode (recommended for development)
raworc start

# Start as daemon (Unix/Linux only)
raworc serve

# Stop daemon
raworc stop

# Check authentication status
raworc status
```

## Authentication

### Initial Setup

```bash
raworc auth
```

You'll be prompted to choose:
1. Login with service account (username/password)
2. Store JWT token directly

Example session:
```
Raworc Authentication

Choose authentication method:
1. Login with service account
2. Store JWT token directly

Enter choice (1 or 2): 1
Service Account Login
Server URL: http://localhost:9000
Username: admin
Password: ********
Authenticating...

✓ Authentication successful!
   User: admin
   Server: http://localhost:9000

You can now use 'raworc' to connect to this server.
```

### Checking Status

```bash
raworc status
```

Output:
```
Authentication Status:
  ✓ Logged in as: admin (http://localhost:9000)
```

## Interactive Mode

### Connecting to Server

```bash
raworc connect
# or just
raworc
```

You'll see:
```
╭──────────────────────────────────────────────────╮
│ ❋ Welcome to Raworc!                             │
│                                                  │
│   Remote Agent Work Orchestration                │
│                                                  │
│   Type /help for commands, /quit or q to exit    │
╰──────────────────────────────────────────────────╯

  ✓ Logged in as: admin (http://localhost:9000)

raworc>
```

### Available Commands

```bash
raworc> /help
```

Output:
```
Available Commands:

  /api <METHOD> <endpoint> [json]  - Execute REST API request
  /api <endpoint>                  - Execute GET request (shorthand)
  /status                          - Show server status
  /help                            - Show this help
  /quit, /q, q, quit, exit         - Exit interactive mode

Examples:
  /api version                     - GET /api/v0/version
  /api service-accounts            - GET /api/v0/service-accounts
  /api GET roles                   - GET /api/v0/roles
  /api POST roles {"name":"test","rules":[]}
  /api DELETE roles/test-role
  /api PUT service-accounts/admin {"description":"Updated"}
```

## API Examples

### GET Requests

#### Simple GET (shorthand)
```bash
raworc> /api version
```
Output:
```
GET version → 200 OK
Response:
  {
    "version": "0.1.0",
    "api": "v0"
  }
```

#### List Resources
```bash
raworc> /api service-accounts
```
Output:
```
GET service-accounts → 200 OK
Response:
  [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user": "admin",
      "namespace": null,
      "description": "Administrator account",
      "active": true,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ]
```

#### Get Specific Resource
```bash
raworc> /api roles/admin
```

### POST Requests

#### Create a Role
```bash
raworc> /api POST roles {"name":"viewer","rules":[{"api_groups":["api"],"resources":["*"],"verbs":["get","list"]}],"description":"Read-only access"}
```
Output:
```
POST roles → 200 OK
Response:
  {
    "id": "abc-123",
    "name": "viewer",
    "namespace": null,
    "rules": [
      {
        "api_groups": ["api"],
        "resources": ["*"],
        "verbs": ["get", "list"]
      }
    ],
    "description": "Read-only access",
    "created_at": "2025-01-01T00:00:00Z"
  }
```

#### Create a Service Account
```bash
raworc> /api POST service-accounts {"user":"bot-user","pass":"secure123","description":"Automation bot"}
```

#### Create a Role Binding
```bash
raworc> /api POST role-bindings {"role_name":"viewer","principal_name":"bot-user","principal_type":"ServiceAccount"}
```

### PUT Requests

#### Update Service Account
```bash
raworc> /api PUT service-accounts/bot-user {"description":"Updated automation bot"}
```

#### Update Agent
```bash
raworc> /api PUT agents/assistant {"instructions":"You are a helpful assistant focused on data analysis","model":"gpt-4-turbo"}
```

### DELETE Requests

#### Delete a Role
```bash
raworc> /api DELETE roles/viewer
```
Output:
```
DELETE roles/viewer → 200 OK
```

#### Delete a Service Account
```bash
raworc> /api DELETE service-accounts/bot-user
```

## Agent Management

### List Agents
```bash
raworc> /api agents
```

### Create an Agent
```bash
raworc> /api POST agents {
  "name": "code-assistant",
  "description": "Helps with code reviews and debugging",
  "instructions": "You are a helpful coding assistant",
  "model": "gpt-4"
}
```

### Update an Agent
```bash
raworc> /api PUT agents/code-assistant {
  "instructions": "You are an expert code reviewer focused on security",
  "model": "gpt-4-turbo",
  "tools": ["static-analysis"],
  "guardrails": ["no-secrets"]
}
```

### Delete an Agent
```bash
raworc> /api DELETE agents/code-assistant
```

## RBAC Management

### Creating a Complete RBAC Setup

1. **Create a role with specific permissions**:
```bash
raworc> /api POST roles {
  "name": "agent-manager",
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["agents"],
      "verbs": ["*"]
    }
  ],
  "description": "Can manage agents"
}
```

2. **Create a service account**:
```bash
raworc> /api POST service-accounts {
  "user": "agent-admin",
  "pass": "SecurePass123!",
  "description": "Agent administrator"
}
```

3. **Bind the role to the account**:
```bash
raworc> /api POST role-bindings {
  "role_name": "agent-manager",
  "principal_name": "agent-admin",
  "principal_type": "ServiceAccount"
}
```

### Namespace-Scoped Permissions

Create namespace-specific roles:
```bash
raworc> /api POST roles {
  "name": "prod-admin",
  "namespace": "production",
  "rules": [
    {
      "api_groups": ["*"],
      "resources": ["*"],
      "verbs": ["*"]
    }
  ],
  "description": "Production namespace admin"
}
```

## Common Workflows

### Setting Up a New User

```bash
# 1. Create the service account
raworc> /api POST service-accounts {"user":"developer","pass":"DevPass123!","description":"Developer account"}

# 2. Create or use existing role
raworc> /api POST roles {"name":"developer","rules":[{"api_groups":["api"],"resources":["agents","service-accounts"],"verbs":["get","list"]}]}

# 3. Bind role to account
raworc> /api POST role-bindings {"role_name":"developer","principal_name":"developer","principal_type":"ServiceAccount"}
```

### Auditing Permissions

```bash
# Check who has admin access
raworc> /api role-bindings | grep admin

# List all roles
raworc> /api roles

# Check specific user's bindings
raworc> /api GET role-bindings?principal_name=developer
```

## Tips and Tricks

### Command History
- Use arrow keys to navigate command history
- Commands are saved between sessions

### Auto-completion
- Tab completion works for commands
- Start typing `/` to see available commands

### Quick Exit
- Type `q`, `quit`, or `exit` to leave
- `/q` or `/quit` also work
- Ctrl+C for emergency exit

### JSON Formatting
- JSON is automatically pretty-printed
- Use proper JSON syntax (double quotes)
- Escape special characters as needed

### Error Handling
- Check HTTP status codes
- Read error messages for details
- Use `/status` to verify connection

## Troubleshooting

### Authentication Issues
```bash
# Re-authenticate
raworc auth

# Check current status
raworc status
```

### Connection Problems
```bash
# Verify server is running
curl http://localhost:9000/api/v0/health

# Check logs
tail -f logs/raworc.log
```

### Permission Denied
```bash
# Check current user
raworc> /api auth/me

# List role bindings for user
raworc> /api role-bindings
```

## Advanced Usage

### Scripting with Raworc

Create a script file:
```bash
#!/bin/bash
# setup-env.sh

# Create roles
echo '/api POST roles {"name":"dev-role","rules":[{"api_groups":["api"],"resources":["agents"],"verbs":["*"]}]}' | raworc connect

# Create accounts
echo '/api POST service-accounts {"user":"dev1","pass":"pass123"}' | raworc connect

# Bind roles
echo '/api POST role-bindings {"role_name":"dev-role","principal_name":"dev1","principal_type":"ServiceAccount"}' | raworc connect
```

### Using with jq

```bash
# Get all agent names
raworc connect <<< '/api agents' | jq -r '.[].name'

# Filter active agents
raworc connect <<< '/api agents' | jq '.[] | select(.active == true)'
```

### Environment Variables

```bash
# Set custom server
export RAWORC_SERVER=http://production:9000
raworc auth

# Use different config
export RAWORC_CONFIG=/path/to/config
raworc connect
```