---
sidebar_position: 1
title: Configuration
---

# Configuration Guide

Raworc can be configured using environment variables. This guide covers all configuration options and best practices.

## Environment Variables

### Server Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `RAWORC_HOST` | Host/IP address to bind the server to | `0.0.0.0` | `127.0.0.1`, `192.168.1.100` |
| `RAWORC_PORT` | Port number for the REST API server | `9000` | `8080`, `3000` |

#### RAWORC_HOST

Controls which network interfaces the server listens on:
- `0.0.0.0` - Listen on all interfaces (default, allows external connections)
- `127.0.0.1` - Listen on localhost only (more secure for development)
- Specific IP - Listen on a specific network interface

```bash
# Development (localhost only)
RAWORC_HOST=127.0.0.1 raworc start

# Production (all interfaces)
RAWORC_HOST=0.0.0.0 raworc start
```

#### RAWORC_PORT

The port number for the REST API server:
- Default: `9000`
- Common alternatives: `8080`, `3000`, `8000`
- Must be above 1024 for non-root users

```bash
# Use custom port
RAWORC_PORT=8080 raworc start
```

### Organization Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|  
| `DEFAULT_NAMESPACE` | Default organization namespace for resources | `default` | `acme-corp`, `main` |

#### DEFAULT_NAMESPACE

Sets the default organization when creating resources without specifying a namespace:
- Resources (agents, sessions) are created in this organization by default
- Service accounts and roles are always global
- Role bindings can specify organization scope

```bash
# Set default organization
DEFAULT_NAMESPACE=acme-corp raworc start

# Resources created without namespace go to acme-corp
POST /agents {"name":"bot"} # Creates in acme-corp namespace
```

### Database Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://postgres@localhost/raworc` | `postgresql://user:password@host:5432/dbname` |

#### DATABASE_URL Format

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

Examples:
```bash
# Local development
DATABASE_URL=postgresql://localhost/raworc

# With authentication
DATABASE_URL=postgresql://raworc:mypassword@localhost:5432/raworc

# Remote database
DATABASE_URL=postgresql://raworc:secure_pass@db.example.com:5432/raworc_prod

# With SSL
DATABASE_URL=postgresql://user:pass@host/db?sslmode=require
```

### Security Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `JWT_SECRET` | Secret key for JWT token signing | `super-secret-key` | `your-secure-random-string` |

#### JWT_SECRET

Used to sign and verify JWT tokens:
- **MUST** be changed in production
- Should be a long, random string
- Keep it secret and secure

Generate a secure secret:
```bash
# Using OpenSSL
openssl rand -base64 32

# Using /dev/urandom
tr -dc A-Za-z0-9 </dev/urandom | head -c 32

# Using Python
python -c "import secrets; print(secrets.token_urlsafe(32))"
```

## Configuration Methods

### 1. Environment Variables

Set directly in your shell:
```bash
export RAWORC_PORT=8080
export DATABASE_URL=postgresql://user:pass@localhost/raworc
export JWT_SECRET=my-secure-secret
raworc start
```

### 2. .env File

Create a `.env` file in your project directory:
```bash
# .env
RAWORC_HOST=0.0.0.0
RAWORC_PORT=9000
DATABASE_URL=postgresql://raworc:password@localhost:5432/raworc
JWT_SECRET=your-very-secure-secret-key
```

Load it before starting:
```bash
source .env
raworc start
```

### 3. systemd Service

For production Linux systems:

```ini
# /etc/systemd/system/raworc.service
[Unit]
Description=Raworc Server
After=network.target postgresql.service

[Service]
Type=simple
User=raworc
Group=raworc
WorkingDirectory=/opt/raworc
Environment="DATABASE_URL=postgresql://raworc:password@localhost/raworc"
Environment="JWT_SECRET=your-secure-secret"
Environment="RAWORC_HOST=0.0.0.0"
Environment="RAWORC_PORT=9000"
ExecStart=/opt/raworc/bin/raworc start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable raworc
sudo systemctl start raworc
```

### 4. Docker

Using environment variables:
```bash
docker run -d \
  -e DATABASE_URL=postgresql://user:pass@db:5432/raworc \
  -e JWT_SECRET=secure-secret \
  -e RAWORC_PORT=9000 \
  -p 9000:9000 \
  raworc:latest
```

Using env file:
```bash
docker run -d --env-file .env -p 9000:9000 raworc:latest
```

### 5. Docker Compose

```yaml
version: '3.8'
services:
  raworc:
    image: raworc:latest
    environment:
      DATABASE_URL: postgresql://raworc:password@postgres:5432/raworc
      JWT_SECRET: ${JWT_SECRET:-default-dev-secret}
      RAWORC_HOST: 0.0.0.0
      RAWORC_PORT: 9000
    env_file:
      - .env  # Optional: load from file
    ports:
      - "9000:9000"
```

### 6. Kubernetes

Using ConfigMap and Secret:

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: raworc-config
data:
  RAWORC_HOST: "0.0.0.0"
  RAWORC_PORT: "9000"
