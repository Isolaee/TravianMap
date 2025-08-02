-- Travian Map Database Initialization
-- This file is automatically executed when the PostgreSQL container starts

-- Create the villages table with additional fields for future expansion
CREATE TABLE IF NOT EXISTS villages (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    population INTEGER NOT NULL DEFAULT 0,
    tribe VARCHAR(50),
    player_name VARCHAR(255),
    alliance VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_villages_coordinates ON villages (x, y);
CREATE INDEX IF NOT EXISTS idx_villages_population ON villages (population);
CREATE INDEX IF NOT EXISTS idx_villages_player ON villages (player_name);
CREATE INDEX IF NOT EXISTS idx_villages_alliance ON villages (alliance);

-- Create a function to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create a trigger to automatically update updated_at
CREATE TRIGGER update_villages_updated_at 
    BEFORE UPDATE ON villages 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Insert some sample data
INSERT INTO villages (name, x, y, population, tribe, player_name, alliance) VALUES
    ('Capital Village', 0, 0, 1000, 'Romans', 'Emperor', 'Roman Empire'),
    ('North Settlement', 5, 10, 500, 'Gauls', 'Warrior', 'Gaul Alliance'),
    ('East Outpost', 15, 3, 300, 'Teutons', 'Barbarian', 'Wild Ones'),
    ('South Trading Post', -8, -12, 750, 'Romans', 'Merchant', 'Traders Guild'),
    ('Western Fort', -20, 5, 600, 'Teutons', 'Defender', 'Iron Shield'),
    ('Mountain Village', 25, 25, 800, 'Gauls', 'Highlander', 'Mountain Clan'),
    ('River Crossing', -15, 8, 450, 'Romans', 'Engineer', 'Bridge Builders'),
    ('Forest Camp', 12, -5, 350, 'Gauls', 'Ranger', 'Forest Guard'),
    ('Desert Oasis', 30, -20, 200, 'Egyptians', 'Nomad', 'Desert Wind'),
    ('Coastal Town', -25, -15, 900, 'Romans', 'Admiral', 'Naval Force')
ON CONFLICT DO NOTHING;

-- Create a view for villages with distance calculations (for future use)
CREATE OR REPLACE VIEW villages_with_distance AS
SELECT 
    v.*,
    SQRT(POWER(v.x, 2) + POWER(v.y, 2)) as distance_from_origin
FROM villages v;

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE travian_map TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;
