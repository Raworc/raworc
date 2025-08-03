# RBAC Permissions

## Guards

### Public Access
- `version`
- `status`

### Authentication Required
- `whoami`

### Admin Only
- `generateExternalToken`
- `setupAdminRole`

### RBAC Permissions

#### Service Accounts
- **List**: `rbac.serviceaccounts.list`
- **Get**: `rbac.serviceaccounts.get`
- **Create**: `rbac.serviceaccounts.create`
- **Delete**: `rbac.serviceaccounts.delete`

#### Roles
- **List**: `rbac.roles.get`
- **Get**: `rbac.roles.get`
- **Create**: `rbac.roles.*`
- **Delete**: `rbac.roles.*`

#### Role Bindings
- **List**: `rbac.rolebindings.list`
- **Get**: `rbac.rolebindings.get`
- **Create**: `rbac.rolebindings.*`
- **Delete**: `rbac.rolebindings.*`

## Admin Role

Default admin role has full access:
```
api_groups: ["*"]
resources: ["*"]
verbs: ["*"]
```

## Permission Format

Permissions follow the pattern:
```
<api_group>.<resource>.<verb>
```

Where:
- `api_group`: Usually "rbac"
- `resource`: serviceaccounts, roles, rolebindings
- `verb`: get, list, create, update, delete, *