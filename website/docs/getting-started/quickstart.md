---
sidebar_position: 1
title: Quick Start
---

# Quick Start Guide

Get Raworc up and running in 5 minutes!

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.70+** installed ([Install Rust](https://rustup.rs/))
- **PostgreSQL 14+** running ([Install PostgreSQL](https://www.postgresql.org/download/))
- **Git** for cloning the repository

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/raworc/raworc.git
cd raworc
```

### 2. Build the Project

```bash
cargo build --release
```

This will create the `raworc` binary in `./target/release/`.

### 3. Set Up the Database

First, ensure PostgreSQL is running and create a database:

```sql
CREATE DATABASE raworc;
```

Then set the database URL and run the initial migration:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/raworc"
psql $DATABASE_URL < migrations/20250806000000_initial_schema.sql
```

This creates all tables with proper namespace architecture where:
- Namespaces represent organizations (like company departments or customer accounts)
- Resources (agents, sessions) belong to organizations
- Service accounts and roles are global (can work across organizations)
- Role bindings specify WHERE roles apply (specific organization or globally)

## Starting the Server

### Set Required Environment Variables

```bash
# Required for authentication
export JWT_SECRET="$(openssl rand -base64 32)"

# Optional (defaults shown)
export RAWORC_HOST="0.0.0.0"
export RAWORC_PORT="9000"
```

### Foreground Mode

Run the server in the foreground (recommended for development):

```bash
./target/release/raworc start
```

You should see output indicating the server is running:
```
Starting Raworc server in foreground mode...
Server running at http://0.0.0.0:9000
```

### Daemon Mode (Unix/Linux/macOS)

For production environments, you can run as a daemon:

```bash
./target/release/raworc serve
```

To stop the daemon:
```bash
./target/release/raworc stop
```

## Authentication

Before using the CLI, you need to authenticate with the server.

### Interactive Authentication

```bash
./target/release/raworc auth
```

Choose authentication method:
1. **Service Account Login** - Use username/password
2. **JWT Token** - Provide an existing JWT token

For a new installation, use the default admin credentials:
- Username: `admin`
- Password: `admin`

⚠️ **Security Warning**: Change these default credentials immediately after first login!

The default admin account has global access to all organizations.

## Using the CLI

Once authenticated, connect to the server:

```bash
./target/release/raworc connect
```

You'll see the Raworc interactive prompt:
```
╭──────────────────────────────────────────────────╮
│ ❋ Welcome to Raworc!                             │
│                                                  │
│   Remote Agent Work Orchestration                │
│                                                  │
│   Type /help for commands, /quit or q to exit    │
╰──────────────────────────────────────────────────╯

  ✓ Logged in as: admin (http://localhost:9000)

raworc>
```

Try some commands:
```bash
# Check API version
raworc> /api version

# List global service accounts
raworc> /api service-accounts

# Create an agent in the default organization
raworc> /api POST agents {"name":"helper","namespace":"default","model":"gpt-4","instructions":"You are a helpful assistant"}

# List agents in your accessible organizations
raworc> /api agents

# Grant a user access to an organization
raworc> /api POST role-bindings {"role_name":"developer","principal_name":"alice","principal_type":"ServiceAccount","namespace":"acme-corp"}

# Get help
raworc> /help
```

## Next Steps

- 📖 Read the [Architecture Overview](/docs/concepts/architecture) to understand how Raworc works
- 🏢 Learn about [Organization-based Namespaces](/docs/concepts/namespace-architecture)
- 🔧 Configure Raworc using [environment variables](/docs/admin/configuration)
- 🔐 Set up [RBAC and Organizations](/docs/admin/rbac-namespaces)
- 🤖 Learn how to [deploy agents](/docs/guides/managing-agents)
- 📡 Explore the [REST API](/docs/api/rest-api)
- 🏭 See [Organization Workflow Examples](/docs/guides/organization-workflows)

## Troubleshooting

### Server won't start
- Check if the port 9000 is already in use
- Verify PostgreSQL is running and accessible
- Check logs in the `logs/` directory

### Authentication fails
- Ensure the server is running
- Verify the server URL is correct
- Check network connectivity

### Database connection errors
- Verify `DATABASE_URL` is set correctly
- Ensure PostgreSQL is running
- Check database permissions

Need more help? [Open an issue](https://github.com/raworc/raworc/issues) on GitHub.