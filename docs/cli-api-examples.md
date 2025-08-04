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