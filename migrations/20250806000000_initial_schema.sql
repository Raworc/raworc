-- Initial Raworc Database Schema
-- This creates all tables with proper namespace architecture from the beginning
-- Namespaces represent organizations and apply to resources (agents, sessions)
-- RBAC entities (service accounts, roles) are global

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- RBAC Tables (Global entities)
-- =====================================================

-- Service Accounts (Global users that can work across organizations)
CREATE TABLE IF NOT EXISTS service_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,  -- Username
    password_hash VARCHAR(255) NOT NULL,         -- Hashed password
    description TEXT,                    -- Optional description
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT service_accounts_name_check CHECK (name ~ '^[a-zA-Z0-9_.-]+$')
);

CREATE INDEX idx_service_accounts_name ON service_accounts(name);
CREATE INDEX idx_service_accounts_active ON service_accounts(active);

COMMENT ON TABLE service_accounts IS 'Global user accounts that can be granted access to multiple organizations';
COMMENT ON COLUMN service_accounts.name IS 'Unique username across the entire platform';
COMMENT ON COLUMN service_accounts.description IS 'Optional description of the account purpose';

-- Roles (Global permission templates)
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,  -- Role name
    description TEXT,
    rules JSONB NOT NULL DEFAULT '[]'::JSONB,  -- Array of permission rules
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT roles_name_check CHECK (name ~ '^[a-zA-Z0-9_.-]+$'),
    CONSTRAINT roles_rules_check CHECK (jsonb_typeof(rules) = 'array')
);

CREATE INDEX idx_roles_name ON roles(name);

COMMENT ON TABLE roles IS 'Global role definitions that can be used across all organizations';
COMMENT ON COLUMN roles.rules IS 'JSON array of rules: [{api_groups: [], resources: [], verbs: []}]';

-- Role Bindings (Connect roles to users and specify WHERE they apply)
CREATE TABLE IF NOT EXISTS role_bindings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_name VARCHAR(255) NOT NULL,
    principal_name VARCHAR(255) NOT NULL,
    principal_type VARCHAR(50) NOT NULL DEFAULT 'ServiceAccount',
    namespace VARCHAR(255),  -- NULL means global (all namespaces)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT role_bindings_principal_type_check 
        CHECK (principal_type IN ('ServiceAccount', 'User', 'Group')),
    CONSTRAINT role_bindings_unique_binding 
        UNIQUE (role_name, principal_name, principal_type, namespace)
);

CREATE INDEX idx_role_bindings_role ON role_bindings(role_name);
CREATE INDEX idx_role_bindings_principal ON role_bindings(principal_name, principal_type);
CREATE INDEX idx_role_bindings_namespace ON role_bindings(namespace);

COMMENT ON TABLE role_bindings IS 'Grants roles to principals within specific organizations or globally';
COMMENT ON COLUMN role_bindings.namespace IS 'Organization where role applies. NULL = global (all organizations)';

-- =====================================================
-- Resource Tables (Belong to organizations)
-- =====================================================

-- Agents (AI agents belong to organizations)
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',  -- Organization that owns this agent
    description TEXT,
    instructions TEXT NOT NULL,
    model VARCHAR(100) NOT NULL,
    tools JSONB DEFAULT '[]'::JSONB,
    routes JSONB DEFAULT '[]'::JSONB,
    guardrails JSONB DEFAULT '[]'::JSONB,
    knowledge_bases JSONB DEFAULT '[]'::JSONB,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT agents_unique_name_namespace UNIQUE (name, namespace),
    CONSTRAINT agents_name_check CHECK (name ~ '^[a-zA-Z0-9_.-]+$'),
    CONSTRAINT agents_namespace_check CHECK (namespace ~ '^[a-zA-Z0-9_.-]+$')
);

CREATE INDEX idx_agents_namespace ON agents(namespace);
CREATE INDEX idx_agents_name ON agents(name);
CREATE INDEX idx_agents_active ON agents(active);
CREATE INDEX idx_agents_deleted_at ON agents(deleted_at);

COMMENT ON TABLE agents IS 'AI agents that belong to specific organizations';
COMMENT ON COLUMN agents.namespace IS 'Organization that owns this agent';

