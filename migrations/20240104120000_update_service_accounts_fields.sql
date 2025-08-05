-- Update service_accounts table to add new fields

-- Add active column with default true
ALTER TABLE service_accounts ADD COLUMN active BOOLEAN DEFAULT true;

-- Add last_login_at column
ALTER TABLE service_accounts ADD COLUMN last_login_at TIMESTAMPTZ;

-- Create index on active for filtering
CREATE INDEX idx_service_accounts_active ON service_accounts(active);

-- Create index on last_login_at for reporting
CREATE INDEX idx_service_accounts_last_login ON service_accounts(last_login_at);