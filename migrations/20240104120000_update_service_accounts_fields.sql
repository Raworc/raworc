-- Update service_accounts table to match the struct fields

-- Rename email column to description since that's what it's being used for
ALTER TABLE service_accounts RENAME COLUMN email TO description;

-- Add active column with default true
ALTER TABLE service_accounts ADD COLUMN active BOOLEAN DEFAULT true;

-- Add last_login_at column
ALTER TABLE service_accounts ADD COLUMN last_login_at TIMESTAMPTZ;

-- Create index on active for filtering
CREATE INDEX idx_service_accounts_active ON service_accounts(active);

-- Create index on last_login_at for reporting
CREATE INDEX idx_service_accounts_last_login ON service_accounts(last_login_at);