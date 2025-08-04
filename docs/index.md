# Raworc Documentation

Welcome to the Raworc documentation! This index provides organized access to all available documentation.

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ installed
- PostgreSQL 14+ running
- Git

### Quick Start

```bash
# Clone the repository
git clone https://github.com/raworc/raworc.git
cd raworc

# Build the project
cargo build --release

# Set up database
export DATABASE_URL="postgresql://user:password@localhost:5432/raworc"
psql $DATABASE_URL < migrations/001_create_rbac_tables.sql

# Start the server
./target/release/raworc start

# In another terminal, authenticate
./target/release/raworc auth
```

## 📚 Available Documentation

### API Documentation
- **[REST API Reference](rest-api.md)** - Complete endpoint documentation
- **[CLI Examples](cli-api-examples.md)** - Command-line interface examples
- **[Configuration Guide](configuration.md)** - Environment variables and configuration options

### System Documentation
- **[RBAC System](rbac.md)** - Role-based access control explained

### Interactive Documentation (requires running server)
- **[Swagger UI](http://localhost:9000/swagger-ui/)** - Interactive API explorer
- **[OpenAPI Spec](http://localhost:9000/api-docs/openapi.json)** - OpenAPI 3.0 specification

## 🔧 Quick Reference

### Environment Variables
| Variable | Description | Default |
|----------|-------------|---------|
| `RAWORC_HOST` | Server bind address | `0.0.0.0` |
| `RAWORC_PORT` | Server port | `9000` |
| `DATABASE_URL` | PostgreSQL connection | `postgresql://postgres@localhost/raworc` |
| `JWT_SECRET` | JWT signing secret | `super-secret-key` |

### Common Commands
```bash
# Server management
raworc start              # Start server in foreground
raworc serve              # Start as daemon (Unix only)
raworc stop               # Stop daemon
raworc status             # Check authentication status

# Authentication
raworc auth               # Interactive authentication

# Interactive CLI
raworc connect            # Connect to server
raworc> /api version      # Get API version
raworc> /api auth/me      # Get current user info
raworc> /help             # Show available commands
```

## 🔍 External Resources

### Community
- **[GitHub Repository](https://github.com/raworc/raworc)** - Source code
- **[Issue Tracker](https://github.com/raworc/raworc/issues)** - Report bugs or request features

### External Links
- **[Twitter/X](https://x.com/raworc)** - Latest updates

## 📝 Version Information

- **Current Version**: 0.1.0
- **API Version**: v1
- **Last Updated**: January 2025

---

Can't find what you're looking for? [Open an issue](https://github.com/raworc/raworc/issues/new)