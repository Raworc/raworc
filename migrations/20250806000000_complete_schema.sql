-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create session_state enum type if it doesn't exist
DO $$ BEGIN
    CREATE TYPE session_state AS ENUM ('INIT', 'READY', 'IDLE', 'BUSY', 'ERROR');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create message_role enum type if it doesn't exist
DO $$ BEGIN
    CREATE TYPE message_role AS ENUM ('USER', 'AGENT', 'SYSTEM');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Service Accounts table
CREATE TABLE IF NOT EXISTS service_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    workspace VARCHAR(255),
    password_hash TEXT NOT NULL,
    description TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMPTZ,
    CONSTRAINT service_accounts_name_check CHECK (name ~ '^[a-zA-Z0-9_.-]+$'),
    CONSTRAINT service_accounts_workspace_check CHECK (workspace IS NULL OR workspace ~ '^[a-zA-Z0-9_.-]+$')
);

CREATE INDEX idx_service_accounts_name ON service_accounts(name);
CREATE INDEX idx_service_accounts_active ON service_accounts(active);
CREATE INDEX idx_service_accounts_workspace ON service_accounts(workspace);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_roles_name ON roles(name);

-- Role Bindings table (many-to-many between subjects and roles)
CREATE TABLE IF NOT EXISTS role_bindings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_name VARCHAR(255) NOT NULL REFERENCES roles(name) ON DELETE CASCADE,
    principal_name VARCHAR(255) NOT NULL,
    principal_type VARCHAR(50) NOT NULL CHECK (principal_type IN ('User', 'ServiceAccount')),
    workspace VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_role_binding UNIQUE(role_name, principal_name, principal_type, workspace)
);

CREATE INDEX idx_role_bindings_role ON role_bindings(role_name);
CREATE INDEX idx_role_bindings_principal ON role_bindings(principal_name, principal_type);
CREATE INDEX idx_role_bindings_workspace ON role_bindings(workspace);

-- Agents table
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    workspace VARCHAR(255) NOT NULL DEFAULT 'default',
    description TEXT,
    instructions TEXT NOT NULL,
    model VARCHAR(100) NOT NULL,
    tools JSONB DEFAULT '[]'::jsonb,
    routes JSONB DEFAULT '[]'::jsonb,
    guardrails JSONB DEFAULT '[]'::jsonb,
    knowledge_bases JSONB DEFAULT '[]'::jsonb,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT agents_name_check CHECK (name ~ '^[a-zA-Z0-9_.-]+$'),
    CONSTRAINT agents_workspace_check CHECK (workspace ~ '^[a-zA-Z0-9_.-]+$'),
    CONSTRAINT agents_unique_name_workspace UNIQUE(name, workspace)
);

CREATE INDEX idx_agents_name ON agents(name);
CREATE INDEX idx_agents_workspace ON agents(workspace);
CREATE INDEX idx_agents_active ON agents(active);
CREATE INDEX idx_agents_deleted_at ON agents(deleted_at);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    workspace VARCHAR(255) NOT NULL DEFAULT 'default',
    starting_prompt TEXT,
    state session_state NOT NULL DEFAULT 'INIT',
    waiting_timeout_seconds INT DEFAULT 300,
    container_id VARCHAR(255),
    persistent_volume_id VARCHAR(255),
    created_by VARCHAR(255) NOT NULL,
    parent_session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    agents JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ,
    terminated_at TIMESTAMPTZ,
    termination_reason TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_sessions_workspace ON sessions(workspace);
CREATE INDEX idx_sessions_created_by ON sessions(created_by);
CREATE INDEX idx_sessions_state ON sessions(state);
CREATE INDEX idx_sessions_parent ON sessions(parent_session_id);
CREATE INDEX idx_sessions_deleted_at ON sessions(deleted_at);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_container_id ON sessions(container_id);

-- Session Agents junction table
CREATE TABLE IF NOT EXISTS session_agents (
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    configuration JSONB DEFAULT '{}',
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (session_id, agent_id)
);

-- Session Messages table
CREATE TABLE IF NOT EXISTS session_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role message_role NOT NULL,
    content TEXT NOT NULL,
    agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT messages_agent_fk FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    CONSTRAINT check_agent_id_for_agent_role CHECK (
        (role != 'AGENT') OR (agent_id IS NOT NULL)
    )
);

CREATE INDEX idx_session_messages_session_id ON session_messages(session_id);
CREATE INDEX idx_session_messages_role ON session_messages(role);
CREATE INDEX idx_session_messages_agent_id ON session_messages(agent_id);
CREATE INDEX idx_session_messages_created_at ON session_messages(created_at DESC);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_service_accounts_updated_at BEFORE UPDATE ON service_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- View for active sessions by organization
CREATE VIEW active_sessions_by_org AS
SELECT 
    workspace AS org,
    COUNT(DISTINCT id) AS session_count,
    COUNT(DISTINCT created_by) AS unique_users,
    COUNT(CASE WHEN state = 'READY' THEN 1 END) AS ready_count,
    COUNT(CASE WHEN state = 'BUSY' THEN 1 END) AS busy_count,
    COUNT(CASE WHEN state = 'IDLE' THEN 1 END) AS idle_count
FROM sessions
WHERE deleted_at IS NULL
GROUP BY workspace;

-- View for agent usage metrics
CREATE VIEW agent_usage_metrics AS
SELECT 
    a.id AS agent_id,
    a.name AS agent_name,
    a.workspace,
    COUNT(DISTINCT s.id) AS total_sessions,
    COUNT(DISTINCT s.created_by) AS unique_users,
    COUNT(DISTINCT DATE(s.created_at)) AS days_active,
    MAX(s.created_at) AS last_used_at
FROM agents a
LEFT JOIN session_agents sa ON a.id = sa.agent_id
LEFT JOIN sessions s ON sa.session_id = s.id
WHERE a.deleted_at IS NULL
GROUP BY a.id, a.name, a.workspace;

-- Function to update session message count and last activity
CREATE OR REPLACE FUNCTION update_session_message_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE sessions 
        SET last_activity_at = CURRENT_TIMESTAMP
        WHERE id = NEW.session_id;
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to update session on new message
CREATE TRIGGER update_session_on_message AFTER INSERT ON session_messages
    FOR EACH ROW EXECUTE FUNCTION update_session_message_count();

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID,
    actor VARCHAR(255) NOT NULL,
    actor_type VARCHAR(50) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    details JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_actor ON audit_log(actor);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);

-- Function for cascading session deletion
CREATE OR REPLACE FUNCTION cascade_session_deletion()
RETURNS TRIGGER AS $$
BEGIN
    -- Mark session as deleted instead of actually deleting
    UPDATE sessions 
    SET deleted_at = CURRENT_TIMESTAMP,
        termination_reason = 'User requested deletion'
    WHERE id = OLD.id;
    
    -- Return NULL to prevent actual deletion
    RETURN NULL;
END;
$$ language 'plpgsql';

-- Trigger for soft delete on sessions
CREATE TRIGGER soft_delete_session 
BEFORE DELETE ON sessions
FOR EACH ROW EXECUTE FUNCTION cascade_session_deletion();