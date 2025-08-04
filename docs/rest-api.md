# Raworc REST API Documentation

## Overview

Raworc provides a comprehensive REST API for managing platform operations, authentication, and role-based access control.

- **Base URL**: `/api/v1`
- **Authentication**: Bearer token (JWT)
- **Content-Type**: `application/json`

## OpenAPI Documentation

When the server is running, you can access:
- **Swagger UI**: `/swagger-ui/`
- **OpenAPI JSON**: `/api-docs/openapi.json`

## Authentication

All endpoints except `/health`, `/version`, and `/auth/internal` require authentication via Bearer token.

Include the token in the Authorization header:
```
Authorization: Bearer <token>
```

## API Endpoints

### Health & Version

#### GET /health
Check server health status.

**Response**: `200 OK`

#### GET /version
Get API version information.

**Response**:
```json
{
  "version": "0.1.0",
  "api": "v1"
}
```

### Authentication

#### POST /auth/internal
Authenticate a service account and receive a JWT token.

**Request**:
```json
{
  "user": "admin",
  "pass": "password123",
  "namespace": null
}
```

**Response**:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_at": "2025-01-02T12:00:00Z"
}
```

#### GET /auth/me
Get information about the authenticated user.

**Response**:
```json
{
  "user": "admin",
  "namespace": null,
  "type": "ServiceAccount"
}
```

### Service Accounts

#### GET /service-accounts
List all service accounts.

**Response**:
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

#### POST /service-accounts
Create a new service account.

**Request**:
```json
{
  "user": "deploy-bot",
  "pass": "SecurePass123!",
  "namespace": "production",
  "description": "Deployment automation bot"
}
```

**Response**: Same as GET /service-accounts/{id}

#### GET /service-accounts/{id}
Get a specific service account by ID or username.

**Response**:
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

#### DELETE /service-accounts/{id}
Delete a service account by ID or username.

**Response**: `200 OK`

### Roles

#### GET /roles
List all roles.

**Response**:
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

#### POST /roles
Create a new role.

**Request**:
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

**Response**: Same as GET /roles/{id}

#### GET /roles/{id}
Get a specific role by ID or name.

#### DELETE /roles/{id}
Delete a role by ID or name.

**Response**: `200 OK`

### Role Bindings

#### GET /role-bindings
List all role bindings.

**Response**:
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

#### POST /role-bindings
Create a new role binding.

**Request**:
```json
{
  "role_name": "reader",
  "principal_name": "deploy-bot",
  "principal_type": "ServiceAccount",
  "namespace": null
}
```

**Response**: Same as GET /role-bindings/{id}

#### GET /role-bindings/{id}
Get a specific role binding by ID.

#### DELETE /role-bindings/{id}
Delete a role binding by ID.

**Response**: `200 OK`

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

Common error codes:
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Missing or invalid token
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists
- `422 Unprocessable Entity` - Validation error
- `500 Internal Server Error` - Server error

## CLI Usage

The Raworc CLI provides an interactive way to interact with the REST API:

```bash
# Authenticate
raworc auth

# Use interactive mode
raworc connect

# In interactive mode:
raworc> /api version
raworc> /api GET service-accounts
raworc> /api POST roles {"name":"test","rules":[]}
raworc> /api DELETE roles/test
```

See [CLI API Examples](cli-api-examples.md) for more details.