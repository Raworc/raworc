---
sidebar_position: 2
title: Installation
---

# Installation Guide

This guide covers various installation methods for Raworc.

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores
- **RAM**: 4 GB
- **Storage**: 10 GB available space
- **OS**: Linux, macOS, or Windows (with limitations)

### Recommended Requirements
- **CPU**: 4+ cores
- **RAM**: 8+ GB
- **Storage**: 50+ GB (for persistent volumes)
- **OS**: Linux (Ubuntu 20.04+ or similar)

## Installation Methods

### From Source

Building from source gives you the latest features and is the recommended approach for development.

#### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git
- PostgreSQL 14+
- Build essentials (gcc, make, etc.)

#### Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/raworc/raworc.git
   cd raworc
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Install the binary** (optional)
   ```bash
   sudo cp target/release/raworc /usr/local/bin/
   ```

### Using Docker

Docker provides an isolated environment and simplifies deployment.

#### Using Docker Compose

1. **Create docker-compose.yml**
   ```yaml
   version: '3.8'
   services:
     postgres:
       image: postgres:14
       environment:
         POSTGRES_DB: raworc
         POSTGRES_USER: raworc
         POSTGRES_PASSWORD: changeme
       volumes:
         - postgres_data:/var/lib/postgresql/data
       ports:
         - "5432:5432"

     raworc:
       build: .
       environment:
         DATABASE_URL: postgresql://raworc:changeme@postgres:5432/raworc
         RAWORC_HOST: 0.0.0.0
         RAWORC_PORT: 9000
         JWT_SECRET: your-secret-key
       ports:
         - "9000:9000"
       depends_on:
         - postgres
       volumes:
         - ./logs:/app/logs

   volumes:
     postgres_data:
   ```

2. **Start the services**
   ```bash
   docker-compose up -d
   ```

#### Building Docker Image

1. **Create Dockerfile**
   ```dockerfile
   FROM rust:1.70 as builder
   WORKDIR /app
   COPY Cargo.toml Cargo.lock ./
   COPY src ./src
   COPY migrations ./migrations
   RUN cargo build --release

   FROM debian:bullseye-slim
   RUN apt-get update && apt-get install -y \
       libssl1.1 \
       ca-certificates \
       && rm -rf /var/lib/apt/lists/*
   
   COPY --from=builder /app/target/release/raworc /usr/local/bin/
   COPY --from=builder /app/migrations /app/migrations
   
   EXPOSE 9000
   CMD ["raworc", "start"]
   ```

2. **Build and run**
   ```bash
   docker build -t raworc .
   docker run -p 9000:9000 raworc
   ```


## Database Setup

### PostgreSQL Installation

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
```

#### macOS (using Homebrew)
```bash
brew install postgresql
brew services start postgresql
```

#### Using Docker
```bash
docker run -d \
  --name postgres \
  -e POSTGRES_DB=raworc \
  -e POSTGRES_USER=raworc \
  -e POSTGRES_PASSWORD=changeme \
  -p 5432:5432 \
  postgres:14
```

### Database Initialization

1. **Create database**
   ```sql
   CREATE DATABASE raworc;
   CREATE USER raworc WITH ENCRYPTED PASSWORD 'changeme';
   GRANT ALL PRIVILEGES ON DATABASE raworc TO raworc;
   ```

2. **Run migrations**
   ```bash
   export DATABASE_URL="postgresql://raworc:changeme@localhost:5432/raworc"
   psql $DATABASE_URL < migrations/001_create_rbac_tables.sql
   psql $DATABASE_URL < migrations/002_create_agents_table.sql
   psql $DATABASE_URL < migrations/003_create_sessions_table.sql
   ```

## Platform-Specific Notes

### Linux
- Full feature support including daemon mode
- Recommended for production deployments
- Systemd service file example:
  ```ini
  [Unit]
  Description=Raworc Server
  After=network.target postgresql.service

  [Service]
  Type=simple
  User=raworc
  Environment="DATABASE_URL=postgresql://raworc:password@localhost/raworc"
  Environment="JWT_SECRET=your-secret-key"
  ExecStart=/usr/local/bin/raworc start
  Restart=always

  [Install]
  WantedBy=multi-user.target
  ```

### macOS
- Daemon mode supported
- May require additional permissions for network binding
- Consider using launchd for auto-start

### Windows
- Limited to foreground mode (no daemon support)
- Use Windows Task Scheduler for auto-start
- Consider using WSL2 for full feature support

## Verification

After installation, verify everything is working:

1. **Check version**
   ```bash
   raworc --version
   ```

2. **Test server startup**
   ```bash
   raworc start
   # In another terminal:
   curl http://localhost:9000/api/v0/health
   ```

3. **Authenticate**
   ```bash
   raworc auth
   ```
   
   **Important Security Note**: On first run, Raworc creates a default admin account with credentials:
   - Username: `admin`
   - Password: `admin`
   
   ⚠️ **Change these credentials immediately after first login for security!**

## Troubleshooting

### Common Issues

**Build fails with OpenSSL errors**
- Install OpenSSL development headers
- Ubuntu: `sudo apt install libssl-dev`
- macOS: `brew install openssl`

**Database connection refused**
- Check PostgreSQL is running
- Verify connection parameters
- Check firewall settings

**Permission denied binding to port**
- Use a port above 1024
- Run with appropriate permissions
- Check if port is already in use

## Next Steps

- [Configure Raworc](/docs/admin/configuration) for your environment
- Set up [RBAC permissions](/docs/admin/rbac)
- Learn about [deploying agents](/docs/guides/managing-agents)