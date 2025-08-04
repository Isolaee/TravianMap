use sqlx::{PgPool, Row};
use anyhow::Result;
use crate::MapData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub is_active: bool,
}

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

fn get_table_name_for_server_and_date(server_id: i32, date: chrono::NaiveDate) -> String {
    format!("villages_server_{}_{}", server_id, date.format("%Y_%m_%d"))
}

fn get_table_name_for_date(date: chrono::NaiveDate) -> String {
    // Default server (id = 1) for backward compatibility
    get_table_name_for_server_and_date(1, date)
}

fn get_today_table_name() -> String {
    let today = chrono::Utc::now().date_naive();
    get_table_name_for_date(today)
}

pub async fn create_table_for_server_and_date(pool: &PgPool, server_id: i32, date: chrono::NaiveDate) -> Result<String> {
    let table_name = get_table_name_for_server_and_date(server_id, date);
    
    // Create the villages table with Travian x_world structure for the specific server and date
    let create_query = format!(
        r#"
        CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
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
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
        table_name
    );
    
    sqlx::query(&create_query)
        .execute(pool)
        .await?;

    // Create indexes for the new table
    let coord_index = format!("CREATE INDEX IF NOT EXISTS idx_{}_coordinates ON {} (server_id, x, y)", table_name, table_name);
    sqlx::query(&coord_index).execute(pool).await?;

    let pop_index = format!("CREATE INDEX IF NOT EXISTS idx_{}_population ON {} (server_id, population)", table_name, table_name);
    sqlx::query(&pop_index).execute(pool).await?;

    let world_index = format!("CREATE INDEX IF NOT EXISTS idx_{}_worldid ON {} (server_id, worldid)", table_name, table_name);
    sqlx::query(&world_index).execute(pool).await?;

    Ok(table_name)
}

pub async fn create_table_for_date(pool: &PgPool, date: chrono::NaiveDate) -> Result<String> {
    // Default to server_id = 1 for backward compatibility
    create_table_for_server_and_date(pool, 1, date).await
}

pub async fn create_tables(pool: &PgPool) -> Result<()> {
    // Create the servers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS servers (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            url VARCHAR(512) NOT NULL,
            is_active BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create the default villages table (for backward compatibility)
    let today = chrono::Utc::now().date_naive();
    create_table_for_date(pool, today).await?;
    Ok(())
}

pub async fn get_available_dates(pool: &PgPool) -> Result<Vec<(chrono::NaiveDate, i32)>> {
    // Query for all tables that match the villages_YYYY_MM_DD pattern
    let rows = sqlx::query(
        r#"
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE 'villages_%' 
        AND table_name ~ '^villages_[0-9]{4}_[0-9]{2}_[0-9]{2}$'
        ORDER BY table_name DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for row in rows {
        let table_name: String = row.get("table_name");
        
        // Extract date from table name (format: villages_YYYY_MM_DD)
        if let Some(date_part) = table_name.strip_prefix("villages_") {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_part, "%Y_%m_%d") {
                // Get village count for this table
                let count_query = format!("SELECT COUNT(*) FROM {}", table_name);
                let count: i64 = sqlx::query_scalar(&count_query)
                    .fetch_one(pool)
                    .await?;
                
                result.push((date, count as i32));
            }
        }
    }
    
    Ok(result)
}

pub async fn cleanup_old_tables(pool: &PgPool) -> Result<()> {
    let available_dates = get_available_dates(pool).await?;
    
    // Keep only the last 10 tables
    if available_dates.len() > 10 {
        let tables_to_drop = &available_dates[10..];
        
        for (date, _) in tables_to_drop {
            let table_name = get_table_name_for_date(*date);
            let drop_query = format!("DROP TABLE IF EXISTS {}", table_name);
            sqlx::query(&drop_query).execute(pool).await?;
            println!("Dropped old table: {}", table_name);
        }
    }
    
    Ok(())
}

pub async fn insert_sample_data(pool: &PgPool) -> Result<()> {
    // Sample data insertion is now optional and disabled by default
    // The database starts empty and ready for real Travian server data
    println!("Sample data insertion skipped - database ready for real data");
    Ok(())
}

pub async fn get_all_villages(pool: &PgPool) -> Result<Vec<MapData>> {
    // Get the active server
    let active_server = get_active_server(pool).await?;
    
    if let Some(server) = active_server {
        get_villages_for_server(pool, server.id).await
    } else {
        Ok(Vec::new()) // No active server
    }
}

