use sqlx::{PgPool, Row};
use anyhow::Result;
use crate::MapData;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

fn get_table_name_for_date(date: chrono::NaiveDate) -> String {
    format!("villages_{}", date.format("%Y_%m_%d"))
}

fn get_today_table_name() -> String {
    let today = chrono::Utc::now().date_naive();
    get_table_name_for_date(today)
}

pub async fn create_table_for_date(pool: &PgPool, date: chrono::NaiveDate) -> Result<String> {
    let table_name = get_table_name_for_date(date);
    
    // Create the villages table with Travian x_world structure for the specific date
    let create_query = format!(
        r#"
        CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
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
    let coord_index = format!("CREATE INDEX IF NOT EXISTS idx_{}_coordinates ON {} (x, y)", table_name, table_name);
    sqlx::query(&coord_index).execute(pool).await?;

    let pop_index = format!("CREATE INDEX IF NOT EXISTS idx_{}_population ON {} (population)", table_name, table_name);
    sqlx::query(&pop_index).execute(pool).await?;

    let world_index = format!("CREATE INDEX IF NOT EXISTS idx_{}_worldid ON {} (worldid)", table_name, table_name);
    sqlx::query(&world_index).execute(pool).await?;

    Ok(table_name)
}

pub async fn create_tables(pool: &PgPool) -> Result<()> {
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
    // Check if we already have data
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM villages")
        .fetch_one(pool)
        .await?;

    if count > 0 {
        return Ok(()); // Data already exists
    }

    // Insert sample villages
    let villages = vec![
        ("Capital Village", 0, 0, 1000),
        ("North Settlement", 5, 10, 500),
        ("East Outpost", 15, 3, 300),
        ("South Trading Post", -8, -12, 750),
        ("Western Fort", -20, 5, 600),
        ("Mountain Village", 25, 25, 800),
        ("River Crossing", -15, 8, 450),
        ("Forest Camp", 12, -5, 350),
        ("Desert Oasis", 30, -20, 200),
        ("Coastal Town", -25, -15, 900),
        ("Hill Fort", 8, 18, 650),
        ("Valley Farm", -5, -8, 400),
    ];

    for (name, x, y, population) in villages {
        sqlx::query(
            "INSERT INTO villages (village, x, y, population, player, alliance) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(name)
        .bind(x)
        .bind(y)
        .bind(population)
        .bind("Sample Player")
        .bind("Sample Alliance")
        .execute(pool)
        .await?;
    }

    println!("Inserted {} sample villages into database", 12);
    Ok(())
}

pub async fn get_all_villages(pool: &PgPool) -> Result<Vec<MapData>> {
    // Get the latest table (most recent date)
    let available_dates = get_available_dates(pool).await?;
    
    if available_dates.is_empty() {
        return Ok(Vec::new()); // No tables available
    }
    
    let latest_date = available_dates[0].0;
    get_villages_by_date(pool, latest_date).await
}

pub async fn get_villages_by_date(pool: &PgPool, date: chrono::NaiveDate) -> Result<Vec<MapData>> {
    let table_name = get_table_name_for_date(date);
    
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
        "SELECT id, village, x, y, population, player, alliance, worldid FROM {} ORDER BY population DESC",
        table_name
    );
    
    let rows = sqlx::query(&query)
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

pub async fn get_villages_near(pool: &PgPool, x: i32, y: i32, radius: i32) -> Result<Vec<MapData>> {
    // Get the latest table (most recent date)
    let available_dates = get_available_dates(pool).await?;
    
    if available_dates.is_empty() {
        return Ok(Vec::new()); // No tables available
    }
    
    let latest_date = available_dates[0].0;
    let table_name = get_table_name_for_date(latest_date);
    
    let query = format!(
        r#"
        SELECT id, village, x, y, population, player, alliance, worldid
        FROM {} 
        WHERE ABS(x - $1) <= $3 AND ABS(y - $2) <= $3
        ORDER BY (ABS(x - $1) + ABS(y - $2)), population DESC
        "#,
        table_name
    );
    
    let rows = sqlx::query(&query)
        .bind(x)
        .bind(y)
        .bind(radius)
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
    let today = chrono::Utc::now().date_naive();
    
    // Create table for today if it doesn't exist
    let table_name = create_table_for_date(pool, today).await?;
    
    // Clear existing data for today
    let delete_query = format!("DELETE FROM {}", table_name);
    sqlx::query(&delete_query).execute(pool).await?;
    
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
                            match insert_parsed_village_to_table(pool, parsed_village, &table_name).await {
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

async fn insert_parsed_village_to_table(pool: &PgPool, village: ParsedVillage, table_name: &str) -> Result<()> {
    let query = format!(
        r#"
        INSERT INTO {} (worldid, x, y, tid, vid, village, uid, player, aid, alliance, population)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        table_name
    );
    
    sqlx::query(&query)
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
