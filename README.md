# Raworc

Container orchestration system with AI agent integration.

## Quick Start

```bash
# Build the CLI
cargo build --release

# Build Docker images
./target/release/raworc build

# Start all services
./target/release/raworc start

# Authenticate with the server
./target/release/raworc auth

# Connect to server (interactive mode)
./target/release/raworc connect
# Or just run without arguments (defaults to connect)
./target/release/raworc
```

Services started:
- PostgreSQL (port 5433)
- API Server (port 9000)  
- Operator (manages containers)

## CLI Commands

### Core Commands

```bash
# Authentication & Connection
raworc                    # Connect to server (default)
raworc auth               # Authenticate with API server
raworc status             # Show authentication status
raworc connect            # Interactive connection to server

# Service Management
raworc start              # Start all services via Docker Compose
raworc start server       # Start specific service
raworc stop               # Stop all services
raworc stop server        # Stop specific service

# Build Docker Images
raworc build              # Build all images
raworc build server       # Build specific image
raworc build operator
raworc build host
```

### Interactive Mode

Once authenticated, use `raworc` or `raworc connect` to enter interactive mode:

```bash
raworc> /api health                    # GET /api/v0/health
raworc> /api agents                    # List agents
raworc> /api sessions                  # List sessions
raworc> /api POST agents {"name":"test","model":"claude-3-haiku"}
raworc> /api DELETE sessions/uuid
raworc> /status                        # Show auth status
raworc> /help                          # Show commands
raworc> /quit                          # Exit
```

## Testing the System

Default credentials: `admin` / `admin`

### Using curl (current method)

```bash
# 1. Get auth token
TOKEN=$(curl -s -X POST http://localhost:9000/api/v0/auth/internal \
  -H "Content-Type: application/json" \
  -d '{"user":"admin","pass":"admin"}' | jq -r '.token')

# 2. Create an agent
AGENT_ID=$(curl -s -X POST http://localhost:9000/api/v0/agents \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-agent",
    "workspace": "default",
    "instructions": "You are a helpful assistant",
    "model": "claude-3-haiku",
    "tools": [],
    "routes": []
  }' | jq -r '.id')

# 3. Create a session
SESSION_ID=$(curl -s -X POST http://localhost:9000/api/v0/sessions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"test-session\",
    \"workspace\": \"default\",
    \"starting_prompt\": \"Hello\",
    \"agent_ids\": [\"$AGENT_ID\"]
  }" | jq -r '.id')

# 4. Check session status
curl -s -H "Authorization: Bearer $TOKEN" \
  http://localhost:9000/api/v0/sessions/$SESSION_ID | jq
```

### CLI commands (planned)

```bash
# Future commands to be implemented:
raworc auth login --user admin --pass admin
raworc connect http://localhost:9000
raworc agent create --name "my-agent" --model "claude-3-haiku"
raworc session create --name "my-session" --agent $AGENT_ID
raworc session list
raworc session logs $SESSION_ID
```

## Architecture

- **Server**: REST API for sessions, agents, auth
- **Operator**: Monitors task queue, manages containers
- **Host**: Agent runtime in containers
- **Database**: PostgreSQL storage

## Configuration

Environment variables:
- `DATABASE_URL`: PostgreSQL connection
- `JWT_SECRET`: JWT token secret
- `HOST_AGENT_IMAGE`: Container image (default: raworc-host:latest)
- `HOST_AGENT_CPU_LIMIT`: CPU limit (default: 0.5)
- `HOST_AGENT_MEMORY_LIMIT`: Memory in bytes (default: 536870912)

## Development

```bash
cargo test         # run tests
cargo fmt          # format code
cargo clippy       # check lints
```