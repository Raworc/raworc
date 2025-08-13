# Raworc

**Remote Agent Work Orchestration** - A Docker-based orchestration platform for managing containerized AI agent sessions.

## Features

- **Docker Container Management** - Each session runs in its own isolated Docker container
- **Resource Limits** - CPU, memory, and disk limits per container
- **Persistent Volumes** - Session data persists across container restarts
- **REST API** - Full-featured REST API with OpenAPI documentation
- **RBAC System** - Role-based access control for multi-user environments
- **Session Lifecycle** - Automatic container management (idle timeout, health checks)
- **CLI Interface** - Interactive command-line interface for server management

## Quick Start

### Prerequisites

- Docker Engine (20.10+)
- PostgreSQL (15+) or use Docker Compose
- Rust (1.75+) for building from source

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/raworc/raworc.git
cd raworc

# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f raworc
```

The server will be available at `http://localhost:9000`

### Manual Installation

#### 1. Set up PostgreSQL

```bash
# Using Docker
docker run -d --name postgres \
  -e POSTGRES_DB=raworc \
  -e POSTGRES_USER=raworc \
  -e POSTGRES_PASSWORD=raworc \
  -p 5432:5432 \
  postgres:15
```

#### 2. Build and Run Raworc

```bash
# Clone the repository
git clone https://github.com/raworc/raworc.git
cd raworc

# Copy and configure environment
cp .env.example .env
# Edit .env with your settings

# Build the project
cargo build --release

# Run database migrations
export DATABASE_URL=postgresql://raworc:raworc@localhost:5432/raworc
sqlx migrate run

# Start the server
./target/release/raworc start
```

## Usage

### Server Management

```bash
# Start server in foreground
raworc start

# Start server as daemon (Unix only)
raworc serve

# Stop the server
raworc stop

# Check authentication status
raworc status
```

### Authentication

```bash
# Authenticate with the server
raworc auth

# Choose authentication method:
# 1. Service account login (default: admin/admin)
# 2. JWT token
```

### Interactive CLI

```bash
# Connect to server (after authentication)
raworc connect

# Available commands in interactive mode:
/api <endpoint>              # Execute API requests
/status                      # Show server status
/help                        # Show help
/quit                        # Exit
```

### API Examples

```bash
# In interactive mode
/api version                                    # GET /api/v0/version
/api sessions                                   # List sessions
/api POST sessions {"name":"test","workspace":"default"}
/api DELETE sessions/<id>
```

### REST API

The REST API is available at `http://localhost:9000/api/v0`

- Swagger UI: `http://localhost:9000/swagger-ui/`
- OpenAPI spec: `http://localhost:9000/api-docs/openapi.json`

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:pass@host:port/dbname

# JWT
JWT_SECRET=your-secret-key

# Server
RAWORC_HOST=0.0.0.0
RAWORC_PORT=9000

# Host Agent
HOST_AGENT_IMAGE=python:3.11-slim
HOST_AGENT_CPU_LIMIT=0.5          # CPUs
HOST_AGENT_MEMORY_LIMIT=536870912 # Bytes (512MB)
HOST_AGENT_DISK_LIMIT=1073741824  # Bytes (1GB)
HOST_AGENT_VOLUMES_PATH=/var/lib/raworc/volumes

# Logging
RUST_LOG=info  # trace, debug, info, warn, error
```

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Client    │────▶│   Raworc     │────▶│   Docker    │
│  (REST API) │     │    Server    │     │   Engine    │
└─────────────┘     └──────────────┘     └─────────────┘
                            │                     │
                            ▼                     ▼
                    ┌──────────────┐     ┌─────────────┐
                    │  PostgreSQL  │     │  Container  │
                    │   Database   │     │  (Session)  │
                    └──────────────┘     └─────────────┘
```

### Components

- **REST API Server** - Axum-based HTTP server with JWT authentication
- **Docker Manager** - Bollard-based Docker client for container lifecycle
- **Database Layer** - SQLx with PostgreSQL for state persistence
- **RBAC System** - Role-based access control with service accounts
- **CLI Interface** - Interactive command-line client

## Development

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- start
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Docker Development

```bash
# Build Docker image
docker build -t raworc:latest .

# Run with Docker Compose
docker-compose up --build

# Clean up
docker-compose down -v
```

## Testing

### Run Test Script

```bash
# Make script executable
chmod +x test-docker.sh

# Run tests
./test-docker.sh
```

The test script will:
1. Start the server
2. Authenticate as admin
3. Create a test session
4. Verify Docker container creation
5. Clean up resources

## Troubleshooting

### Common Issues

#### Docker Permission Denied
```bash
# Add user to docker group
sudo usermod -aG docker $USER
# Log out and back in
```

#### Database Connection Failed
```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Test connection
psql postgresql://raworc:raworc@localhost:5432/raworc
```

#### Port Already in Use
```bash
# Change port in .env
RAWORC_PORT=9001

# Or find and stop process using port
lsof -i :9000
kill <PID>
```

#### Container Creation Failed
```bash
# Check Docker daemon
docker version

# Check Docker socket permissions
ls -la /var/run/docker.sock

# Pull required image manually
docker pull python:3.11-slim
```

## Security Considerations

- Change default admin password immediately
- Use strong JWT secrets in production
- Run containers with resource limits
- Use network isolation for containers
- Regular security updates for base images

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For issues and questions, please use the GitHub issue tracker.