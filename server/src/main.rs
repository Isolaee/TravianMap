use axum::{
    extract::{Path, State},
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
        .route("/api/villages", get(get_villages).post(create_village))
        .route("/api/villages/:id", put(update_village).delete(delete_village))
        .route("/api/servers", get(get_servers).post(add_server_api))
        .route("/api/servers/:id/activate", put(activate_server_api))
        .route("/api/servers/:id", delete(remove_server_api))
        .route("/api/world-info", get(get_world_info))
        .route("/api/afk-villages", post(find_afk_villages_api))
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

#[derive(Deserialize)]
struct AddServerRequest {
    name: String,
    url: String,
}

async fn get_servers(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match database::get_all_servers(&pool).await {
        Ok(servers) => Ok(Json(serde_json::json!({
            "status": "success",
            "servers": servers
        }))),
        Err(e) => {
            eprintln!("Failed to get servers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn add_server_api(
    State(pool): State<PgPool>,
    Json(request): Json<AddServerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if request.name.trim().is_empty() || request.url.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match database::add_server(&pool, &request.name.trim(), &request.url.trim()).await {
        Ok(server) => Ok(Json(serde_json::json!({
            "status": "success",
            "server": server
        }))),
        Err(e) => {
            eprintln!("Failed to add server: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn activate_server_api(
    State(pool): State<PgPool>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Activate the server and auto-load data
    match database::set_active_server_with_auto_load(&pool, server_id).await {
        Ok(load_message) => {
            println!("Server activation result: {}", load_message);
            Ok(Json(serde_json::json!({
                "status": "success",
                "message": "Server activated successfully",
                "auto_load_message": load_message
            })))
        },
        Err(e) => {
            eprintln!("Failed to activate server: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn remove_server_api(
    State(pool): State<PgPool>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match database::remove_server(&pool, server_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Server removed successfully"
        }))),
        Err(e) => {
            eprintln!("Failed to remove server: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_world_info(State(pool): State<PgPool>) -> Result<Json<serde_json::Value>, StatusCode> {
    match database::get_world_info(&pool).await {
        Ok(world_info) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": world_info
        }))),
        Err(e) => {
            eprintln!("Failed to get world info: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn find_afk_villages_api(
    State(pool): State<PgPool>,
    Json(params): Json<database::AfkSearchParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate parameters
    if !["NE", "SE", "SW", "NW"].contains(&params.quadrant.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if params.days < 1 || params.days > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match database::find_afk_villages(&pool, params).await {
        Ok(afk_villages) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": afk_villages
        }))),
        Err(e) => {
            eprintln!("Failed to find AFK villages: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
