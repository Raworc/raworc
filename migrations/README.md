# Raworc Database Migrations

This directory contains SQL migrations for Raworc, managed by sqlx.

## Migration Commands

```bash
# Run all pending migrations
raworc migrate up

# Check migration status
raworc migrate status

# Run migrations (alternative)
raworc migrate
```

## For Existing Databases

If you have an existing Raworc database from before sqlx migrations were introduced, run:

```bash
./scripts/migrate-existing-db.sh
```

This will mark your existing schema as migrated without re-running the migrations.

## Creating New Migrations

To create a new migration:

1. Create a new SQL file with the naming pattern: `YYYYMMDDHHMMSS_description.sql`
2. Write your SQL changes in the file
3. Run `raworc migrate up` to apply it

Example:
```bash
echo "ALTER TABLE agents ADD COLUMN new_field TEXT;" > migrations/$(date +%Y%m%d%H%M%S)_add_agent_field.sql
```

## Migration File Format

- Migrations are plain SQL files
- They run in order based on the timestamp prefix
- Each migration runs in a transaction
- Failed migrations will be rolled back automatically

## Notes

- Migrations are automatically run when starting the server
- The `_sqlx_migrations` table tracks which migrations have been applied
- Migrations are idempotent - running them multiple times is safe