---
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: raworc-secrets
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:pass@postgres:5432/raworc"
  JWT_SECRET: "your-secure-secret"
---
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: raworc
spec:
  template:
    spec:
      containers:
      - name: raworc
        image: raworc:latest
        envFrom:
        - configMapRef:
            name: raworc-config
        - secretRef:
            name: raworc-secrets
```

## Environment-Specific Configurations

### Development

```bash
# .env.development
RAWORC_HOST=127.0.0.1
RAWORC_PORT=9000
DATABASE_URL=postgresql://postgres@localhost/raworc_dev
JWT_SECRET=dev-secret-not-for-production
```

### Testing

```bash
# .env.test
RAWORC_HOST=127.0.0.1
RAWORC_PORT=9001
DATABASE_URL=postgresql://postgres@localhost/raworc_test
JWT_SECRET=test-secret
```

### Production

```bash
# .env.production
RAWORC_HOST=0.0.0.0
RAWORC_PORT=9000
DATABASE_URL=postgresql://raworc:${DB_PASSWORD}@db.internal:5432/raworc_prod
JWT_SECRET=${JWT_SECRET}  # Set via CI/CD secrets
```

## Security Best Practices

### 1. JWT Secret Management

- **Never** commit secrets to version control
- Use a password manager or secret management service
- Rotate secrets regularly
- Use different secrets for each environment

### 2. Database Security

- Use strong passwords
- Enable SSL/TLS for database connections
- Limit network access to database
- Use read-only replicas where possible

### 3. Network Security

- Use HTTPS in production (reverse proxy)
- Implement firewall rules
- Consider using `127.0.0.1` with a reverse proxy
- Enable rate limiting

## Logging Configuration

Raworc automatically creates logs in the `logs/` directory:

```
logs/
├── raworc.log           # Current log file
├── raworc.log.2025-01-01 # Rotated logs
└── raworc.log.2025-01-02
```

### Log Rotation

Logs are automatically rotated daily. Old logs are kept with date suffixes.

### Log Levels

Set via Rust's `RUST_LOG` environment variable:
```bash
# Debug logging
RUST_LOG=debug raworc start

# Info logging (default)
RUST_LOG=info raworc start

# Error only
RUST_LOG=error raworc start

# Module-specific
RUST_LOG=raworc=debug,tower_http=info raworc start
```

## Performance Tuning

### Database Connection Pool

Configure via DATABASE_URL parameters:
```
postgresql://user:pass@host/db?max_connections=100&pool_timeout=30
```

### Server Threads

Raworc uses Tokio's default thread pool. Adjust via:
```bash
TOKIO_WORKER_THREADS=8 raworc start
```

## Troubleshooting

### Port Already in Use

```bash
# Check what's using the port
lsof -i :9000
# or
netstat -tulpn | grep 9000

# Use a different port
RAWORC_PORT=8080 raworc start
```

### Database Connection Failed

```bash
# Test connection
psql $DATABASE_URL -c "SELECT 1"

# Check PostgreSQL is running
systemctl status postgresql

# Verify credentials
psql -h localhost -U raworc -d raworc
```

### Invalid JWT Secret

If you see JWT validation errors:
1. Ensure JWT_SECRET is set
2. Check it's the same across all instances
3. Verify no trailing spaces or newlines

## Configuration Validation

Before starting in production, validate your configuration:

```bash
# Test database connection
psql $DATABASE_URL -c "SELECT version()"

# Check port availability
nc -zv localhost $RAWORC_PORT

# Verify JWT secret is set
[ -z "$JWT_SECRET" ] && echo "WARNING: JWT_SECRET not set"
```

## Multi-Organization Setup

### Namespace Isolation

Raworc uses namespaces to represent organizations:
- Each organization has its own namespace (e.g., `acme-corp`, `tech-startup`)
- Resources (agents, sessions) belong to specific organizations
- Service accounts and roles are global and can be granted access to multiple organizations

### Organization Management Examples

```bash
# Start server with default organization
DEFAULT_NAMESPACE=acme-corp raworc start

# Create resources for different organizations
POST /agents {"name":"bot1","namespace":"acme-corp",...}
POST /agents {"name":"bot2","namespace":"tech-startup",...}

# Grant user access to specific organization
POST /role-bindings {
  "role_name":"developer",
  "principal_name":"alice",
  "namespace":"acme-corp"  # Alice can only access acme-corp
}

# Grant user global access (platform admin)
POST /role-bindings {
  "role_name":"admin",
  "principal_name":"platform-admin",
  "namespace":null  # null = access to all organizations
}
```

## Next Steps

- Set up [RBAC and Organizations](/docs/admin/rbac-namespaces) for multi-tenant access control
- Learn about [Organization Workflows](/docs/guides/organization-workflows)
- Review [Namespace Architecture](/docs/concepts/namespace-architecture)
- Configure [monitoring](/docs/admin/monitoring) for production
- Review [security best practices](/docs/admin/security)