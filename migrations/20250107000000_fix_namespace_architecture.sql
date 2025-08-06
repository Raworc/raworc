-- Fix namespace architecture: Move namespaces from RBAC entities to resources
-- and update role bindings to specify namespace scope

-- Remove namespace from service_accounts
ALTER TABLE service_accounts DROP CONSTRAINT service_accounts_name_namespace_key;
ALTER TABLE service_accounts DROP COLUMN namespace;
ALTER TABLE service_accounts ADD CONSTRAINT service_accounts_name_key UNIQUE (name);

-- Remove namespace from roles  
ALTER TABLE roles DROP CONSTRAINT roles_name_namespace_key;
ALTER TABLE roles DROP COLUMN namespace;
ALTER TABLE roles ADD CONSTRAINT roles_name_key UNIQUE (name);

-- Update role_bindings to use namespace as scope (where role applies)
-- namespace NULL means global binding (applies to all namespaces)
ALTER TABLE role_bindings DROP CONSTRAINT role_bindings_name_namespace_key;
ALTER TABLE role_bindings ALTER COLUMN namespace DROP NOT NULL;
COMMENT ON COLUMN role_bindings.namespace IS 'Namespace where this role binding applies. NULL means global (all namespaces)';

-- Add unique constraint for role_bindings
ALTER TABLE role_bindings ADD CONSTRAINT role_bindings_role_principal_namespace_key 
  UNIQUE (role_name, principal_name, principal_type, namespace);

-- Add namespace to agents table
ALTER TABLE agents ADD COLUMN namespace TEXT NOT NULL DEFAULT 'default';
CREATE INDEX idx_agents_namespace ON agents(namespace);
COMMENT ON COLUMN agents.namespace IS 'Namespace that owns this agent';

-- Add namespace to sessions table  
ALTER TABLE sessions ADD COLUMN namespace TEXT NOT NULL DEFAULT 'default';
CREATE INDEX idx_sessions_namespace ON sessions(namespace);
COMMENT ON COLUMN sessions.namespace IS 'Namespace that owns this session';

-- Drop old indexes that are no longer needed
DROP INDEX IF EXISTS idx_service_accounts_namespace;
DROP INDEX IF EXISTS idx_roles_namespace;

-- Update existing role_bindings to be global (since they were system-wide before)
UPDATE role_bindings SET namespace = NULL WHERE namespace = 'default';