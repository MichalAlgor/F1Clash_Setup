pub mod data;
pub mod drivers_data;
mod models;
mod routes;
mod templates;

use std::sync::Arc;
use axum::Router;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub active_season: Arc<RwLock<String>>,
}

impl AppState {
    pub async fn season(&self) -> String {
        self.active_season.read().await.clone()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

    // Load active season from settings
    let season: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'active_season'")
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| "2025".to_string());

    let state = AppState {
        pool,
        active_season: Arc::new(RwLock::new(season)),
    };

    let app = Router::new()
        .merge(routes::inventory::router())
        .merge(routes::setups::router())
        .merge(routes::boosts::router())
        .merge(routes::drivers::router())
        .merge(routes::optimizer::router())
        .merge(routes::season::router())
        .merge(routes::export_import::router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