pub async fn get_villages_for_server(pool: &PgPool, server_id: i32) -> Result<Vec<MapData>> {
    // Get the latest table for this server (most recent date)
    let available_dates = get_available_dates_for_server(pool, server_id).await?;
    
    if available_dates.is_empty() {
        return Ok(Vec::new()); // No tables available for this server
    }
    
    let latest_date = available_dates[0].0;
    get_villages_by_server_and_date(pool, server_id, latest_date).await
}

pub async fn get_available_dates_for_server(pool: &PgPool, server_id: i32) -> Result<Vec<(chrono::NaiveDate, i32)>> {
    // Query for all tables that match the villages_server_{server_id}_YYYY_MM_DD pattern
    let pattern = format!("villages_server_{}_", server_id);
    let rows = sqlx::query(
        r#"
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE $1
        AND table_name ~ $2
        ORDER BY table_name DESC
        "#
    )
    .bind(format!("{}%", pattern))
    .bind(format!("^villages_server_{}_[0-9]{{4}}_[0-9]{{2}}_[0-9]{{2}}$", server_id))
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    
    for row in rows {
        let table_name: String = row.get("table_name");
        
        // Extract date from table name (format: villages_server_{server_id}_YYYY_MM_DD)
        if let Some(date_part) = table_name.strip_prefix(&format!("villages_server_{}_", server_id)) {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_part, "%Y_%m_%d") {
                // Get village count for this table
                let count_query = format!("SELECT COUNT(*) FROM {} WHERE server_id = $1", table_name);
                let count: i64 = sqlx::query_scalar(&count_query)
                    .bind(server_id)
                    .fetch_one(pool)
                    .await?;
                
                result.push((date, count as i32));
            }
        }
    }
    
    Ok(result)
}

pub async fn get_villages_by_server_and_date(pool: &PgPool, server_id: i32, date: chrono::NaiveDate) -> Result<Vec<MapData>> {
    let table_name = get_table_name_for_server_and_date(server_id, date);
    
    // Check if table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
    )
    .bind(&table_name)
    .fetch_one(pool)
    .await?;
    
    if !table_exists {
        return Ok(Vec::new());
    }
    
    let query = format!(
        "SELECT id, village, x, y, population, player, alliance, worldid FROM {} WHERE server_id = $1 ORDER BY population DESC",
        table_name
    );
    
    let rows = sqlx::query(&query)
        .bind(server_id)
        .fetch_all(pool)
        .await?;

    let villages: Vec<MapData> = rows
        .into_iter()
        .map(|row| MapData {
            id: row.get::<i32, _>("id") as u32,
            name: row.get("village"),
            x: row.get("x"),
            y: row.get("y"),
            population: row.get::<i32, _>("population") as u32,
            player: row.get("player"),
            alliance: row.get("alliance"),
            worldid: row.get::<Option<i32>, _>("worldid").map(|v| v as u32),
        })
        .collect();

    Ok(villages)
}

pub async fn add_village(pool: &PgPool, name: &str, x: i32, y: i32, population: u32) -> Result<MapData> {
    let row = sqlx::query(
        "INSERT INTO villages (village, x, y, population, player, alliance) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, village, x, y, population, player, alliance, worldid"
    )
    .bind(name)
    .bind(x)
    .bind(y)
    .bind(population as i32)
    .bind("Unknown Player")
    .bind("No Alliance")
    .fetch_one(pool)
    .await?;

    Ok(MapData {
        id: row.get::<i32, _>("id") as u32,
        name: row.get("village"),
        x: row.get("x"),
        y: row.get("y"),
        population: row.get::<i32, _>("population") as u32,
        player: row.get("player"),
        alliance: row.get("alliance"),
        worldid: row.get::<Option<i32>, _>("worldid").map(|v| v as u32),
    })
}

pub async fn update_village_population(pool: &PgPool, id: u32, population: u32) -> Result<Option<MapData>> {
    let result = sqlx::query(
        r#"
        UPDATE villages 
        SET population = $2, updated_at = NOW() 
        WHERE id = $1 
        RETURNING id, village, x, y, population, player, alliance, worldid
        "#
    )
    .bind(id as i32)
    .bind(population as i32)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = result {
        Ok(Some(MapData {
            id: row.get::<i32, _>("id") as u32,
            name: row.get("village"),
            x: row.get("x"),
            y: row.get("y"),
            population: row.get::<i32, _>("population") as u32,
            player: row.get("player"),
            alliance: row.get("alliance"),
            worldid: row.get::<Option<i32>, _>("worldid").map(|v| v as u32),
        }))
    } else {
        Ok(None)
    }
}

