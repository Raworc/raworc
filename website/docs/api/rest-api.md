---
sidebar_position: 2
title: REST API Reference
---

# REST API Reference

Complete reference for all Raworc REST API endpoints.

## Overview

- **Base URL**: `/api/v0`
- **Authentication**: Bearer token (JWT)
- **Content-Type**: `application/json`

## Health & Version

### GET /health

Check server health status.

**Authentication**: Not required

**Response**: `200 OK`
```text
OK
```

### GET /version

Get API version information.

**Authentication**: Not required

**Response**: `200 OK`
```json
{
  "version": "0.1.0",
  "api": "v0"
}
```

## Authentication

### POST /auth/internal

Authenticate a service account and receive a JWT token.

**Authentication**: Not required

**Request Body**:
```json
{
  "user": "admin",
  "pass": "password123",
  "namespace": null
}
```

**Response**: `200 OK`
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_at": "2025-01-02T12:00:00Z"
}
```

**Errors**:
- `401 Unauthorized` - Invalid credentials

### GET /auth/me

Get information about the authenticated user.

**Authentication**: Required

**Response**: `200 OK`
```json
{
  "user": "admin",
  "namespace": null,
  "type": "ServiceAccount"
}
```

## Service Accounts

### GET /service-accounts

List all service accounts.

**Authentication**: Required  
**Permissions**: `get`, `list` on `service-accounts`

**Response**: `200 OK`
```json
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

### POST /service-accounts

Create a new service account.

**Authentication**: Required  
**Permissions**: `create` on `service-accounts`

**Request Body**:
```json
{
  "user": "deploy-bot",
  "pass": "SecurePass123!",
  "namespace": "production",
  "description": "Deployment automation bot"
}
```

**Response**: `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "user": "deploy-bot",
  "namespace": "production",
  "description": "Deployment automation bot",
  "active": true,
  "created_at": "2025-01-01T00:00:00Z"
}
```

**Errors**:
- `400 Bad Request` - Invalid input
- `409 Conflict` - Account already exists

### GET /service-accounts/{id}

Get a specific service account by ID or username.

**Authentication**: Required  
**Permissions**: `get` on `service-accounts`

**Parameters**:
- `id` (path) - Service account ID or username

**Response**: `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "user": "admin",
  "namespace": null,
  "description": "Administrator account",
  "active": true,
  "created_at": "2025-01-01T00:00:00Z"
}
```

**Errors**:
- `404 Not Found` - Account not found

### DELETE /service-accounts/{id}

Delete a service account by ID or username.

**Authentication**: Required  
**Permissions**: `delete` on `service-accounts`

**Parameters**:
- `id` (path) - Service account ID or username

**Response**: `200 OK`

**Errors**:
- `404 Not Found` - Account not found

## Roles

### GET /roles

List all roles.

**Authentication**: Required  
**Permissions**: `get`, `list` on `roles`

**Response**: `200 OK`
```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "admin",
    "namespace": null,
    "rules": [
      {
        "api_groups": ["*"],
        "resources": ["*"],
        "verbs": ["*"]
      }
    ],
    "description": "Full cluster admin",
    "created_at": "2025-01-01T00:00:00Z"
  }
]
```

### POST /roles

Create a new role.

**Authentication**: Required  
**Permissions**: `create` on `roles`

**Request Body**:
```json
{
  "name": "reader",
  "namespace": null,
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["service-accounts", "roles"],
      "verbs": ["get", "list"]
    }
  ],
  "description": "Read-only access to accounts and roles"
}
```

