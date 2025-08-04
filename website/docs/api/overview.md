---
sidebar_position: 1
title: API Overview
---

# API Overview

Raworc provides a comprehensive REST API for managing all platform operations. The API follows RESTful principles and returns JSON responses.

## Base Information

- **Base URL**: `http://your-server:9000/api/v0`
- **Protocol**: HTTP/HTTPS
- **Format**: JSON
- **Authentication**: Bearer token (JWT)

## Interactive Documentation

When the Raworc server is running, you can access interactive API documentation:

- **Swagger UI**: `http://your-server:9000/swagger-ui/`
- **OpenAPI Spec**: `http://your-server:9000/api-docs/openapi.json`

## Authentication

All API endpoints (except `/health`, `/version`, and `/auth/internal`) require authentication using a JWT bearer token.

### Obtaining a Token

```bash
POST /api/v0/auth/internal
Content-Type: application/json

{
  "user": "admin",
  "pass": "your-password",
  "namespace": null
}
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_at": "2025-01-02T12:00:00Z"
}
```

### Using the Token

Include the token in the Authorization header:
```
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc...
```

## API Endpoints

### Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/version` | GET | API version info |
| `/auth/internal` | POST | Authenticate and get token |
| `/auth/me` | GET | Get current user info |

### Service Accounts

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/service-accounts` | GET | List all service accounts |
| `/service-accounts` | POST | Create new service account |
| `/service-accounts/{id}` | GET | Get specific service account |
| `/service-accounts/{id}` | DELETE | Delete service account |

### Roles

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/roles` | GET | List all roles |
| `/roles` | POST | Create new role |
| `/roles/{id}` | GET | Get specific role |
| `/roles/{id}` | DELETE | Delete role |

### Role Bindings

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/role-bindings` | GET | List all role bindings |
| `/role-bindings` | POST | Create new role binding |
| `/role-bindings/{id}` | GET | Get specific role binding |
| `/role-bindings/{id}` | DELETE | Delete role binding |

### Agents

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/agents` | GET | List all agents |
| `/agents` | POST | Create new agent |
| `/agents/{id}` | GET | Get specific agent |
| `/agents/{id}` | PUT | Update agent |
| `/agents/{id}` | DELETE | Delete agent |

## Request Format

### Headers

Required headers for authenticated requests:
```
Authorization: Bearer <token>
Content-Type: application/json
```

### Request Body

All POST and PUT requests accept JSON:
```json
{
  "field1": "value1",
  "field2": "value2"
}
```

## Response Format

### Success Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "example",
  "created_at": "2025-01-01T00:00:00Z"
}
```

### Error Response

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Resource not found"
  }
}
```

## Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful deletion) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict |
| 422 | Unprocessable Entity |
| 500 | Internal Server Error |

## Rate Limiting

Currently, Raworc does not enforce rate limiting, but this may change in future versions. Best practices:
- Cache responses when possible
- Use pagination for list operations
- Implement exponential backoff for retries

## Pagination

List endpoints support pagination (coming soon):
```
GET /api/v0/agents?page=1&limit=20
```

## Filtering

Some endpoints support filtering (coming soon):
```
GET /api/v0/agents?active=true&model=gpt-4
```

## CLI Integration

The Raworc CLI provides convenient access to the API:

```bash
# In interactive mode
raworc> /api agents
raworc> /api POST agents {"name":"helper","model":"gpt-4"}
raworc> /api DELETE agents/helper
```

## SDK Support

Official SDKs are planned for:
- Python
- JavaScript/TypeScript
- Go
- Rust

## Webhooks

Webhook support is planned for future releases to enable:
- Real-time notifications
- Event-driven workflows
- Third-party integrations

## API Versioning

The API uses URL versioning:
- Current version: `v0`
- Format: `/api/v{version}/endpoint`

Breaking changes will result in a new API version.

## Best Practices

1. **Use Specific Fields**: Only request/send needed fields
2. **Handle Errors**: Implement proper error handling
3. **Validate Input**: Validate data before sending
4. **Use HTTPS**: Always use HTTPS in production
5. **Token Management**: Refresh tokens before expiry
6. **Idempotency**: Make requests idempotent where possible

## Next Steps

- Explore the [REST API Reference](/docs/api/rest-api) for detailed endpoint documentation
- Try the [CLI Examples](/docs/guides/cli-examples) for interactive API usage
- Read about [Authentication](/docs/api/authentication) in detail
- Learn about [RBAC Permissions](/docs/admin/rbac) for API access control