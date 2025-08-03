# PostgreSQL Setup Guide

This guide explains how to set up PostgreSQL for the Raworc platform.

## Prerequisites

- PostgreSQL 14 or higher
- A PostgreSQL client (psql, pgAdmin, or similar)

## Setup Options

### Option 1: Local PostgreSQL

1. **Install PostgreSQL**
   ```bash
   # macOS
   brew install postgresql@14
   brew services start postgresql@14
   
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install postgresql postgresql-contrib
   
   # Fedora/RHEL
   sudo dnf install postgresql postgresql-server
   sudo postgresql-setup --initdb
   sudo systemctl start postgresql
   ```

2. **Create Database and User**
   ```bash
   # Connect as postgres user
   sudo -u postgres psql
   
   # Create database and user
   CREATE DATABASE raworc;
   CREATE USER raworc_user WITH ENCRYPTED PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE raworc TO raworc_user;
   
   # Enable UUID extension
   \c raworc
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   ```

3. **Configure Environment**
   ```bash
   cp .env.example .env
   # Edit .env and set:
   DATABASE_URL=postgresql://raworc_user:your_password@localhost:5432/raworc
   JWT_SECRET=your-secure-random-string
   ```

### Option 2: Docker PostgreSQL

1. **Run PostgreSQL Container**
   ```bash
   docker run -d \
     --name raworc-postgres \
     -e POSTGRES_DB=raworc \
     -e POSTGRES_USER=raworc_user \
     -e POSTGRES_PASSWORD=your_password \
     -p 5432:5432 \
     postgres:14
   ```

2. **Configure Environment**
   ```bash
   cp .env.example .env
   # Edit .env and set:
   DATABASE_URL=postgresql://raworc_user:your_password@localhost:5432/raworc
   JWT_SECRET=your-secure-random-string
   ```

### Option 3: Managed PostgreSQL Services

For production, you can use managed PostgreSQL services:
- AWS RDS PostgreSQL
- Google Cloud SQL for PostgreSQL
- Azure Database for PostgreSQL
- DigitalOcean Managed Databases
- Heroku Postgres
- Supabase (PostgreSQL-based)

Simply obtain the connection string from your provider and set it in your `.env` file.

## Database Migration

1. **Run the Migration**
   ```bash
   # Using psql
   psql $DATABASE_URL < migrations/001_create_rbac_tables.sql
   
   # Or connect to the database and run:
   \i migrations/001_create_rbac_tables.sql
   ```

## Database Schema

The Raworc platform uses the following tables:

### Tables

- **service_accounts**: Internal accounts with credentials and RBAC
  - `id` (UUID): Primary key
  - `name` (TEXT): Unique username
  - `namespace` (TEXT): Organizational namespace
  - `email` (TEXT): Optional email/description
  - `password_hash` (TEXT): Bcrypt password hash
  - `created_at`, `updated_at` (TIMESTAMPTZ): Timestamps

- **roles**: Permission definitions
  - `id` (UUID): Primary key
  - `name` (TEXT): Role name
  - `namespace` (TEXT): Namespace scope
  - `rules` (JSONB): Array of permission rules
  - `created_at`, `updated_at` (TIMESTAMPTZ): Timestamps

- **role_bindings**: Links between roles and subjects/service accounts
  - `id` (UUID): Primary key
  - `name` (TEXT): Binding name
  - `namespace` (TEXT): Namespace scope
  - `role_name` (TEXT): Reference to role
  - `subjects` (JSONB): Array of subjects
  - `created_at`, `updated_at` (TIMESTAMPTZ): Timestamps

### Features

- UUID primary keys for all tables
- Automatic timestamp updates via triggers
- Row Level Security (RLS) ready
- Optimized indexes for performance
- JSONB support for flexible data structures

## Testing the Setup

1. **Start the Raworc server**
   ```bash
   cargo run
   ```

2. **Verify database connection**
   - The server should connect successfully
   - Default admin account will be created automatically (user: admin, pass: admin)

3. **Test GraphQL endpoint**
   ```bash
   # Login mutation
   curl -X POST http://localhost:9000/graphql \
     -H "Content-Type: application/json" \
     -d '{
       "query": "mutation { login(username: \"admin\", password: \"admin\") { token } }"
     }'
   ```

## Troubleshooting

### Connection Issues
- Verify PostgreSQL is running: `pg_isready`
- Check connection string format: `postgresql://user:password@host:port/database`
- Ensure PostgreSQL is accepting connections (check `pg_hba.conf`)

### Migration Errors
- Ensure the UUID extension is enabled: `CREATE EXTENSION IF NOT EXISTS "uuid-ossp";`
- Check user has CREATE privileges on the database
- Verify you're connected to the correct database

### Authentication Failures
- Verify DATABASE_URL is correctly set in environment
- Check JWT_SECRET is set
- Ensure bcrypt is working correctly (test with admin/admin)

## Production Considerations

1. **Security**
   - Use strong passwords
   - Enable SSL/TLS connections
   - Implement proper firewall rules
   - Use connection pooling

2. **Performance**
   - Configure appropriate connection pool size
   - Set up proper indexes (already included in migration)
   - Monitor query performance
   - Consider read replicas for scaling

3. **Backup**
   - Set up regular automated backups
   - Test restore procedures
   - Keep backups in different regions