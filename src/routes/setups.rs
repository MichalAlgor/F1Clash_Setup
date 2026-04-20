use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::AppState;
use crate::auth::AuthStatus;
use crate::error::AppError;
use crate::get_session_season;
use crate::models::driver::{DriverBoost, DriverInventoryItem, DriverStats, OwnedDriverDefinition};
use crate::models::part::{OwnedLevelStats, OwnedPartDefinition, PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem, Setup, SetupWithStats};
use crate::session::UserSession;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/setups", get(list).post(create))
        .route("/setups/new", get(new))
        .route("/setups/compare", get(compare))
        .route("/setups/{id}/edit", get(edit))
        .route("/setups/{id}", get(show).post(update))
        .route("/setups/{id}", delete(destroy))
}

async fn list(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let drivers_catalog = state.drivers_catalog_for_season(&season).await;
    let setups = sqlx::query_as::<_, Setup>(
        "SELECT * FROM setups WHERE season = $1 AND session_id = $2 ORDER BY name",
    )
    .bind(&season)
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut with_stats = Vec::new();
    for setup in setups {
        let (stats, driver_stats) =
            compute_all_stats(&state.pool, &setup, &catalog, &drivers_catalog, &session_id).await;
        with_stats.push(SetupWithStats {
            setup,
            stats,
            driver_stats,
        });
    }

    templates::setups::list_page(&with_stats, &auth)
}

async fn new(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let drivers_catalog = state.drivers_catalog_for_season(&season).await;
    let categories = state.categories_for_season(&season).await;
    let inventory_by_category =
        load_inventory_by_category(&state.pool, &season, &catalog, &categories, &session_id).await;
    let driver_items = load_driver_inventory(&state.pool, &season, &session_id).await;
    templates::setups::form_page(
        &inventory_by_category,
        &driver_items,
        &drivers_catalog,
        None,
        &auth,
    )
}

fn empty_str_as_none<'de, D>(d: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    if s.trim().is_empty() {
        Ok(None)
    } else {
        s.trim()
            .parse::<i32>()
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
pub struct SetupForm {
    pub name: String,
    #[serde(rename = "engine", deserialize_with = "empty_str_as_none", default)]
    pub engine_id: Option<i32>,
    #[serde(rename = "front_wing", deserialize_with = "empty_str_as_none", default)]
    pub front_wing_id: Option<i32>,
    #[serde(rename = "rear_wing", deserialize_with = "empty_str_as_none", default)]
    pub rear_wing_id: Option<i32>,
    #[serde(rename = "suspension", deserialize_with = "empty_str_as_none", default)]
    pub suspension_id: Option<i32>,
    #[serde(rename = "brakes", deserialize_with = "empty_str_as_none", default)]
    pub brakes_id: Option<i32>,
    #[serde(rename = "gearbox", deserialize_with = "empty_str_as_none", default)]
    pub gearbox_id: Option<i32>,
    #[serde(rename = "battery", deserialize_with = "empty_str_as_none", default)]
    pub battery_id: Option<i32>,
    #[serde(deserialize_with = "empty_str_as_none", default)]
    pub driver1_id: Option<i32>,
    #[serde(deserialize_with = "empty_str_as_none", default)]
    pub driver2_id: Option<i32>,
}

async fn create(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<SetupForm>,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "setup_create",
        serde_json::json!({ "season": season }),
    );
    sqlx::query(
        "INSERT INTO setups (name, engine_id, front_wing_id, rear_wing_id, suspension_id, \
         brakes_id, gearbox_id, battery_id, driver1_id, driver2_id, season, session_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
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
    .bind(&session_id)
    .execute(&state.pool)
    .await?;

    Ok(Redirect::to("/setups"))
}

async fn show(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let drivers_catalog = state.drivers_catalog_for_season(&season).await;
    let setup =
        sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE id = $1 AND session_id = $2")
            .bind(id)
            .bind(&session_id)
            .fetch_one(&state.pool)
            .await?;

    let (stats, driver_stats) =
        compute_all_stats(&state.pool, &setup, &catalog, &drivers_catalog, &session_id).await;

    let additional_stat_label = catalog.iter().find_map(|p| p.additional_stat_name.clone());

    // Load part names for the slots (to show "Default" where missing)
    let categories = state.categories_for_season(&season).await;
    let slot_ids: Vec<i32> = [
        setup.engine_id,
        setup.front_wing_id,
        setup.rear_wing_id,
        setup.suspension_id,
        setup.brakes_id,
        setup.gearbox_id,
        setup.battery_id,
    ]
    .into_iter()
    .flatten()
    .collect();
    let slot_items = if slot_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE id = ANY($1) AND session_id = $2",
        )
        .bind(&slot_ids[..])
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
    };

    // Build per-slot display: (category_name, part_name_or_default, level_or_none)
    let slot_display: Vec<(&str, String, Option<i32>)> = {
        let cat_slots: Vec<(&PartCategory, Option<i32>)> = categories
            .iter()
            .map(|cat| {
                let id = match cat {
                    PartCategory::Engine => setup.engine_id,
                    PartCategory::FrontWing => setup.front_wing_id,
                    PartCategory::RearWing => setup.rear_wing_id,
                    PartCategory::Suspension => setup.suspension_id,
                    PartCategory::Brakes => setup.brakes_id,
                    PartCategory::Gearbox => setup.gearbox_id,
                    PartCategory::Battery => setup.battery_id,
                };
                (cat, id)
            })
            .collect();
        cat_slots
            .into_iter()
            .map(|(cat, id)| {
                let (name, level) = match id {
                    None => ("Default".to_string(), None),
                    Some(part_id) => slot_items
                        .iter()
                        .find(|i| i.id == part_id)
                        .map(|i| (i.part_name.clone(), Some(i.level)))
                        .unwrap_or_else(|| ("Default".to_string(), None)),
                };
                (cat.display_name(), name, level)
            })
            .collect()
    };

    let s = SetupWithStats {
        setup,
        stats,
        driver_stats,
    };

    Ok(crate::templates::layout::page(
        &s.setup.name,
        &auth,
        html! {
            h1 { (&s.setup.name) }

            h2 { "Parts" }
            figure {
                table {
                    thead { tr { th { "Category" } th { "Part" } th { "Lvl" } } }
                    tbody {
                        @for (cat_name, part_name, level) in &slot_display {
                            tr {
                                td { (cat_name) }
                                @if part_name == "Default" {
                                    td colspan="2" class="secondary" { "Default (1/1/1/1 · 1.00s pit)" }
                                } @else {
                                    td { (part_name) }
                                    td { (level.unwrap_or(0)) }
                                }
                            }
                        }
                    }
                }
            }

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

            div class="setup-actions" {
                a href="/setups" role="button" class="outline" { "← Back" }
                a href={"/setups/" (s.setup.id) "/edit"} role="button" { "Edit" }
            }
        },
    ))
}

