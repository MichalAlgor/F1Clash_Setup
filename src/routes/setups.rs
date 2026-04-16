use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth::AuthStatus;
use crate::models::driver::{DriverBoost, DriverInventoryItem, DriverStats, OwnedDriverDefinition};
use crate::models::part::{OwnedLevelStats, OwnedPartDefinition, PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem, Setup, SetupWithStats};
use crate::templates;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/setups", get(list).post(create))
        .route("/setups/new", get(new))
        .route("/setups/{id}", get(show).post(update))
        .route("/setups/{id}", delete(destroy))
}

async fn list(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;
    let drivers_catalog = state.drivers_catalog_for_season().await;
    let setups = sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE season = $1 ORDER BY name")
        .bind(&season)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let mut with_stats = Vec::new();
    for setup in setups {
        let (stats, driver_stats) = compute_all_stats(&state.pool, &setup, &catalog, &drivers_catalog).await;
        with_stats.push(SetupWithStats { setup, stats, driver_stats });
    }

    templates::setups::list_page(&with_stats, &auth)
}

async fn new(State(state): State<AppState>, auth: AuthStatus) -> impl IntoResponse {
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;
    let drivers_catalog = state.drivers_catalog_for_season().await;
    let categories = state.categories_for_season().await;
    let inventory_by_category = load_inventory_by_category(&state.pool, &season, &catalog, &categories).await;
    let driver_items = load_driver_inventory(&state.pool, &season).await;
    templates::setups::form_page(&inventory_by_category, &driver_items, &drivers_catalog, None, &auth)
}

#[derive(Deserialize)]
pub struct SetupForm {
    pub name: String,
    #[serde(rename = "engine", default)]
    pub engine_id: Option<i32>,
    #[serde(rename = "front_wing", default)]
    pub front_wing_id: Option<i32>,
    #[serde(rename = "rear_wing", default)]
    pub rear_wing_id: Option<i32>,
    #[serde(rename = "suspension", default)]
    pub suspension_id: Option<i32>,
    #[serde(rename = "brakes", default)]
    pub brakes_id: Option<i32>,
    #[serde(rename = "gearbox", default)]
    pub gearbox_id: Option<i32>,
    #[serde(rename = "battery", default)]
    pub battery_id: Option<i32>,
    pub driver1_id: Option<i32>,
    pub driver2_id: Option<i32>,
}

