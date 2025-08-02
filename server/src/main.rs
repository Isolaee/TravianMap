use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;
use tower_http::cors::CorsLayer;
use anyhow::Result;

mod database;

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct MapData {
    id: u32,
    name: String,
    x: i32,
    y: i32,
    population: u32,
    player: Option<String>,
    alliance: Option<String>,
    worldid: Option<u32>,
}

#[derive(Deserialize)]
struct MapQuery {
    x: Option<i32>,
    y: Option<i32>,
    radius: Option<i32>,
}

#[derive(Deserialize)]
struct CreateVillageRequest {
    name: String,
    x: i32,
    y: i32,
    population: u32,
}

#[derive(Deserialize)]
struct UpdatePopulationRequest {
    population: u32,
}

#[derive(Deserialize)]
struct LoadSqlRequest {
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    println!("Starting Travian Map Server...");

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/travian_map".to_string());

    println!("Connecting to database: {}", database_url.replace("password", "***"));

    // Create database connection pool
    let pool = database::create_pool(&database_url).await
        .expect("Failed to create database pool");

    // Create tables and insert sample data
    database::create_tables(&pool).await
        .expect("Failed to create tables");
    
    database::insert_sample_data(&pool).await
        .expect("Failed to insert sample data");

    println!("Database initialized successfully!");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/map", get(get_map_data))
        .route("/api/villages", get(get_villages).post(create_village))
        .route("/api/villages/:id", put(update_village).delete(delete_village))
        .route("/api/load-sql", post(load_sql_from_url))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3001".to_string());
    let bind_address = format!("{}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("Failed to bind to address");
    
    println!("Server running on http://{}", bind_address);
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

async fn root() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "success".to_string(),
        message: "Travian Map Server is running!".to_string(),
    })
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        message: "Server is operational".to_string(),
    })
}

async fn get_map_data(State(pool): State<PgPool>, Query(params): Query<MapQuery>) -> Result<Json<Vec<MapData>>, StatusCode> {
    let radius = params.radius.unwrap_or(10);
    
    let villages = if let (Some(x), Some(y)) = (params.x, params.y) {
        database::get_villages_near(&pool, x, y, radius).await
    } else {
        database::get_all_villages(&pool).await
    };

    match villages {
        Ok(villages) => Ok(Json(villages)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_villages(State(pool): State<PgPool>) -> Result<Json<Vec<MapData>>, StatusCode> {
    match database::get_all_villages(&pool).await {
        Ok(villages) => Ok(Json(villages)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_village(
    State(pool): State<PgPool>,
    Json(request): Json<CreateVillageRequest>,
) -> Result<Json<MapData>, StatusCode> {
    match database::add_village(&pool, &request.name, request.x, request.y, request.population).await {
        Ok(village) => Ok(Json(village)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_village(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
    Json(request): Json<UpdatePopulationRequest>,
) -> Result<Json<MapData>, StatusCode> {
    match database::update_village_population(&pool, id, request.population).await {
        Ok(Some(village)) => Ok(Json(village)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_village(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> StatusCode {
    match database::delete_village(&pool, id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn load_sql_from_url(
    State(pool): State<PgPool>,
    Json(request): Json<LoadSqlRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("Loading SQL from URL: {}", request.url);

    // Fetch the SQL file from the URL
    let client = reqwest::Client::new();
    let response = match client.get(&request.url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to fetch URL: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if !response.status().is_success() {
        eprintln!("HTTP error: {}", response.status());
        return Err(StatusCode::BAD_REQUEST);
    }

    let sql_content = match response.text().await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read response: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Clear existing data
    match database::clear_all_villages(&pool).await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to clear existing data: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Parse and execute the SQL
    match database::execute_sql(&pool, &sql_content).await {
        Ok(count) => {
            println!("Successfully loaded {} villages from SQL", count);
            Ok(Json(serde_json::json!({
                "status": "success",
                "message": format!("Successfully loaded {} villages from SQL file", count)
            })))
        },
        Err(e) => {
            eprintln!("Failed to execute SQL: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
