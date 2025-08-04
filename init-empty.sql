-- Travian Map Database - Empty Schema Initialization
-- This file creates the base schema without any sample data

-- Create the servers table
CREATE TABLE IF NOT EXISTS servers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    url VARCHAR(512) NOT NULL,
    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for the servers table
CREATE INDEX IF NOT EXISTS idx_servers_active ON servers (is_active);
CREATE INDEX IF NOT EXISTS idx_servers_name ON servers (name);

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE travian_map TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;

-- Set up automatic updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for servers table
DROP TRIGGER IF EXISTS update_servers_updated_at ON servers;
CREATE TRIGGER update_servers_updated_at
    BEFORE UPDATE ON servers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create a function to validate server URLs
CREATE OR REPLACE FUNCTION validate_server_url(url_input TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    -- Check if URL is valid (basic validation)
    IF url_input IS NULL OR LENGTH(TRIM(url_input)) = 0 THEN
        RETURN FALSE;
    END IF;
    
    -- Check if URL starts with http or https
    IF NOT (url_input ~* '^https?://') THEN
        RETURN FALSE;
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Create a view for active servers
CREATE OR REPLACE VIEW active_servers AS
SELECT id, name, url, created_at, updated_at
FROM servers 
WHERE is_active = TRUE;

-- Log the initialization
DO $$
BEGIN
    RAISE NOTICE 'Travian Map Database initialized successfully with empty schema';
    RAISE NOTICE 'Database ready for server and village data';
END $$;