async fn create(State(state): State<AppState>, Form(form): Form<SetupForm>) -> impl IntoResponse {
    let season = state.season().await;
    sqlx::query(
        "INSERT INTO setups (name, engine_id, front_wing_id, rear_wing_id, suspension_id, brakes_id, gearbox_id, battery_id, driver1_id, driver2_id, season)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .bind(form.gearbox_id)
    .bind(form.battery_id)
    .bind(form.driver1_id)
    .bind(form.driver2_id)
    .bind(&season)
    .execute(&state.pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}

async fn show(State(state): State<AppState>, Path(id): Path<i32>, auth: AuthStatus) -> impl IntoResponse {
    let catalog = state.catalog_for_season().await;
    let drivers_catalog = state.drivers_catalog_for_season().await;
    let setup = sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .unwrap();

    let (stats, driver_stats) = compute_all_stats(&state.pool, &setup, &catalog, &drivers_catalog).await;

    // Find the label for this season's special stat (e.g. "DRS", "Overtake Mode")
    let additional_stat_label = catalog.iter()
        .find_map(|p| p.additional_stat_name.clone());

    let s = SetupWithStats { setup, stats, driver_stats };

    crate::templates::layout::page(
        &s.setup.name,
        &auth,
        html! {
            h1 { (&s.setup.name) }

            h2 { "Part Stats" }
            figure {
                table {
                    thead { tr { th { "Stat" } th { "Value" } } }
                    tbody {
                        tr { td { "Speed" } td { (s.stats.speed) } }
                        tr { td { "Cornering" } td { (s.stats.cornering) } }
                        tr { td { "Power Unit" } td { (s.stats.power_unit) } }
                        tr { td { "Qualifying" } td { (s.stats.qualifying) } }
                        tr { td { "Pit Stop Time" } td { (format!("{:.2}s", s.stats.pit_stop_time)) } }
                        @if s.stats.additional_stat_value > 0 {
                            @let label = additional_stat_label.as_deref().unwrap_or("Special");
                            tr { td { (label) } td { (s.stats.additional_stat_value) } }
                        }
                        tr { td { strong { "Total Performance" } } td { strong { (s.stats.total_performance()) } } }
                    }
                }
            }

            @if s.driver_stats.total() > 0 {
                h2 { "Driver Stats" }
                figure {
                    table {
                        thead { tr { th { "Stat" } th { "Value" } } }
                        tbody {
                            tr { td { "Overtaking" } td { (s.driver_stats.overtaking) } }
                            tr { td { "Defending" } td { (s.driver_stats.defending) } }
                            tr { td { "Qualifying" } td { (s.driver_stats.qualifying) } }
                            tr { td { "Race Start" } td { (s.driver_stats.race_start) } }
                            tr { td { "Tyre Management" } td { (s.driver_stats.tyre_management) } }
                            tr { td { strong { "Total" } } td { strong { (s.driver_stats.total()) } } }
                        }
                    }
                }
            }

            a href="/setups" role="button" class="outline" { "← Back to setups" }
        },
    )
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Form(form): Form<SetupForm>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE setups SET name=$1, engine_id=$2, front_wing_id=$3, rear_wing_id=$4, suspension_id=$5, brakes_id=$6, gearbox_id=$7, battery_id=$8, driver1_id=$9, driver2_id=$10
         WHERE id=$11",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .bind(form.gearbox_id)
    .bind(form.battery_id)
    .bind(form.driver1_id)
    .bind(form.driver2_id)
    .bind(id)
    .execute(&state.pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}

async fn destroy(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM setups WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .unwrap();

    html! {}
}

async fn compute_all_stats(
    pool: &PgPool,
    setup: &Setup,
    catalog: &[OwnedPartDefinition],
    drivers_catalog: &[OwnedDriverDefinition],
) -> (Stats, DriverStats) {
    let part_stats = compute_part_stats(pool, setup, catalog).await;
    let driver_stats = compute_driver_stats(pool, setup, drivers_catalog).await;
    (part_stats, driver_stats)
}

async fn compute_part_stats(
    pool: &PgPool,
    setup: &Setup,
    catalog: &[OwnedPartDefinition],
) -> Stats {
    let mut part_ids: Vec<i32> = vec![
        setup.engine_id, setup.front_wing_id, setup.rear_wing_id,
        setup.suspension_id, setup.brakes_id, setup.gearbox_id,
    ];
    if let Some(id) = setup.battery_id { part_ids.push(id); }

    let items = sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory WHERE id = ANY($1)")
        .bind(&part_ids[..]).fetch_all(pool).await.unwrap_or_default();
    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts")
        .fetch_all(pool).await.unwrap_or_default();

    let mut stats = Stats::default();
    for item in &items {
        if let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name) {
            if let Some(level_stats) = part_def.stats_for_level(item.level) {
                let mut ps = Stats {
                    speed: level_stats.speed, cornering: level_stats.cornering,
                    power_unit: level_stats.power_unit, qualifying: level_stats.qualifying,
                    pit_stop_time: level_stats.pit_stop_time,
                    additional_stat_value: level_stats.additional_stat_value,
                };
                if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                    ps = ps.boosted(b.percentage);
                }
                stats = stats.add(&ps);
            }
        }
    }
    stats
}

async fn compute_driver_stats(
    pool: &PgPool,
    setup: &Setup,
    drivers_catalog: &[OwnedDriverDefinition],
) -> DriverStats {
    let driver_ids: Vec<i32> = [setup.driver1_id, setup.driver2_id]
        .iter().filter_map(|id| *id).collect();
    if driver_ids.is_empty() { return DriverStats::default(); }

    let items = sqlx::query_as::<_, DriverInventoryItem>("SELECT * FROM driver_inventory WHERE id = ANY($1)")
        .bind(&driver_ids[..]).fetch_all(pool).await.unwrap_or_default();
    let boosts = sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts")
        .fetch_all(pool).await.unwrap_or_default();

    let mut stats = DriverStats::default();
    for item in &items {
        if let Some(def) = drivers_catalog.iter().find(|d| d.name == item.driver_name && d.rarity == item.rarity) {
            if let Some(ls) = def.stats_for_level(item.level) {
                let mut ds = ls.to_stats();
                if let Some(b) = boosts.iter().find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity) {
                    ds = ds.boosted(b.percentage);
                }
                stats = stats.add(&ds);
            }
        }
    }
    stats
}

pub async fn load_inventory_by_category(
    pool: &PgPool,
    season: &str,
    catalog: &[OwnedPartDefinition],
    categories: &[PartCategory],
) -> Vec<(PartCategory, Vec<(InventoryItem, OwnedLevelStats)>)> {
    let items = sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory WHERE season = $1 ORDER BY part_name")
        .bind(season).fetch_all(pool).await.unwrap_or_default();

    categories.iter().map(|cat| {
        let cat_items: Vec<_> = items.iter().filter_map(|item| {
            let part_def = catalog.iter().find(|p| p.name == item.part_name)?;
            if part_def.category != *cat { return None; }
            let level_stats = part_def.stats_for_level(item.level)?.clone();
            Some((item.clone(), level_stats))
        }).collect();
        (*cat, cat_items)
    }).collect()
}

pub async fn load_driver_inventory(pool: &PgPool, season: &str) -> Vec<DriverInventoryItem> {
    sqlx::query_as::<_, DriverInventoryItem>("SELECT * FROM driver_inventory WHERE season = $1 ORDER BY driver_name")
        .bind(season).fetch_all(pool).await.unwrap_or_default()
}
