# Quick Start Guide

Get Raworc up and running in 5 minutes!

## Prerequisites

- Rust 1.70+ installed
- PostgreSQL 14+ running
- Git

## 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/raworc/raworc.git
cd raworc

# Build the project
cargo build --release
```

## 2. Set Up Database

```bash
# Set your database URL
export DATABASE_URL="postgresql://user:password@localhost:5432/raworc"

# Run migrations
psql $DATABASE_URL < migrations/001_create_rbac_tables.sql
```

## 3. Start the Server

```bash
# Start Raworc server (with defaults)
./target/release/raworc start

# Or with custom configuration
RAWORC_HOST=0.0.0.0 RAWORC_PORT=8080 ./target/release/raworc start

# Server will be available at:
# - REST API: http://localhost:9000/api/v1
# - Swagger UI: http://localhost:9000/swagger-ui/
```

> **Note**: See [Configuration Guide](configuration.md) for all available options.

## 4. Authenticate

In a new terminal:

```bash
# Authenticate with default admin account
./target/release/raworc auth

# Enter credentials:
# Username: admin
# Password: admin
```

## 5. Test the API

Using the CLI interactive mode:

```bash
# Connect to the server
./target/release/raworc connect

# Try some commands
raworc> /api version
raworc> /api auth/me
raworc> /api service-accounts
raworc> /help
```

## 6. Create Your First Service Account

```bash
# In the interactive CLI
raworc> /api POST service-accounts {"user":"my-bot","pass":"SecurePass123!","description":"My first bot"}

# List accounts to verify
raworc> /api service-accounts
```

## Next Steps

- 📚 Read the [REST API Documentation](rest-api.md)
- 🔐 Learn about [RBAC](rbac.md) to set up proper access control
- 🛠️ Explore [CLI Examples](cli-api-examples.md) for more commands
- 🎯 Check out the [Swagger UI](http://localhost:9000/swagger-ui/) for interactive API testing

## Troubleshooting

### Database Connection Failed
- Ensure PostgreSQL is running: `pg_isready`
- Check DATABASE_URL is correct
- Verify database exists: `psql -l`

### Server Won't Start
- Check if port 9000 is available: `lsof -i :9000`
- Review logs: `tail -f logs/raworc.log.$(date +%Y-%m-%d)`

### Authentication Failed
- Ensure server is running: `./target/release/raworc status`
- Default credentials: username `admin`, password `admin`

Need help? [Open an issue](https://github.com/raworc/raworc/issues)