**Response**: `200 OK`
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174001",
  "name": "reader",
  "namespace": null,
  "rules": [
    {
      "api_groups": ["api"],
      "resources": ["service-accounts", "roles"],
      "verbs": ["get", "list"]
    }
  ],
  "description": "Read-only access to accounts and roles",
  "created_at": "2025-01-01T00:00:00Z"
}
```

**Errors**:
- `400 Bad Request` - Invalid input
- `409 Conflict` - Role already exists

### GET /roles/{id}

Get a specific role by ID or name.

**Authentication**: Required  
**Permissions**: `get` on `roles`

**Parameters**:
- `id` (path) - Role ID or name

**Response**: `200 OK`
(Same format as POST response)

**Errors**:
- `404 Not Found` - Role not found

### DELETE /roles/{id}

Delete a role by ID or name.

**Authentication**: Required  
**Permissions**: `delete` on `roles`

**Parameters**:
- `id` (path) - Role ID or name

**Response**: `200 OK`

**Errors**:
- `404 Not Found` - Role not found

## Role Bindings

### GET /role-bindings

List all role bindings.

**Authentication**: Required  
**Permissions**: `get`, `list` on `role-bindings`

**Response**: `200 OK`
```json
[
  {
    "id": "987f6543-a21b-98c7-d654-321098765432",
    "namespace": null,
    "role_name": "admin",
    "principal_name": "admin",
    "principal_type": "ServiceAccount",
    "created_at": "2025-01-01T00:00:00Z"
  }
]
```

### POST /role-bindings

Create a new role binding.

**Authentication**: Required  
**Permissions**: `create` on `role-bindings`

**Request Body**:
```json
{
  "role_name": "reader",
  "principal_name": "deploy-bot",
  "principal_type": "ServiceAccount",
  "namespace": null
}
```

**Response**: `200 OK`
```json
{
  "id": "987f6543-a21b-98c7-d654-321098765433",
  "namespace": null,
  "role_name": "reader",
  "principal_name": "deploy-bot",
  "principal_type": "ServiceAccount",
  "created_at": "2025-01-01T00:00:00Z"
}
```

**Errors**:
- `400 Bad Request` - Invalid input
- `404 Not Found` - Role or principal not found
- `409 Conflict` - Binding already exists

### GET /role-bindings/{id}

Get a specific role binding by ID.

**Authentication**: Required  
**Permissions**: `get` on `role-bindings`

**Parameters**:
- `id` (path) - Role binding ID

**Response**: `200 OK`
(Same format as POST response)

**Errors**:
- `404 Not Found` - Binding not found

### DELETE /role-bindings/{id}

Delete a role binding by ID.

**Authentication**: Required  
**Permissions**: `delete` on `role-bindings`

**Parameters**:
- `id` (path) - Role binding ID

**Response**: `200 OK`

**Errors**:
- `404 Not Found` - Binding not found

## Agents

### GET /agents

List all active agents.

**Authentication**: Required  
**Permissions**: `get`, `list` on `agents`

**Response**: `200 OK`
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "assistant",
    "description": "General purpose assistant agent",
    "instructions": "You are a helpful AI assistant. Be concise and accurate in your responses.",
    "model": "gpt-4",
    "tools": [],
    "routes": [],
    "guardrails": [],
    "knowledge_bases": [],
    "active": true,
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z"
  }
]
```

### POST /agents

Create a new agent.

**Authentication**: Required  
**Permissions**: `create` on `agents`

**Request Body**:
```json
{
  "name": "code-reviewer",
  "description": "Code review specialist",
  "instructions": "You are an expert code reviewer. Focus on security, performance, and best practices.",
  "model": "gpt-4-turbo",
  "tools": ["static-analysis", "security-scan"],
  "routes": [{"pattern": "*.py", "weight": 1.0}],
  "guardrails": ["no-secrets", "no-pii"],
  "knowledge_bases": ["python-best-practices"]
}
```

**Response**: `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "name": "code-reviewer",
  "description": "Code review specialist",
  "instructions": "You are an expert code reviewer. Focus on security, performance, and best practices.",
  "model": "gpt-4-turbo",
  "tools": ["static-analysis", "security-scan"],
  "routes": [{"pattern": "*.py", "weight": 1.0}],
  "guardrails": ["no-secrets", "no-pii"],
  "knowledge_bases": ["python-best-practices"],
  "active": true,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z"
}
```

**Errors**:
- `400 Bad Request` - Invalid input
- `409 Conflict` - Agent name already exists

### GET /agents/{id}

Get a specific agent by ID or name.

**Authentication**: Required  
**Permissions**: `get` on `agents`

**Parameters**:
- `id` (path) - Agent ID or name

**Response**: `200 OK`
(Same format as POST response)

**Errors**:
- `404 Not Found` - Agent not found

### PUT /agents/{id}

Update an agent by ID.

**Authentication**: Required  
**Permissions**: `update` on `agents`

**Parameters**:
- `id` (path) - Agent ID

**Request Body** (partial update supported):
```json
{
  "instructions": "Updated instructions for the agent",
  "model": "gpt-4-turbo",
  "active": true
}
```

**Response**: `200 OK`
(Returns updated agent)

**Errors**:
- `400 Bad Request` - Invalid input
- `404 Not Found` - Agent not found
- `409 Conflict` - Name already taken

### DELETE /agents/{id}

Delete (soft delete) an agent by ID.

**Authentication**: Required  
**Permissions**: `delete` on `agents`

**Parameters**:
- `id` (path) - Agent ID

**Response**: `204 No Content`

**Errors**:
- `404 Not Found` - Agent not found

## Error Responses

All errors follow a consistent format:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Role not found"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `BAD_REQUEST` | 400 | Invalid request data |
| `UNAUTHORIZED` | 401 | Missing or invalid token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource already exists |
| `UNPROCESSABLE_ENTITY` | 422 | Validation error |
| `INTERNAL_ERROR` | 500 | Server error |

## Rate Limiting

Currently not implemented, but reserved headers for future use:
- `X-RateLimit-Limit`: Request limit
- `X-RateLimit-Remaining`: Requests remaining
- `X-RateLimit-Reset`: Reset timestamp

## API Versioning

The API uses URL-based versioning:
- Current: `/api/v0`
- Future: `/api/v1`, `/api/v2`, etc.

Deprecated endpoints will include:
```
Deprecation: true
Sunset: 2025-12-31T23:59:59Z
```