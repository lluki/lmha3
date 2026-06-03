-- Add is_admin column to tenants
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- Set existing admin user to have is_admin = TRUE
UPDATE tenants SET is_admin = TRUE WHERE username = 'admin';
