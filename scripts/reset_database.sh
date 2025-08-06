#!/bin/bash

# Database Reset Script
# This script drops and recreates the database with the new schema

set -e

# Configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-raworc}"
DB_USER="${DB_USER:-raworc}"
DB_PASS="${DB_PASS:-raworc}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}⚠️  WARNING: This will completely reset the database!${NC}"
echo -e "${YELLOW}Database: $DB_NAME on $DB_HOST:$DB_PORT${NC}"
read -p "Are you sure you want to continue? (yes/no): " -r
echo

if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo -e "${RED}Aborted.${NC}"
    exit 1
fi

# Export password for psql
export PGPASSWORD=$DB_PASS

echo -e "${YELLOW}Dropping existing database...${NC}"
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;" || true

echo -e "${GREEN}Creating new database...${NC}"
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d postgres -c "CREATE DATABASE $DB_NAME;"

echo -e "${GREEN}Running migrations...${NC}"

# Check if sqlx is available and use it, otherwise fall back to direct SQL
if command -v sqlx &> /dev/null; then
    export DATABASE_URL="postgresql://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME"
    if sqlx migrate run 2>/dev/null; then
        echo -e "${GREEN}Migrations applied via sqlx${NC}"
    else
        # Fallback to direct SQL if sqlx fails
        psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME < migrations/20250806000000_initial_schema.sql
        echo -e "${GREEN}Migrations applied via psql${NC}"
    fi
else
    # No sqlx available, use direct SQL
    psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME < migrations/20250806000000_initial_schema.sql
    echo -e "${GREEN}Migrations applied via psql${NC}"
fi

echo -e "${GREEN}✅ Database reset complete!${NC}"
echo
echo "Default credentials:"
echo "  Username: admin"
echo "  Password: admin"
echo
echo -e "${YELLOW}⚠️  Remember to change the default password!${NC}"