-- Sessions (Work sessions belong to organizations)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL DEFAULT 'default',  -- Organization that owns this session
    starting_prompt TEXT,
    lifecycle_state VARCHAR(50) NOT NULL DEFAULT 'NOT_STARTED',
    waiting_timeout_seconds INTEGER DEFAULT 300,
    container_id VARCHAR(255),
    persistent_volume_id VARCHAR(255),
    created_by VARCHAR(255) NOT NULL,  -- Username who created it
    parent_session_id UUID,
    agents JSONB DEFAULT '[]'::JSONB,  -- Array of agent references
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    last_activity_at TIMESTAMP WITH TIME ZONE,
    terminated_at TIMESTAMP WITH TIME ZONE,
    termination_reason TEXT,
    metadata JSONB DEFAULT '{}'::JSONB,
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT sessions_lifecycle_state_check 
        CHECK (lifecycle_state IN ('NOT_STARTED', 'STARTED', 'BUSY', 'WAITING', 'TERMINATED')),
    CONSTRAINT sessions_namespace_check CHECK (namespace ~ '^[a-zA-Z0-9_.-]+$'),
    CONSTRAINT sessions_parent_fk FOREIGN KEY (parent_session_id) 
        REFERENCES sessions(id) ON DELETE SET NULL
);

CREATE INDEX idx_sessions_namespace ON sessions(namespace);
CREATE INDEX idx_sessions_created_by ON sessions(created_by);
CREATE INDEX idx_sessions_lifecycle_state ON sessions(lifecycle_state);
CREATE INDEX idx_sessions_parent_session ON sessions(parent_session_id);
CREATE INDEX idx_sessions_deleted_at ON sessions(deleted_at);
CREATE INDEX idx_sessions_namespace_state ON sessions(namespace, lifecycle_state);

COMMENT ON TABLE sessions IS 'Work sessions that belong to specific organizations';
COMMENT ON COLUMN sessions.namespace IS 'Organization that owns this session';
COMMENT ON COLUMN sessions.parent_session_id IS 'Reference to parent session for remixed sessions';

-- =====================================================
-- Initial System Data
-- =====================================================

-- Create default roles
INSERT INTO roles (name, description, rules) VALUES
    ('admin', 'Full administrative access', 
     '[{"api_groups": ["*"], "resources": ["*"], "verbs": ["*"]}]'::JSONB),
    ('developer', 'Developer access', 
     '[{"api_groups": ["api"], "resources": ["agents", "sessions"], "verbs": ["get", "list", "create", "update", "delete"]}]'::JSONB),
    ('viewer', 'Read-only access', 
     '[{"api_groups": ["api"], "resources": ["*"], "verbs": ["get", "list"]}]'::JSONB),
    ('operator', 'Session operator access',
     '[{"api_groups": ["api"], "resources": ["sessions"], "verbs": ["*"]}, 
       {"api_groups": ["api"], "resources": ["agents"], "verbs": ["get", "list"]}]'::JSONB)
ON CONFLICT (name) DO NOTHING;

-- Create default admin account (password: admin - CHANGE THIS!)
-- Password is bcrypt hashed
INSERT INTO service_accounts (name, password_hash, description) VALUES
    ('admin', '$2b$12$GULotn07538rBIG/EoE.8euIJwepsVnqJbb1HSgUi4RYdzcxj0DUG', 'Default administrator account')
ON CONFLICT (name) DO NOTHING;

-- Grant admin role globally to admin user
INSERT INTO role_bindings (role_name, principal_name, principal_type, namespace) VALUES
    ('admin', 'admin', 'ServiceAccount', NULL)  -- NULL namespace = global access
ON CONFLICT (role_name, principal_name, principal_type, namespace) DO NOTHING;

-- =====================================================
-- Functions for automatic timestamp updates
-- =====================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for updated_at
CREATE TRIGGER update_service_accounts_updated_at BEFORE UPDATE ON service_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =====================================================
-- Views for easier querying
-- =====================================================

-- Active agents per organization
CREATE OR REPLACE VIEW active_agents_by_org AS
SELECT 
    namespace as organization,
    COUNT(*) as agent_count
FROM agents
WHERE active = true AND deleted_at IS NULL
GROUP BY namespace;

-- Active sessions per organization
CREATE OR REPLACE VIEW active_sessions_by_org AS
SELECT 
    namespace as organization,
    lifecycle_state,
    COUNT(*) as session_count
FROM sessions
WHERE deleted_at IS NULL
GROUP BY namespace, lifecycle_state;

-- User access summary
CREATE OR REPLACE VIEW user_access_summary AS
SELECT 
    rb.principal_name as username,
    rb.role_name as role,
    COALESCE(rb.namespace, '*GLOBAL*') as organization,
    rb.created_at as granted_at
FROM role_bindings rb
WHERE rb.principal_type = 'ServiceAccount'
ORDER BY rb.principal_name, rb.namespace;

COMMENT ON VIEW user_access_summary IS 'Shows which users have which roles in which organizations';