pub async fn delete_village(pool: &PgPool, id: u32) -> Result<bool> {
    let result = sqlx::query("DELETE FROM villages WHERE id = $1")
        .bind(id as i32)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn clear_todays_villages(pool: &PgPool) -> Result<()> {
    let today = chrono::Utc::now().date_naive();
    let table_name = get_table_name_for_date(today);
    
    // Check if table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
    )
    .bind(&table_name)
    .fetch_one(pool)
    .await?;
    
    if table_exists {
        let delete_query = format!("DELETE FROM {}", table_name);
        sqlx::query(&delete_query).execute(pool).await?;
    }
    
    Ok(())
}

pub async fn execute_sql_with_date_tables(pool: &PgPool, sql_content: &str) -> Result<usize> {
    // Get the active server
    let active_server = get_active_server(pool).await?;
    
    if let Some(server) = active_server {
        execute_sql_for_server(pool, sql_content, server.id).await
    } else {
        Err(anyhow::anyhow!("No active server found"))
    }
}

pub async fn execute_sql_for_server(pool: &PgPool, sql_content: &str, server_id: i32) -> Result<usize> {
    let today = chrono::Utc::now().date_naive();
    
    // Create table for today if it doesn't exist
    let table_name = create_table_for_server_and_date(pool, server_id, today).await?;
    
    // Clear existing data for today for this server
    let delete_query = format!("DELETE FROM {} WHERE server_id = $1", table_name);
    sqlx::query(&delete_query).bind(server_id).execute(pool).await?;
    
    // Parse the SQL content to extract INSERT statements for x_world table
    let mut village_count = 0;
    
    // Split by lines and process each line
    for line in sql_content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("/*") {
            continue;
        }
        
        // Look for INSERT statements for x_world table
        if trimmed.to_lowercase().contains("insert into") && 
           (trimmed.to_lowercase().contains("x_world") || trimmed.to_lowercase().contains("`x_world`")) {
            
            // Parse Travian x_world format: INSERT INTO `x_world` VALUES (22028,173,146,5,31912,'Natars 173|146′,1,'Natars',0,",498,NULL,FALSE,NULL,NULL,NULL);
            if let Some(values_start) = trimmed.find("VALUES") {
                let values_part = &trimmed[values_start + 6..].trim();
                
                // Extract the values between parentheses
                if let Some(start) = values_part.find('(') {
                    if let Some(end) = values_part.rfind(')') {
                        let values_str = &values_part[start + 1..end];
                        
                        // Parse the comma-separated values
                        if let Ok(parsed_village) = parse_x_world_values(values_str) {
                            match insert_parsed_village_to_table_with_server(pool, parsed_village, &table_name, server_id).await {
                                Ok(_) => village_count += 1,
                                Err(e) => {
                                    eprintln!("Failed to insert village: {}", e);
                                    // Continue with other villages
                                }
                            }
                        } else {
                            eprintln!("Failed to parse x_world values: {}", values_str);
                        }
                    }
                }
            }
        }
    }
    
    // Cleanup old tables (keep only last 10)
    cleanup_old_tables(pool).await?;
    
    Ok(village_count)
}

struct ParsedVillage {
    worldid: Option<i32>,
    x: i32,
    y: i32,
    tid: Option<i32>,
    vid: Option<i32>,
    village: String,
    uid: Option<i32>,
    player: Option<String>,
    aid: Option<i32>,
    alliance: Option<String>,
    population: i32,
}

fn parse_x_world_values(values_str: &str) -> Result<ParsedVillage> {
    // Split by comma, but be careful with quoted strings
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    
    for ch in values_str.chars() {
        match ch {
            '"' | '\'' => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = ch;
                } else if ch == quote_char {
                    in_quotes = false;
                }
                current.push(ch);
            },
            ',' if !in_quotes => {
                parts.push(current.trim().to_string());
                current.clear();
            },
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        parts.push(current.trim().to_string());
    }
    
    // Ensure we have at least the minimum required fields
    if parts.len() < 11 {
        return Err(anyhow::anyhow!("Not enough values in x_world record"));
    }
    
    // Parse the values according to the x_world format
    let worldid = parts[0].parse::<i32>().ok();
    let x = parts[1].parse::<i32>().unwrap_or(0);
    let y = parts[2].parse::<i32>().unwrap_or(0);
    let tid = parts[3].parse::<i32>().ok();
    let vid = parts[4].parse::<i32>().ok();
    
    // Clean village name (remove quotes)
    let village = parts[5].trim_matches('\'').trim_matches('"').to_string();
    
    let uid = parts[6].parse::<i32>().ok();
    
    // Clean player name (remove quotes)
    let player = if parts[7] == "NULL" || parts[7].is_empty() {
        None
    } else {
        Some(parts[7].trim_matches('\'').trim_matches('"').to_string())
    };
    
    let aid = parts[8].parse::<i32>().ok();
    
    // Clean alliance name (remove quotes)
    let alliance = if parts.len() > 9 && parts[9] != "NULL" && !parts[9].is_empty() {
        Some(parts[9].trim_matches('\'').trim_matches('"').to_string())
    } else {
        None
    };
    
    // Parse population (usually around index 10, but can vary)
    let population = parts[10].parse::<i32>().unwrap_or(0);
    
    Ok(ParsedVillage {
        worldid,
        x,
        y,
        tid,
        vid,
        village,
        uid,
        player,
        aid,
        alliance,
        population,
    })
}

