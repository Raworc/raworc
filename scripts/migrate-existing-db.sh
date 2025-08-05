#!/bin/bash

# Script to help migrate existing Raworc databases to sqlx migration system

echo "Raworc Database Migration Helper"
echo "================================"
echo
echo "This script helps transition existing Raworc databases to use sqlx migrations."
echo

# Get database URL
DATABASE_URL="${DATABASE_URL:-postgresql://postgres@localhost/raworc}"
echo "Database URL: $DATABASE_URL"
echo

# Check if _sqlx_migrations table exists
echo "Checking for existing sqlx migrations table..."
if psql "$DATABASE_URL" -c "SELECT 1 FROM _sqlx_migrations LIMIT 1;" &>/dev/null; then
    echo "✓ sqlx migrations table already exists"
    echo
    echo "Your database is already using sqlx migrations."
    echo "Run 'raworc migrate status' to see migration status."
    exit 0
fi

# Check if tables exist
echo "Checking for existing Raworc tables..."
TABLES_EXIST=false
if psql "$DATABASE_URL" -c "SELECT 1 FROM service_accounts LIMIT 1;" &>/dev/null; then
    TABLES_EXIST=true
    echo "✓ Found existing Raworc tables"
fi

if [ "$TABLES_EXIST" = true ]; then
    echo
    echo "Your database has existing tables that need to be marked as migrated."
    echo
    echo "Creating sqlx migrations table..."
    
    # Create the migrations table
    psql "$DATABASE_URL" << 'EOF'
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
);
EOF

    echo "✓ Created _sqlx_migrations table"
    echo
    echo "Marking existing migrations as applied..."
    
    # Calculate checksums for each migration file
    MIGRATION_DIR="$(dirname "$0")/../migrations"
    
    for migration in "$MIGRATION_DIR"/*.sql; do
        if [ -f "$migration" ]; then
            filename=$(basename "$migration")
            version=$(echo "$filename" | cut -d'_' -f1)
            description=$(echo "$filename" | sed 's/^[0-9]*_//' | sed 's/.sql$//' | tr '_' ' ')
            checksum=$(sha256sum "$migration" | cut -d' ' -f1 | xxd -r -p | base64)
            
            echo "  - $version: $description"
            
            psql "$DATABASE_URL" << EOF
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
VALUES ($version, '$description', true, decode('$checksum', 'base64'), 0)
ON CONFLICT (version) DO NOTHING;
EOF
        fi
    done
    
    echo
    echo "✓ Successfully marked existing migrations as applied"
    echo
    echo "You can now use 'raworc migrate' commands and start the server normally."
else
    echo
    echo "No existing tables found. You can run 'raworc migrate up' to create them."
fi