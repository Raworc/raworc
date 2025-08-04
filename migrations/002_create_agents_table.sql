-- Create agents table
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    instructions TEXT NOT NULL,
    model VARCHAR(255) NOT NULL,
    tools JSONB DEFAULT '[]'::jsonb,
    routes JSONB DEFAULT '[]'::jsonb,
    guardrails JSONB DEFAULT '[]'::jsonb,
    knowledge_bases JSONB DEFAULT '[]'::jsonb,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on name for fast lookups
CREATE INDEX idx_agents_name ON agents(name);

-- Create index on active for filtering
CREATE INDEX idx_agents_active ON agents(active);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE
    ON agents FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Insert a default agent for testing
INSERT INTO agents (name, description, instructions, model) VALUES
    ('assistant', 'General purpose assistant agent', 'You are a helpful AI assistant. Be concise and accurate in your responses.', 'gpt-4');