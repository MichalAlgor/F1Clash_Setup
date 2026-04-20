use axum::Form;
use axum::Router;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Row, postgres::PgRow};

use crate::AppState;
use crate::auth::AuthStatus;
use crate::data::StatPriorities;
use crate::error::AppError;
use crate::get_session_season;
use crate::models::driver::{DriverBoost, DriverInventoryItem};
use crate::models::part::Stats;
use crate::models::setup::{Boost, InventoryItem};
use crate::session::UserSession;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/optimizer/share", post(create_share))
        .route("/share/{hash}", get(view_share))
}

// ── Share form (extends SaveForm with priorities) ─────────────────────────────

#[derive(Deserialize)]
pub struct ShareForm {
    pub name: String,
    #[serde(default)]
    pub brakes_id: Option<i32>,
    #[serde(default)]
    pub gearbox_id: Option<i32>,
    #[serde(default)]
    pub rear_wing_id: Option<i32>,
    #[serde(default)]
    pub front_wing_id: Option<i32>,
    #[serde(default)]
    pub suspension_id: Option<i32>,
    #[serde(default)]
    pub engine_id: Option<i32>,
    #[serde(default)]
    pub battery_id: Option<i32>,
    #[serde(default)]
    pub driver1_id: Option<i32>,
    #[serde(default)]
    pub driver2_id: Option<i32>,
    // Priority flags
    #[serde(default)]
    pub speed: bool,
    #[serde(default)]
    pub cornering: bool,
    #[serde(default)]
    pub power_unit: bool,
    #[serde(default)]
    pub qualifying: bool,
}

// ── Snapshot types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct PartSnapshot {
    pub category: String,
    pub part_name: String,
    pub level: i32,
    pub rarity: String,
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
    pub total: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverSnapshot {
    pub driver_name: String,
    pub rarity: String,
    pub level: i32,
    pub overtaking: i32,
    pub defending: i32,
    pub qualifying: i32,
    pub race_start: i32,
    pub tyre_management: i32,
    pub total: i32,
}

// ── Create share ──────────────────────────────────────────────────────────────

async fn create_share(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
    Form(form): Form<ShareForm>,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let drivers_catalog = state.drivers_catalog_for_season(&season).await;

    // Collect part IDs from form
    let part_ids: Vec<i32> = [
        form.engine_id,
        form.front_wing_id,
        form.rear_wing_id,
        form.suspension_id,
        form.brakes_id,
        form.gearbox_id,
        form.battery_id,
    ]
    .into_iter()
    .flatten()
    .collect();

    // Load inventory items
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE id = ANY($1) AND session_id = $2",
    )
    .bind(&part_ids[..])
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE session_id = $1")
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    // Build parts snapshot
    let mut parts_snapshot: Vec<PartSnapshot> = Vec::new();
    let mut total_parts = Stats::default();

    for item in &items {
        if let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name) {
            if let Some(ls) = part_def.stats_for_level(item.level) {
                let mut s = Stats {
                    speed: ls.speed,
                    cornering: ls.cornering,
                    power_unit: ls.power_unit,
                    qualifying: ls.qualifying,
                    pit_stop_time: ls.pit_stop_time,
                    additional_stat_value: ls.additional_stat_value,
                };
                if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                    s = s.boosted(b.percentage);
                }
                total_parts = total_parts.add(&s);
                parts_snapshot.push(PartSnapshot {
                    category: part_def.category.display_name().to_string(),
                    part_name: item.part_name.clone(),
                    level: item.level,
                    rarity: part_def.rarity.clone(),
                    speed: s.speed,
                    cornering: s.cornering,
                    power_unit: s.power_unit,
                    qualifying: s.qualifying,
                    pit_stop_time: s.pit_stop_time,
                    total: s.total_performance(),
                });
            }
        }
    }

    // Load driver items
    let driver_ids: Vec<i32> = [form.driver1_id, form.driver2_id]
        .into_iter()
        .flatten()
        .collect();

    let driver_items = if driver_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as::<_, DriverInventoryItem>(
            "SELECT * FROM driver_inventory WHERE id = ANY($1) AND session_id = $2",
        )
        .bind(&driver_ids[..])
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
    };

    let driver_boosts =
        sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts WHERE session_id = $1")
            .bind(&session_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let mut drivers_snapshot: Vec<DriverSnapshot> = Vec::new();
    let mut total_ovt = 0;
    let mut total_def = 0;
    let mut total_qual = 0;
    let mut total_rst = 0;
    let mut total_tyr = 0;

    for item in &driver_items {
        if let Some(def) = drivers_catalog
            .iter()
            .find(|d| d.name == item.driver_name && d.rarity == item.rarity)
        {
            if let Some(ls) = def.stats_for_level(item.level) {
                let mut ds = ls.to_stats();
                if let Some(b) = driver_boosts
                    .iter()
                    .find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity)
                {
                    ds = ds.boosted(b.percentage);
                }
                total_ovt += ds.overtaking;
                total_def += ds.defending;
                total_qual += ds.qualifying;
                total_rst += ds.race_start;
                total_tyr += ds.tyre_management;
                drivers_snapshot.push(DriverSnapshot {
                    driver_name: item.driver_name.clone(),
                    rarity: item.rarity.clone(),
                    level: item.level,
                    overtaking: ds.overtaking,
                    defending: ds.defending,
                    qualifying: ds.qualifying,
                    race_start: ds.race_start,
                    tyre_management: ds.tyre_management,
                    total: ds.total(),
                });
            }
        }
    }

    let priorities_val = json!({
        "speed": form.speed,
        "cornering": form.cornering,
        "power_unit": form.power_unit,
        "qualifying": form.qualifying,
    });

    let total_parts_val = json!({
        "speed": total_parts.speed,
        "cornering": total_parts.cornering,
        "power_unit": total_parts.power_unit,
        "qualifying": total_parts.qualifying,
        "pit_stop_time": total_parts.pit_stop_time,
        "total": total_parts.total_performance(),
    });

    let total_drivers_val = json!({
        "overtaking": total_ovt,
        "defending": total_def,
        "qualifying": total_qual,
        "race_start": total_rst,
        "tyre_management": total_tyr,
        "total": total_ovt + total_def + total_qual + total_rst + total_tyr,
    });

    // Generate unique hash
    let share_hash = generate_hash(&state.pool).await;

    sqlx::query(
        "INSERT INTO shared_setups \
         (share_hash, name, season, priorities, parts_snapshot, drivers_snapshot, total_parts, total_drivers) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&share_hash)
    .bind(&form.name)
    .bind(&season)
    .bind(&priorities_val)
    .bind(serde_json::to_value(&parts_snapshot)?)
    .bind(serde_json::to_value(&drivers_snapshot)?)
    .bind(&total_parts_val)
    .bind(&total_drivers_val)
    .execute(&state.pool)
    .await?;

    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "share_create",
        serde_json::json!({ "season": season }),
    );

    Ok(templates::share::shared_page(
        &share_hash,
        &form.name,
        &auth,
    ))
}

