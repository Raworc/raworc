-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create service_accounts table
CREATE TABLE service_accounts (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    namespace TEXT NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create roles table
CREATE TABLE roles (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL,
    namespace TEXT NOT NULL,
    rules JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(name, namespace)
);

-- Create role_bindings table
CREATE TABLE role_bindings (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT NOT NULL,
    namespace TEXT NOT NULL,
    role_name TEXT NOT NULL,
    subjects JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(name, namespace)
);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add updated_at triggers
CREATE TRIGGER update_service_accounts_updated_at BEFORE UPDATE ON service_accounts
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_role_bindings_updated_at BEFORE UPDATE ON role_bindings
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Create indexes for performance
CREATE INDEX idx_service_accounts_namespace ON service_accounts(namespace);
CREATE INDEX idx_service_accounts_email ON service_accounts(email);
CREATE INDEX idx_roles_namespace ON roles(namespace);
CREATE INDEX idx_role_bindings_namespace ON role_bindings(namespace);
CREATE INDEX idx_role_bindings_subjects ON role_bindings USING GIN (subjects);

-- Enable Row Level Security
ALTER TABLE service_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE role_bindings ENABLE ROW LEVEL SECURITY;

-- Create RLS policies (for now, we'll use service role for all operations)
-- These can be refined later based on Supabase Auth integration
CREATE POLICY "Service role can do everything" ON service_accounts
    FOR ALL USING (true);

CREATE POLICY "Service role can do everything" ON roles
    FOR ALL USING (true);

CREATE POLICY "Service role can do everything" ON role_bindings
    FOR ALL USING (true);