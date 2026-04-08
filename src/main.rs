pub mod data;
pub mod drivers_data;
mod models;
mod routes;
mod templates;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;

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

    let app = Router::new()
        .merge(routes::inventory::router())
        .merge(routes::setups::router())
        .merge(routes::boosts::router())
        .merge(routes::drivers::router())
        .merge(routes::optimizer::router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