// ── View share ────────────────────────────────────────────────────────────────

async fn view_share(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(hash): Path<String>,
    auth: AuthStatus,
) -> impl IntoResponse {
    let row: Option<PgRow> = sqlx::query(
        "SELECT id, share_hash, name, season, priorities, parts_snapshot, drivers_snapshot, \
         total_parts, total_drivers FROM shared_setups WHERE share_hash = $1",
    )
    .bind(&hash)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(row) = row else {
        return templates::share::not_found_page(&auth);
    };

    let priorities_val: sqlx::types::Json<Value> = row.get("priorities");
    let parts_val: sqlx::types::Json<Value> = row.get("parts_snapshot");
    let drivers_val: sqlx::types::Json<Value> = row.get("drivers_snapshot");
    let total_parts_val: sqlx::types::Json<Value> = row.get("total_parts");
    let total_drivers_val: sqlx::types::Json<Value> = row.get("total_drivers");

    let record_name: String = row.get("name");
    let record_season: String = row.get("season");
    let record_hash: String = row.get("share_hash");

    // Parse snapshots
    let parts: Vec<PartSnapshot> = serde_json::from_value(parts_val.0.clone()).unwrap_or_default();
    let drivers: Vec<DriverSnapshot> =
        serde_json::from_value(drivers_val.0.clone()).unwrap_or_default();

    // Check viewer's inventory for comparison
    let season = get_session_season(&state.pool, &session_id).await;
    let viewer_items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let priorities = StatPriorities {
        speed: priorities_val.0["speed"].as_bool().unwrap_or(false),
        cornering: priorities_val.0["cornering"].as_bool().unwrap_or(false),
        power_unit: priorities_val.0["power_unit"].as_bool().unwrap_or(false),
        qualifying: priorities_val.0["qualifying"].as_bool().unwrap_or(false),
    };

    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "share_view",
        serde_json::json!({ "season": record_season }),
    );

    templates::share::view_page(
        &record_hash,
        &record_name,
        &record_season,
        &priorities,
        &parts,
        &drivers,
        &total_parts_val.0,
        &total_drivers_val.0,
        &viewer_items,
        &auth,
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn generate_hash(pool: &sqlx::PgPool) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";
    loop {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as usize;
        let hash: String = (0..8)
            .map(|i| {
                let idx = (seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(i * 1442695040888963407))
                    % CHARSET.len();
                CHARSET[idx] as char
            })
            .collect();

        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM shared_setups WHERE share_hash = $1)")
                .bind(&hash)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

        if !exists {
            return hash;
        }
    }
}
