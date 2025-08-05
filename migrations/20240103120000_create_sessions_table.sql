-- Create session lifecycle enum
CREATE TYPE session_lifecycle AS ENUM ('NOT_STARTED', 'STARTED', 'BUSY', 'WAITING', 'TERMINATED');

-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    starting_prompt TEXT NOT NULL,
    lifecycle_state session_lifecycle NOT NULL DEFAULT 'NOT_STARTED',
    waiting_timeout_seconds INTEGER DEFAULT 300, -- 5 minutes default
    container_id VARCHAR(255),
    persistent_volume_id VARCHAR(255),
    created_by VARCHAR(255) NOT NULL,
    parent_session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    last_activity_at TIMESTAMP WITH TIME ZONE,
    terminated_at TIMESTAMP WITH TIME ZONE,
    termination_reason VARCHAR(255),
    metadata JSONB DEFAULT '{}'::jsonb,
    deleted_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT check_lifecycle_timestamps CHECK (
        (lifecycle_state = 'NOT_STARTED' AND started_at IS NULL) OR
        (lifecycle_state != 'NOT_STARTED' AND started_at IS NOT NULL)
    ),
    CONSTRAINT check_terminated_timestamps CHECK (
        (lifecycle_state = 'TERMINATED' AND terminated_at IS NOT NULL) OR
        (lifecycle_state != 'TERMINATED' AND terminated_at IS NULL)
    )
);

-- Create session_agents junction table for many-to-many relationship
CREATE TABLE IF NOT EXISTS session_agents (
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    configuration JSONB DEFAULT '{}'::jsonb,
    PRIMARY KEY (session_id, agent_id)
);

-- Create indexes
CREATE INDEX idx_sessions_lifecycle_state ON sessions(lifecycle_state);
CREATE INDEX idx_sessions_created_by ON sessions(created_by);
CREATE INDEX idx_sessions_parent_session_id ON sessions(parent_session_id);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
CREATE INDEX idx_sessions_last_activity_at ON sessions(last_activity_at DESC);
CREATE INDEX idx_sessions_deleted_at ON sessions(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_session_agents_session_id ON session_agents(session_id);
CREATE INDEX idx_session_agents_agent_id ON session_agents(agent_id);

-- Create updated_at trigger for sessions
CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE
    ON sessions FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();