async fn insert_parsed_village_to_table_with_server(pool: &PgPool, village: ParsedVillage, table_name: &str, server_id: i32) -> Result<()> {
    let query = format!(
        r#"
        INSERT INTO {} (server_id, worldid, x, y, tid, vid, village, uid, player, aid, alliance, population)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        table_name
    );
    
    sqlx::query(&query)
        .bind(server_id)
        .bind(village.worldid)
        .bind(village.x)
        .bind(village.y)
        .bind(village.tid)
        .bind(village.vid)
        .bind(village.village)
        .bind(village.uid)
        .bind(village.player)
        .bind(village.aid)
        .bind(village.alliance)
        .bind(village.population)
        .execute(pool)
        .await?;
    
    Ok(())
}

// Server management functions
pub async fn get_all_servers(pool: &PgPool) -> Result<Vec<Server>> {
    let rows = sqlx::query("SELECT id, name, url, is_active FROM servers ORDER BY name")
        .fetch_all(pool)
        .await?;

    let servers: Vec<Server> = rows
        .into_iter()
        .map(|row| Server {
            id: row.get("id"),
            name: row.get("name"),
            url: row.get("url"),
            is_active: row.get("is_active"),
        })
        .collect();

    Ok(servers)
}

pub async fn add_server(pool: &PgPool, name: &str, url: &str) -> Result<Server> {
    let row = sqlx::query(
        "INSERT INTO servers (name, url, is_active) VALUES ($1, $2, $3) RETURNING id, name, url, is_active"
    )
    .bind(name)
    .bind(url)
    .bind(false) // New servers are not active by default
    .fetch_one(pool)
    .await?;

    let server = Server {
        id: row.get("id"),
        name: row.get("name"),
        url: row.get("url"),
        is_active: row.get("is_active"),
    };

    // If this is the first server, make it active and auto-load data
    let all_servers = get_all_servers(pool).await?;
    if all_servers.len() == 1 {
        set_active_server(pool, server.id).await?;
        
        // Auto-load data for the new active server
        match auto_load_data_for_server(pool, &server).await {
            Ok(load_message) => {
                println!("Auto-loaded data for new server '{}': {}", server.name, load_message);
            },
            Err(e) => {
                println!("Failed to auto-load data for new server '{}': {}", server.name, e);
            }
        }
    }

    Ok(server)
}