async fn edit(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    auth: AuthStatus,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
    let setup =
        sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE id = $1 AND session_id = $2")
            .bind(id)
            .bind(&session_id)
            .fetch_one(&state.pool)
            .await?;
    let catalog = state.catalog_for_season(&season).await;
    let drivers_catalog = state.drivers_catalog_for_season(&season).await;
    let categories = state.categories_for_season(&season).await;
    let inventory_by_category =
        load_inventory_by_category(&state.pool, &season, &catalog, &categories, &session_id).await;
    let driver_items = load_driver_inventory(&state.pool, &season, &session_id).await;
    Ok(templates::setups::form_page(
        &inventory_by_category,
        &driver_items,
        &drivers_catalog,
        Some(&setup),
        &auth,
    ))
}

async fn update(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
    Form(form): Form<SetupForm>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query(
        "UPDATE setups SET name=$1, engine_id=$2, front_wing_id=$3, rear_wing_id=$4, \
         suspension_id=$5, brakes_id=$6, gearbox_id=$7, battery_id=$8, \
         driver1_id=$9, driver2_id=$10 \
         WHERE id=$11 AND session_id=$12",
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
    .bind(&session_id)
    .execute(&state.pool)
    .await?;

    Ok(Redirect::to("/setups"))
}

async fn destroy(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
    crate::analytics::fire(
        &state.analytics,
        session_id.clone(),
        "setup_delete",
        serde_json::json!({ "season": season }),
    );
    sqlx::query("DELETE FROM setups WHERE id = $1 AND session_id = $2")
        .bind(id)
        .bind(&session_id)
        .execute(&state.pool)
        .await?;

    Ok(html! {})
}

#[derive(Deserialize)]
struct CompareQuery {
    ids: String,
}

async fn compare(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    auth: AuthStatus,
    Query(q): Query<CompareQuery>,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let catalog = state.catalog_for_season(&season).await;
    let drivers_catalog = state.drivers_catalog_for_season(&season).await;

    let ids: Vec<i32> = q
        .ids
        .split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .collect();

    let mut setups_with_stats: Vec<SetupWithStats> = Vec::new();
    for id in &ids {
        if let Ok(setup) =
            sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE id = $1 AND session_id = $2")
                .bind(id)
                .bind(&session_id)
                .fetch_one(&state.pool)
                .await
        {
            let (stats, driver_stats) =
                compute_all_stats(&state.pool, &setup, &catalog, &drivers_catalog, &session_id)
                    .await;
            setups_with_stats.push(SetupWithStats {
                setup,
                stats,
                driver_stats,
            });
        }
    }

    templates::setups::comparison_page(&setups_with_stats, &auth)
}

async fn compute_all_stats(
    pool: &PgPool,
    setup: &Setup,
    catalog: &[OwnedPartDefinition],
    drivers_catalog: &[OwnedDriverDefinition],
    session_id: &str,
) -> (Stats, DriverStats) {
    let part_stats = compute_part_stats(pool, setup, catalog, session_id).await;
    let driver_stats = compute_driver_stats(pool, setup, drivers_catalog, session_id).await;
    (part_stats, driver_stats)
}

/// Stats for a slot with no assigned part: 1/1/1/1 + 1.0s pit stop.
fn default_part_stats() -> Stats {
    Stats {
        speed: 1,
        cornering: 1,
        power_unit: 1,
        qualifying: 1,
        pit_stop_time: 1.0,
        additional_stat_value: 0,
    }
}

async fn compute_part_stats(
    pool: &PgPool,
    setup: &Setup,
    catalog: &[OwnedPartDefinition],
    session_id: &str,
) -> Stats {
    let slot_ids: [Option<i32>; 7] = [
        setup.engine_id,
        setup.front_wing_id,
        setup.rear_wing_id,
        setup.suspension_id,
        setup.brakes_id,
        setup.gearbox_id,
        setup.battery_id,
    ];

    // Count how many slots are empty (None = Default)
    let default_count = slot_ids.iter().filter(|id| id.is_none()).count();

    let real_ids: Vec<i32> = slot_ids.into_iter().flatten().collect();

    let items = if real_ids.is_empty() {
        vec![]
    } else {
        sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE id = ANY($1) AND session_id = $2",
        )
        .bind(&real_ids[..])
        .bind(session_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    };

    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE session_id = $1")
        .bind(session_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    let mut stats = Stats::default();

    // Accumulate real part stats
    for item in &items {
        if let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name)
            && let Some(level_stats) = part_def.stats_for_level(item.level)
        {
            let mut ps = Stats {
                speed: level_stats.speed,
                cornering: level_stats.cornering,
                power_unit: level_stats.power_unit,
                qualifying: level_stats.qualifying,
                pit_stop_time: level_stats.pit_stop_time,
                additional_stat_value: level_stats.additional_stat_value,
            };
            if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                ps = ps.boosted(b.percentage);
            }
            stats = stats.add(&ps);
        }
    }

    // Each empty slot contributes 1/1/1/1 + 1.0s
    for _ in 0..default_count {
        stats = stats.add(&default_part_stats());
    }

    stats
}

