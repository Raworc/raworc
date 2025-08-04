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

Then set the database URL and run migrations:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/raworc"
psql $DATABASE_URL < migrations/001_create_rbac_tables.sql
psql $DATABASE_URL < migrations/002_create_agents_table.sql
```

## Starting the Server

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
- Password: `changeme` (change this immediately!)

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

# List service accounts
raworc> /api service-accounts

# Get help
raworc> /help
```

## Next Steps

- 📖 Read the [Architecture Overview](/docs/concepts/architecture) to understand how Raworc works
- 🔧 Configure Raworc using [environment variables](/docs/admin/configuration)
- 🔐 Set up proper [RBAC permissions](/docs/admin/rbac)
- 🤖 Learn how to [deploy agents](/docs/guides/managing-agents)
- 📡 Explore the [REST API](/docs/api/rest-api)

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