pub async fn set_active_server(pool: &PgPool, server_id: i32) -> Result<()> {
    // First, set all servers to inactive
    sqlx::query("UPDATE servers SET is_active = FALSE")
        .execute(pool)
        .await?;
    
    // Then set the specified server as active
    sqlx::query("UPDATE servers SET is_active = TRUE WHERE id = $1")
        .bind(server_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn set_active_server_with_auto_load(pool: &PgPool, server_id: i32) -> Result<String> {
    // First activate the server
    set_active_server(pool, server_id).await?;
    
    // Get the server details for auto-loading
    let servers = get_all_servers(pool).await?;
    if let Some(server) = servers.into_iter().find(|s| s.id == server_id) {
        // Auto-load data if needed
        match auto_load_data_for_server(pool, &server).await {
            Ok(load_message) => Ok(load_message),
            Err(e) => Ok(format!("Server activated but failed to auto-load data: {}", e))
        }
    } else {
        Ok("Server activated successfully".to_string())
    }
}

pub async fn remove_server(pool: &PgPool, server_id: i32) -> Result<()> {
    // First, check if this server is currently active
    let active_server = get_active_server(pool).await?;
    let is_removing_active = active_server.map_or(false, |server| server.id == server_id);
    
    // Get all available dates for this server to clean up data tables
    let available_dates = get_available_dates_for_server(pool, server_id).await?;
    
    // Drop all data tables for this server
    for (date, _) in available_dates {
        let table_name = get_table_name_for_server_and_date(server_id, date);
        let drop_query = format!("DROP TABLE IF EXISTS {}", table_name);
        sqlx::query(&drop_query).execute(pool).await?;
        println!("Dropped table: {}", table_name);
    }
    
    // Remove the server from the servers table
    sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(server_id)
        .execute(pool)
        .await?;
    
    // If we removed the active server, set another server as active (if any exist)
    if is_removing_active {
        let remaining_servers = get_all_servers(pool).await?;
        if let Some(first_server) = remaining_servers.first() {
            set_active_server(pool, first_server.id).await?;
            println!("Set server '{}' as active after removing the active server", first_server.name);
        }
    }
    
    Ok(())
}

pub async fn get_latest_data_date_for_server(pool: &PgPool, server_id: i32) -> Result<Option<chrono::NaiveDate>> {
    let available_dates = get_available_dates_for_server(pool, server_id).await?;
    
    if available_dates.is_empty() {
        Ok(None)
    } else {
        Ok(Some(available_dates[0].0)) // Dates are sorted DESC, so first is latest
    }
}

pub async fn is_new_data_needed_for_server(pool: &PgPool, server_id: i32) -> Result<bool> {
    let today = chrono::Utc::now().date_naive();
    
    match get_latest_data_date_for_server(pool, server_id).await? {
        Some(latest_date) => Ok(latest_date < today),
        None => Ok(true), // No data exists, so we need to load it
    }
}

pub async fn auto_load_data_for_server(pool: &PgPool, server: &Server) -> Result<String> {
    // Check if new data is needed
    if !is_new_data_needed_for_server(pool, server.id).await? {
        return Ok("Data is up to date".to_string());
    }

    // Construct the SQL URL based on the server URL
    let sql_url = if server.url.ends_with("/map.sql") || server.url.ends_with("map.sql") {
        server.url.clone()
    } else {
        format!("{}/map.sql", server.url.trim_end_matches('/'))
    };
    
    println!("Auto-loading data for server '{}' from: {}", server.name, sql_url);

    // Fetch the SQL file from the URL
    let client = reqwest::Client::new();
    let response = client.get(&sql_url).send().await
        .map_err(|e| anyhow::anyhow!("Failed to fetch SQL from {}: {}", sql_url, e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP error {}: Failed to fetch SQL from {}", response.status(), sql_url));
    }

    let sql_content = response.text().await
        .map_err(|e| anyhow::anyhow!("Failed to read SQL response: {}", e))?;

    // Execute the SQL for this specific server
    let count = execute_sql_for_server(pool, &sql_content, server.id).await?;
    
    Ok(format!("Successfully loaded {} villages for server '{}'", count, server.name))
}

pub async fn get_active_server(pool: &PgPool) -> Result<Option<Server>> {
    let row = sqlx::query("SELECT id, name, url, is_active FROM servers WHERE is_active = TRUE LIMIT 1")
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(Server {
            id: row.get("id"),
            name: row.get("name"),
            url: row.get("url"),
            is_active: row.get("is_active"),
        }))
    } else {
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TribeStats {
    pub tribe_id: i32,
    pub tribe_name: String,
    pub village_count: i32,
    pub total_population: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerStats {
    pub player_name: String,
    pub village_count: i32,
    pub total_population: i64,
    pub alliance: Option<String>,
    pub profile_link: Option<String>,
    pub alliance_link: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorldInfo {
    pub tribe_stats: Vec<TribeStats>,
    pub top_players: Vec<PlayerStats>,
    pub total_villages: i32,
    pub total_population: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AfkVillage {
    pub village_name: String,
    pub x: i32,
    pub y: i32,
    pub population: i32,
    pub player_name: String,
    pub alliance: Option<String>,
    pub days_without_growth: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AllianceStats {
    pub alliance_name: String,
    pub alliance_id: Option<i32>,
    pub member_count: i32,
    pub village_count: i32,
    pub total_population: i64,
    pub average_population_per_village: i32,
    pub population_growth: i64,
    pub growth_percentage: f64,
    pub alliance_link: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AllianceInfo {
    pub top_alliances: Vec<AllianceStats>,
    pub total_alliances: i32,
}

#[derive(Serialize, Deserialize)]
pub struct AfkSearchParams {
    pub quadrant: String, // "NE", "SE", "SW", "NW"
    pub days: i32, // 1-10
}

fn get_tribe_name(tribe_id: i32) -> String {
    match tribe_id {
        1 => "Romans".to_string(),
        2 => "Teutons".to_string(),
        3 => "Gauls".to_string(),
        4 => "Nature".to_string(),
        5 => "Natars".to_string(),
        6 => "Egyptians".to_string(),
        7 => "Huns".to_string(),
        _ => format!("Tribe {}", tribe_id),
    }
}

pub async fn get_world_info(pool: &PgPool) -> Result<WorldInfo> {
    // Get the active server
    let active_server = get_active_server(pool).await?;
    
    if let Some(server) = active_server {
        get_world_info_for_server(pool, server.id).await
    } else {
        Err(anyhow::anyhow!("No active server found"))
    }
}

pub async fn get_world_info_for_server(pool: &PgPool, server_id: i32) -> Result<WorldInfo> {
    // Get the latest table for this server
    let available_dates = get_available_dates_for_server(pool, server_id).await?;
    
    if available_dates.is_empty() {
        return Ok(WorldInfo {
            tribe_stats: Vec::new(),
            top_players: Vec::new(),
            total_villages: 0,
            total_population: 0,
        });
    }
    
    let latest_date = available_dates[0].0;
    let table_name = get_table_name_for_server_and_date(server_id, latest_date);
    
    // Check if table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
    )
    .bind(&table_name)
    .fetch_one(pool)
    .await?;
    
    if !table_exists {
        return Ok(WorldInfo {
            tribe_stats: Vec::new(),
            top_players: Vec::new(),
            total_villages: 0,
            total_population: 0,
        });
    }
    
    // Get the active server for profile links
    let active_server = get_active_server(pool).await?;
    let server_base_url = if let Some(server) = &active_server {
        // Remove /map.sql from the end if present and prepare base URL for profile links
        let base_url = server.url.trim_end_matches("/map.sql").trim_end_matches("map.sql");
        Some(base_url.trim_end_matches('/').to_string())
    } else {
        None
    };
    
    // Get tribe statistics
    let tribe_query = format!(
        "SELECT tid, COUNT(*) as village_count, SUM(population) as total_population 
         FROM {} 
         WHERE server_id = $1 AND tid IS NOT NULL 
         GROUP BY tid 
         ORDER BY total_population DESC",
        table_name
    );
    
    let tribe_rows = sqlx::query(&tribe_query)
        .bind(server_id)
        .fetch_all(pool)
        .await?;
    
    let tribe_stats: Vec<TribeStats> = tribe_rows
        .into_iter()
        .map(|row| {
            let tribe_id: i32 = row.get("tid");
            TribeStats {
                tribe_id,
                tribe_name: get_tribe_name(tribe_id),
                village_count: row.get::<i64, _>("village_count") as i32,
                total_population: row.get::<Option<i64>, _>("total_population").unwrap_or(0),
            }
        })
        .collect();
    
    // Get top 10 players by population (excluding Natars)
    let player_query = format!(
        "SELECT player, alliance, uid, aid, COUNT(*) as village_count, SUM(population) as total_population 
         FROM {} 
         WHERE server_id = $1 AND player IS NOT NULL AND player != '' AND player != 'Natars'
         GROUP BY player, alliance, uid, aid 
         ORDER BY total_population DESC 
         LIMIT 10",
        table_name
    );
    
    let player_rows = sqlx::query(&player_query)
        .bind(server_id)
        .fetch_all(pool)
        .await?;
    
    let top_players: Vec<PlayerStats> = player_rows
        .into_iter()
        .map(|row| {
            let uid: Option<i32> = row.get("uid");
            let aid: Option<i32> = row.get("aid");
            let profile_link = if let (Some(base_url), Some(player_uid)) = (&server_base_url, uid) {
                Some(format!("{}/profile/{}", base_url, player_uid))
            } else {
                None
            };
            let alliance_link = if let (Some(base_url), Some(alliance_id)) = (&server_base_url, aid) {
                Some(format!("{}/alliance/{}", base_url, alliance_id))
            } else {
                None
            };
            
            PlayerStats {
                player_name: row.get("player"),
                village_count: row.get::<i64, _>("village_count") as i32,
                total_population: row.get::<Option<i64>, _>("total_population").unwrap_or(0),
                alliance: row.get("alliance"),
                profile_link,
                alliance_link,
            }
        })
        .collect();
    
    // Get total statistics
    let total_query = format!(
        "SELECT COUNT(*) as total_villages, SUM(population) as total_population 
         FROM {} 
         WHERE server_id = $1",
        table_name
    );
    
    let total_row = sqlx::query(&total_query)
        .bind(server_id)
        .fetch_one(pool)
        .await?;
    
    let total_villages = total_row.get::<i64, _>("total_villages") as i32;
    let total_population = total_row.get::<Option<i64>, _>("total_population").unwrap_or(0);
    
    Ok(WorldInfo {
        tribe_stats,
        top_players,
        total_villages,
        total_population,
    })
}

pub async fn find_afk_villages(pool: &PgPool, params: AfkSearchParams) -> Result<Vec<AfkVillage>> {
    // Get the active server
    let active_server = get_active_server(pool).await?;
    
    if let Some(server) = active_server {
        find_afk_villages_for_server(pool, server.id, params).await
    } else {
        Err(anyhow::anyhow!("No active server found"))
    }
}

pub async fn find_afk_villages_for_server(pool: &PgPool, server_id: i32, params: AfkSearchParams) -> Result<Vec<AfkVillage>> {
    let available_dates = get_available_dates_for_server(pool, server_id).await?;
    
    if available_dates.len() < (params.days as usize + 1) {
        return Ok(Vec::new()); // Not enough historical data
    }
    
    let latest_date = available_dates[0].0;
    let comparison_date = available_dates[params.days as usize].0;
    
    let latest_table = get_table_name_for_server_and_date(server_id, latest_date);
    let comparison_table = get_table_name_for_server_and_date(server_id, comparison_date);
    
    // Check if both tables exist
    let latest_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
    )
    .bind(&latest_table)
    .fetch_one(pool)
    .await?;
    
    let comparison_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
    )
    .bind(&comparison_table)
    .fetch_one(pool)
    .await?;
    
    if !latest_exists || !comparison_exists {
        return Ok(Vec::new());
    }
    
    // Determine quadrant coordinates
    let (x_condition, y_condition) = match params.quadrant.as_str() {
        "NE" => ("l.x >= 0", "l.y >= 0"),
        "SE" => ("l.x >= 0", "l.y < 0"),
        "SW" => ("l.x < 0", "l.y < 0"),
        "NW" => ("l.x < 0", "l.y >= 0"),
        _ => return Err(anyhow::anyhow!("Invalid quadrant: {}", params.quadrant)),
    };
    
    // Find villages that haven't grown in population
    let village_query = format!(
        r#"
        SELECT l.village, l.x, l.y, l.population, l.player, l.alliance, l.uid
        FROM {} l
        JOIN {} c ON l.x = c.x AND l.y = c.y AND l.server_id = c.server_id
        WHERE l.server_id = $1 
        AND c.server_id = $1
        AND l.player IS NOT NULL 
        AND l.player != '' 
        AND l.player != 'Natars'
        AND c.player = l.player
        AND l.population <= c.population
        AND {} AND {}
        "#,
        latest_table, comparison_table, x_condition, y_condition
    );
    
    let village_rows = sqlx::query(&village_query)
        .bind(server_id)
        .fetch_all(pool)
        .await?;
    
    let mut afk_villages = Vec::new();
    
    for row in village_rows {
        let player_name: String = row.get("player");
        let _uid: Option<i32> = row.get("uid");
        
        // Check if this player has gained population anywhere else
        let player_growth_query = format!(
            r#"
            SELECT 
                COALESCE(SUM(l.population), 0) as latest_total,
                COALESCE(SUM(c.population), 0) as comparison_total
            FROM {} l
            LEFT JOIN {} c ON l.player = c.player AND l.server_id = c.server_id
            WHERE l.server_id = $1 
            AND l.player = $2
            GROUP BY l.player
            "#,
            latest_table, comparison_table
        );
        
        let growth_row = sqlx::query(&player_growth_query)
            .bind(server_id)
            .bind(&player_name)
            .fetch_optional(pool)
            .await?;
        
        let has_grown = if let Some(growth_row) = growth_row {
            let latest_total: i64 = growth_row.get("latest_total");
            let comparison_total: i64 = growth_row.get("comparison_total");
            latest_total > comparison_total
        } else {
            false
        };
        
        // If player hasn't grown overall, include this village in AFK list
        if !has_grown {
            afk_villages.push(AfkVillage {
                village_name: row.get("village"),
                x: row.get("x"),
                y: row.get("y"),
                population: row.get("population"),
                player_name,
                alliance: row.get("alliance"),
                days_without_growth: params.days,
            });
        }
    }
    
    // Sort by population descending
    afk_villages.sort_by(|a, b| b.population.cmp(&a.population));
    
    Ok(afk_villages)
}

pub async fn get_alliance_info(pool: &PgPool) -> Result<AllianceInfo> {
    // Get the active server
    let active_server = get_active_server(pool).await?;
    
    if let Some(server) = active_server {
        get_alliance_info_for_server(pool, server.id).await
    } else {
        Err(anyhow::anyhow!("No active server found"))
    }
}

pub async fn get_alliance_info_for_server(pool: &PgPool, server_id: i32) -> Result<AllianceInfo> {
    let available_dates = get_available_dates_for_server(pool, server_id).await?;
    
    if available_dates.is_empty() {
        return Ok(AllianceInfo {
            top_alliances: Vec::new(),
            total_alliances: 0,
        });
    }
    
    let latest_date = available_dates[0].0;
    let latest_table = get_table_name_for_server_and_date(server_id, latest_date);
    
    // Check if table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
    )
    .bind(&latest_table)
    .fetch_one(pool)
    .await?;
    
    if !table_exists {
        return Ok(AllianceInfo {
            top_alliances: Vec::new(),
            total_alliances: 0,
        });
    }
    
    // Get the active server for alliance links
    let active_server = get_active_server(pool).await?;
    let server_base_url = if let Some(server) = &active_server {
        let base_url = server.url.trim_end_matches("/map.sql").trim_end_matches("map.sql");
        Some(base_url.trim_end_matches('/').to_string())
    } else {
        None
    };
    
    // Get current alliance statistics
    let alliance_query = format!(
        "SELECT alliance, aid, COUNT(DISTINCT uid) as member_count, COUNT(*) as village_count, SUM(population) as total_population
         FROM {} 
         WHERE server_id = $1 AND alliance IS NOT NULL AND alliance != '' AND alliance != 'Natars'
         GROUP BY alliance, aid 
         ORDER BY total_population DESC 
         LIMIT 20",
        latest_table
    );
    
    let alliance_rows = sqlx::query(&alliance_query)
        .bind(server_id)
        .fetch_all(pool)
        .await?;
    
    let mut alliance_stats = Vec::new();
    
    // Get previous day's data for growth calculation if available
    let has_previous_data = available_dates.len() > 1;
    let previous_table = if has_previous_data {
        Some(get_table_name_for_server_and_date(server_id, available_dates[1].0))
    } else {
        None
    };
    
    for row in alliance_rows {
        let alliance_name: String = row.get("alliance");
        let alliance_id: Option<i32> = row.get("aid");
        let member_count: i64 = row.get("member_count");
        let village_count: i64 = row.get("village_count");
        let current_population: i64 = row.get::<Option<i64>, _>("total_population").unwrap_or(0);
        
        // Calculate growth if previous data is available
        let (population_growth, growth_percentage) = if let Some(ref prev_table) = previous_table {
            // Check if previous table exists
            let prev_table_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1)"
            )
            .bind(prev_table)
            .fetch_one(pool)
            .await?;
            
            if prev_table_exists {
                let prev_query = format!(
                    "SELECT SUM(population) as prev_population
                     FROM {} 
                     WHERE server_id = $1 AND alliance = $2",
                    prev_table
                );
                
                let prev_population: i64 = sqlx::query_scalar(&prev_query)
                    .bind(server_id)
                    .bind(&alliance_name)
                    .fetch_optional(pool)
                    .await?
                    .unwrap_or(0);
                
                let growth = current_population - prev_population;
                let growth_pct = if prev_population > 0 {
                    (growth as f64 / prev_population as f64) * 100.0
                } else {
                    0.0
                };
                
                (growth, growth_pct)
            } else {
                (0, 0.0)
            }
        } else {
            (0, 0.0)
        };
        
        let alliance_link = if let (Some(base_url), Some(aid)) = (&server_base_url, alliance_id) {
            Some(format!("{}/alliance/{}", base_url, aid))
        } else {
            None
        };
        
        let avg_pop_per_village = if village_count > 0 {
            (current_population / village_count) as i32
        } else {
            0
        };
        
        alliance_stats.push(AllianceStats {
            alliance_name,
            alliance_id,
            member_count: member_count as i32,
            village_count: village_count as i32,
            total_population: current_population,
            average_population_per_village: avg_pop_per_village,
            population_growth,
            growth_percentage,
            alliance_link,
        });
    }
    
    // Get total number of alliances
    let total_query = format!(
        "SELECT COUNT(DISTINCT alliance) as total_alliances
         FROM {} 
         WHERE server_id = $1 AND alliance IS NOT NULL AND alliance != '' AND alliance != 'Natars'",
        latest_table
    );
    
    let total_alliances: i64 = sqlx::query_scalar(&total_query)
        .bind(server_id)
        .fetch_one(pool)
        .await?;
    
    Ok(AllianceInfo {
        top_alliances: alliance_stats,
        total_alliances: total_alliances as i32,
    })
}
