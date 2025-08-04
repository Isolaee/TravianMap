-- ===================================================================
-- TRAVIAN MAP DATABASE SCHEMA - MINIMAL VERSION
-- ===================================================================

-- ===================================================================
-- 1. SERVERS TABLE
-- ===================================================================

CREATE TABLE IF NOT EXISTS servers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    url VARCHAR(512) NOT NULL,
    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ===================================================================
-- 2. VILLAGES TABLE TEMPLATE
-- ===================================================================
-- Dynamic table structure: villages_server_{server_id}_{date}

/*
CREATE TABLE villages_server_{server_id}_{date} (
    id SERIAL,
    date DATE NOT NULL,
    server_id INTEGER NOT NULL,
    worldid INTEGER,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    tid INTEGER,
    vid INTEGER,
    village VARCHAR(255) NOT NULL,
    uid INTEGER,
    player VARCHAR(255),
    aid INTEGER,
    alliance VARCHAR(255),
    population INTEGER NOT NULL DEFAULT 0,
    capital VARCHAR(10),
    isWW BOOLEAN DEFAULT FALSE,
    wwname VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (id, date)
);
*/

-- ===================================================================
-- 3. INDEXES
-- ===================================================================

-- Servers table indexes
CREATE INDEX IF NOT EXISTS idx_servers_active ON servers (is_active);
CREATE INDEX IF NOT EXISTS idx_servers_name ON servers (name);

-- Village table indexes (created dynamically by application):
-- - idx_{table}_coordinates ON (server_id, x, y)
-- - idx_{table}_population ON (server_id, population)
-- - idx_{table}_player ON (server_id, player)
-- - idx_{table}_alliance ON (server_id, alliance)

