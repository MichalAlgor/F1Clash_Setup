mod analytics;
pub mod auth;
pub mod catalog;
pub mod data;
pub mod drivers_data;
pub mod error;
mod models;
pub mod optimizer_core;
mod routes;
pub mod session;
mod templates;
pub mod upgrade_advisor;

use axum::Router;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::models::driver::OwnedDriverDefinition;
use crate::models::part::{OwnedPartDefinition, PartCategory};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub catalog: Arc<RwLock<Vec<OwnedPartDefinition>>>,
    /// Which part categories are active per season.
    pub season_categories: Arc<RwLock<HashMap<String, Vec<PartCategory>>>>,
    /// Driver definitions for all seasons.
    pub drivers_catalog: Arc<RwLock<Vec<OwnedDriverDefinition>>>,
    /// Plain-text password for verification on login (None = no auth).
    pub admin_password: Option<String>,
    /// Opaque token stored in the session cookie (derived from password).
    pub session_token: Option<String>,
    /// Analytics handle for query API and dashboard.
    pub analytics: analytics::AnalyticsHandle,
}

impl AppState {
    /// All parts for the given season, in sort_order.
    pub async fn catalog_for_season(&self, season: &str) -> Vec<OwnedPartDefinition> {
        self.catalog
            .read()
            .await
            .iter()
            .filter(|p| p.season == season)
            .cloned()
            .collect()
    }

    /// Find a part by name within the given season.
    pub async fn find_part(&self, name: &str, season: &str) -> Option<OwnedPartDefinition> {
        self.catalog
            .read()
            .await
            .iter()
            .find(|p| p.name == name && p.season == season)
            .cloned()
    }

    /// Parts for the given season filtered by category.
    pub async fn parts_by_category(
        &self,
        category: PartCategory,
        season: &str,
    ) -> Vec<OwnedPartDefinition> {
        self.catalog
            .read()
            .await
            .iter()
            .filter(|p| p.season == season && p.category == category)
            .cloned()
            .collect()
    }

    /// The ordered list of part categories active in the given season.
    pub async fn categories_for_season(&self, season: &str) -> Vec<PartCategory> {
        self.season_categories
            .read()
            .await
            .get(season)
            .cloned()
            .unwrap_or_default()
    }

    /// All driver definitions for the given season, in sort_order.
    pub async fn drivers_catalog_for_season(&self, season: &str) -> Vec<OwnedDriverDefinition> {
        self.drivers_catalog
            .read()
            .await
            .iter()
            .filter(|d| d.season == season)
            .cloned()
            .collect()
    }

    /// Find a driver definition by name+rarity within the given season.
    pub async fn find_driver_def(
        &self,
        name: &str,
        rarity: &str,
        season: &str,
    ) -> Option<OwnedDriverDefinition> {
        self.drivers_catalog
            .read()
            .await
            .iter()
            .find(|d| d.name == name && d.rarity == rarity && d.season == season)
            .cloned()
    }
}

/// Read the active season for a session from the settings table.
/// Inserts a default row on first access so new sessions are bootstrapped automatically.
pub async fn get_session_season(pool: &PgPool, session_id: &str) -> String {
    sqlx::query(
        "INSERT INTO settings (key, value, session_id)
         VALUES ('active_season', '2025', $1)
         ON CONFLICT (key, session_id) DO NOTHING",
    )
    .bind(session_id)
    .execute(pool)
    .await
    .ok();

    sqlx::query_scalar("SELECT value FROM settings WHERE key = 'active_season' AND session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| "2025".to_string())
}

/// Minimal base64 encoder — used to derive the session token from ADMIN_PASSWORD.
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i] as u32;
        let b1 = if i + 1 < input.len() {
            input[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < input.len() {
            input[i + 2] as u32
        } else {
            0
        };
        out.push(CHARS[((b0 >> 2) & 0x3F) as usize] as char);
        out.push(CHARS[(((b0 << 4) | (b1 >> 4)) & 0x3F) as usize] as char);
        out.push(if i + 1 < input.len() {
            CHARS[(((b1 << 2) | (b2 >> 6)) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if i + 2 < input.len() {
            CHARS[(b2 & 0x3F) as usize] as char
        } else {
            '='
        });
        i += 3;
    }
    out
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

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Seed catalog from parts.json (upsert — never deletes)
    catalog::seed_catalog(&pool).await;

    // Seed driver catalog from drivers.json (falls back to static data when absent)
    catalog::seed_drivers_catalog(&pool).await;

    // Load the full catalog (all seasons) into memory
    let parts = catalog::load_catalog(&pool).await;

    // Load season → category mappings
    let season_cats = catalog::load_season_categories(&pool).await;

    // Load driver catalog (all seasons) into memory
    let drivers = catalog::load_drivers_catalog(&pool).await;

    // Auth setup
    let admin_password = std::env::var("ADMIN_PASSWORD").ok();
    let session_token = admin_password
        .as_deref()
        .map(|p| base64_encode(format!("f1clash-admin:{p}").as_bytes()));

    if admin_password.is_none() {
        tracing::warn!("ADMIN_PASSWORD not set — admin routes are unprotected");
    }

    // GeoIP provider selection
    let geoip: Arc<dyn analytics::geoip::GeoIpProvider> =
        match std::env::var("GEOIP_PROVIDER").ok().as_deref() {
            #[cfg(feature = "maxmind")]
            Some("maxmind") => {
                let path = std::env::var("GEOIP_DB_PATH").expect("GEOIP_DB_PATH required");
                Arc::new(analytics::geoip::maxmind::MaxMindGeoIp::open(path).unwrap())
            }
            #[cfg(feature = "http-geoip")]
            Some("http") => Arc::new(analytics::geoip::http::HttpGeoIp::new()),
            _ => Arc::new(analytics::geoip::NoopGeoIp),
        };

    // Analytics setup
    let analytics_handle: analytics::AnalyticsHandle =
        Arc::new(analytics::postgres::PostgresAnalytics::new(pool.clone()));
    let analytics_state = analytics::middleware::AnalyticsState {
        sink: analytics_handle.clone(),
        geoip,
    };

    // Background pruning — delete events older than 90 days, once per day
    let pruner = analytics::postgres::PostgresAnalytics::new(pool.clone());
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
        loop {
            interval.tick().await;
            match pruner.prune(90).await {
                Ok(n) if n > 0 => tracing::info!("pruned {} old page_events", n),
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "prune failed"),
            }
        }
    });

    let state = AppState {
        pool,
        catalog: Arc::new(RwLock::new(parts)),
        season_categories: Arc::new(RwLock::new(season_cats)),
        drivers_catalog: Arc::new(RwLock::new(drivers)),
        admin_password,
        session_token,
        analytics: analytics_handle.clone(),
    };

    let app = Router::new()
        .merge(routes::inventory::router())
        .merge(routes::setups::router())
        .merge(routes::boosts::router())
        .merge(routes::drivers::router())
        .merge(routes::optimizer::router())
        .merge(routes::advisor::router())
        .merge(routes::share::router())
        .merge(routes::season::router())
        .merge(routes::export_import::router())
        .merge(routes::admin::router())
        .merge(analytics::admin::router())
        .merge(routes::auth_routes::router())
        .nest_service("/static", ServeDir::new("static"))
        .fallback(|| async {
            templates::error::error_page(axum::http::StatusCode::NOT_FOUND, "Page not found.")
        })
        .layer(axum::middleware::from_fn_with_state(
            analytics_state,
            analytics::middleware::record_analytics,
        ))
        .layer(axum::middleware::from_fn(session::session_middleware))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