async fn compute_driver_stats(
    pool: &PgPool,
    setup: &Setup,
    drivers_catalog: &[OwnedDriverDefinition],
    session_id: &str,
) -> DriverStats {
    let driver_ids: Vec<i32> = [setup.driver1_id, setup.driver2_id]
        .iter()
        .filter_map(|id| *id)
        .collect();
    if driver_ids.is_empty() {
        return DriverStats::default();
    }

    let items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE id = ANY($1) AND session_id = $2",
    )
    .bind(&driver_ids[..])
    .bind(session_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let boosts =
        sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts WHERE session_id = $1")
            .bind(session_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let mut stats = DriverStats::default();
    for item in &items {
        if let Some(def) = drivers_catalog
            .iter()
            .find(|d| d.name == item.driver_name && d.rarity == item.rarity)
            && let Some(ls) = def.stats_for_level(item.level)
        {
            let mut ds = ls.to_stats();
            if let Some(b) = boosts
                .iter()
                .find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity)
            {
                ds = ds.boosted(b.percentage);
            }
            stats = stats.add(&ds);
        }
    }
    stats
}

pub async fn load_inventory_by_category(
    pool: &PgPool,
    season: &str,
    catalog: &[OwnedPartDefinition],
    categories: &[PartCategory],
    session_id: &str,
) -> Vec<(PartCategory, Vec<(InventoryItem, OwnedLevelStats)>)> {
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(season)
    .bind(session_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    categories
        .iter()
        .map(|cat| {
            let cat_items: Vec<_> = items
                .iter()
                .filter_map(|item| {
                    let part_def = catalog.iter().find(|p| p.name == item.part_name)?;
                    if part_def.category != *cat {
                        return None;
                    }
                    let level_stats = part_def.stats_for_level(item.level)?.clone();
                    Some((item.clone(), level_stats))
                })
                .collect();
            (*cat, cat_items)
        })
        .collect()
}

pub async fn load_driver_inventory(
    pool: &PgPool,
    season: &str,
    session_id: &str,
) -> Vec<DriverInventoryItem> {
    sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 AND session_id = $2 ORDER BY driver_name",
    )
    .bind(season)
    .bind(session_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}
