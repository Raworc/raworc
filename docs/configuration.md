# Raworc Configuration

Raworc can be configured using environment variables. All configuration options have sensible defaults.

## Environment Variables

### Server Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `RAWORC_HOST` | Host/IP address to bind the server to | `0.0.0.0` | `127.0.0.1`, `192.168.1.100` |
| `RAWORC_PORT` | Port number for the REST API server | `9000` | `8080`, `3000` |

### Database Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://postgres@localhost/raworc` | `postgresql://user:password@host:5432/dbname` |

### Security Configuration

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `JWT_SECRET` | Secret key for JWT token signing | `super-secret-key` | `your-secure-random-string` |

## Usage Examples

### Basic Configuration

```bash
# Run with custom port
RAWORC_PORT=8080 raworc start

# Run on localhost only
RAWORC_HOST=127.0.0.1 raworc start

# Custom database
DATABASE_URL=postgresql://myuser:mypass@db.example.com:5432/raworc raworc start
```

### Production Configuration

```bash
# Create a .env file
cat > .env << EOF
RAWORC_HOST=0.0.0.0
RAWORC_PORT=9000
DATABASE_URL=postgresql://raworc:secure_password@postgres:5432/raworc_prod
JWT_SECRET=$(openssl rand -base64 32)
EOF

# Source the environment file
source .env

# Start the server
raworc start
```

### Docker Configuration

```dockerfile
# In your Dockerfile or docker-compose.yml
ENV RAWORC_HOST=0.0.0.0
ENV RAWORC_PORT=9000
ENV DATABASE_URL=postgresql://raworc:password@postgres:5432/raworc
ENV JWT_SECRET=your-secure-secret
```

### Kubernetes Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: raworc-config
data:
  RAWORC_HOST: "0.0.0.0"
  RAWORC_PORT: "9000"
  DATABASE_URL: "postgresql://raworc:password@postgres-service:5432/raworc"
---
apiVersion: v1
kind: Secret
metadata:
  name: raworc-secrets
type: Opaque
stringData:
  JWT_SECRET: "your-secure-secret-key"
```

## Network Binding

The `RAWORC_HOST` setting determines which network interfaces the server listens on:

- `0.0.0.0` - Listen on all interfaces (default, allows external connections)
- `127.0.0.1` - Listen on localhost only (more secure for development)
- Specific IP - Listen on a specific network interface

## Security Considerations

1. **JWT_SECRET**: Always use a strong, random secret in production. You can generate one with:
   ```bash
   openssl rand -base64 32
   ```

2. **DATABASE_URL**: Never commit database credentials to version control. Use environment variables or secrets management.

3. **Network Binding**: In production, consider using `127.0.0.1` and putting a reverse proxy (nginx, caddy) in front for SSL termination.

## Default Ports

If you change the default port (9000), remember to update:
- Client connections
- Docker port mappings
- Kubernetes service definitions
- Firewall rules
- Documentation references