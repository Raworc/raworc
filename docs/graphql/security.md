# Security

## Authentication

### JWT Tokens
- Include token in `Authorization: Bearer <token>` header
- Tokens expire after 24 hours by default
- External token generation requires admin access

### Service Accounts
- Default admin account: `admin/admin`
- Change default password in production
- Use strong passwords for new accounts

## Authorization

### RBAC System
- All operations require proper permissions
- Admin role has full access (`*.*.*`)
- Permissions checked on every request

### Guards
- `AuthGuard`: Requires valid JWT token
- `RbacGuard`: Requires specific permissions
- `admin_only()`: Requires admin role

## Best Practices

1. **Change default admin password**
2. **Use least privilege principle**
3. **Rotate tokens regularly**
4. **Monitor access logs**
5. **Use HTTPS in production**