use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::AppState;
use crate::auth::AuthStatus;
use crate::drivers_data::DriverRarity;
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
        .route("/setups/{id}/delete", post(destroy))
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

    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE session_id = $1")
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    let driver_boosts =
        sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts WHERE session_id = $1")
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

    // Compute base totals (without boosts) so we can show the boost delta in the list.
    // Only do the extra work when boosts actually exist for this session.
    let base_totals: Vec<(i32, i32)> = if boosts.is_empty() && driver_boosts.is_empty() {
        with_stats
            .iter()
            .map(|s| (s.stats.total_performance(), s.driver_stats.total()))
            .collect()
    } else {
        // Load all inventory once instead of N per-setup queries.
        let all_inv = sqlx::query_as::<_, InventoryItem>(
            "SELECT * FROM inventory WHERE season = $1 AND session_id = $2",
        )
        .bind(&season)
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
        let all_drv = sqlx::query_as::<_, DriverInventoryItem>(
            "SELECT * FROM driver_inventory WHERE season = $1 AND session_id = $2",
        )
        .bind(&season)
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        with_stats
            .iter()
            .map(|s| {
                let slot_ids = [
                    s.setup.engine_id,
                    s.setup.front_wing_id,
                    s.setup.rear_wing_id,
                    s.setup.suspension_id,
                    s.setup.brakes_id,
                    s.setup.gearbox_id,
                    s.setup.battery_id,
                ];
                let default_count = slot_ids.iter().filter(|id| id.is_none()).count();
                let mut base_parts = Stats::default();
                for slot_id in slot_ids.into_iter().flatten() {
                    if let Some(item) = all_inv.iter().find(|i| i.id == slot_id)
                        && let Some(def) = catalog.iter().find(|p| p.name == item.part_name)
                        && let Some(ls) = def.stats_for_level(item.level)
                    {
                        base_parts = base_parts.add(&Stats {
                            speed: ls.speed,
                            cornering: ls.cornering,
                            power_unit: ls.power_unit,
                            qualifying: ls.qualifying,
                            pit_stop_time: ls.pit_stop_time,
                            additional_stat_value: ls.additional_stat_value,
                        });
                    }
                }
                for _ in 0..default_count {
                    base_parts = base_parts.add(&default_part_stats());
                }

                let base_driver_total: i32 = [s.setup.driver1_id, s.setup.driver2_id]
                    .into_iter()
                    .flatten()
                    .filter_map(|id| all_drv.iter().find(|i| i.id == id))
                    .filter_map(|item| {
                        let def = drivers_catalog
                            .iter()
                            .find(|d| d.name == item.driver_name && d.rarity == item.rarity)?;
                        let ls = def.stats_for_level(item.level)?;
                        Some(ls.to_stats().total())
                    })
                    .sum();

                (base_parts.total_performance(), base_driver_total)
            })
            .collect()
    };

    templates::setups::list_page(&with_stats, &base_totals, &auth)
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

    // Load boosts so we can show them explicitly in the UI.
    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE session_id = $1")
        .bind(&session_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    let driver_boosts =
        sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts WHERE session_id = $1")
            .bind(&session_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    // (category_name, part_name, level, rarity_css_class, boost_pct)
    let slot_display: Vec<(&str, String, Option<i32>, &'static str, Option<i32>)> = {
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
                let (name, level, rarity_class, boost_pct) = match id {
                    None => ("Default".to_string(), None, "", None),
                    Some(part_id) => slot_items
                        .iter()
                        .find(|i| i.id == part_id)
                        .map(|i| {
                            let rarity_class = catalog
                                .iter()
                                .find(|p| p.name == i.part_name)
                                .map(|p| p.rarity_css_class())
                                .unwrap_or("");
                            let boost_pct = boosts
                                .iter()
                                .find(|b| b.part_name == i.part_name)
                                .map(|b| b.percentage);
                            (i.part_name.clone(), Some(i.level), rarity_class, boost_pct)
                        })
                        .unwrap_or_else(|| ("Default".to_string(), None, "", None)),
                };
                (cat.display_name(), name, level, rarity_class, boost_pct)
            })
            .collect()
    };

    let driver_slot_ids: Vec<i32> = [setup.driver1_id, setup.driver2_id]
        .into_iter()
        .flatten()
        .collect();

    let driver_slot_items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE id = ANY($1) AND session_id = $2",
    )
    .bind(&driver_slot_ids[..])
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // Per-driver display in slot order (driver1 first, driver2 second) so the
    // 3-column comparison table always shows driver 1 on the left.
    let driver_slot_display: Vec<(String, &'static str, DriverStats, DriverStats, Option<i32>)> =
        [setup.driver1_id, setup.driver2_id]
            .into_iter()
            .flatten()
            .filter_map(|slot_id| {
                let item = driver_slot_items.iter().find(|i| i.id == slot_id)?;
                let def = drivers_catalog
                    .iter()
                    .find(|d| d.name == item.driver_name && d.rarity == item.rarity)?;
                let ls = def.stats_for_level(item.level)?;
                let base = ls.to_stats();
                let boost_pct = driver_boosts
                    .iter()
                    .find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity)
                    .map(|b| b.percentage);
                let boosted = boost_pct.map_or_else(|| base.clone(), |pct| base.boosted(pct));
                let rarity_class =
                    DriverRarity::from_db(&item.rarity).map_or("", |r| r.css_class());
                Some((
                    item.driver_name.clone(),
                    rarity_class,
                    base,
                    boosted,
                    boost_pct,
                ))
            })
            .collect();

    // Base part stats (no boosts) so we can show the boost delta on each stat.
    let base_part_stats = {
        let all_slot_ids = [
            setup.engine_id,
            setup.front_wing_id,
            setup.rear_wing_id,
            setup.suspension_id,
            setup.brakes_id,
            setup.gearbox_id,
            setup.battery_id,
        ];
        let default_count = all_slot_ids.iter().filter(|id| id.is_none()).count();
        let mut base = Stats::default();
        for item in &slot_items {
            if let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name)
                && let Some(ls) = part_def.stats_for_level(item.level)
            {
                base = base.add(&Stats {
                    speed: ls.speed,
                    cornering: ls.cornering,
                    power_unit: ls.power_unit,
                    qualifying: ls.qualifying,
                    pit_stop_time: ls.pit_stop_time,
                    additional_stat_value: ls.additional_stat_value,
                });
            }
        }
        for _ in 0..default_count {
            base = base.add(&default_part_stats());
        }
        base
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
            div style="display: flex; justify-content: space-between; align-items: center; gap: 20px;" {
                h1 { (&s.setup.name) }
                form method="post" action="/setup/share" {
                    button type="submit" class="save-form-btn outline" { "Share" }
                    input type="hidden" name="name" value=(&s.setup.name);
                    input type="hidden" name="brakes_id" value=(&s.setup.brakes_id.unwrap_or_default().to_string());
                    input type="hidden" name="gearbox_id" value=(&s.setup.gearbox_id.unwrap_or_default().to_string());
                    input type="hidden" name="rear_wing_id" value=(&s.setup.rear_wing_id.unwrap_or_default().to_string());
                    input type="hidden" name="front_wing_id" value=(&s.setup.front_wing_id.unwrap_or_default().to_string());
                    input type="hidden" name="suspension_id" value=(&s.setup.suspension_id.unwrap_or_default().to_string());
                    input type="hidden" name="engine_id" value=(&s.setup.engine_id.unwrap_or_default().to_string());
                    input type="hidden" name="battery_id" value=(&s.setup.battery_id.unwrap_or_default().to_string());
                    input type="hidden" name="driver1_id" value=(&s.setup.driver1_id.unwrap_or_default().to_string());
                    input type="hidden" name="driver2_id" value=(&s.setup.driver2_id.unwrap_or_default().to_string());
                }
            }

            p {
                @let base_p = base_part_stats.total_performance();
                @let boost_p = s.stats.total_performance();
                @let base_d: i32 = driver_slot_display.iter().map(|d| d.2.total()).sum();
                @let boost_d = s.driver_stats.total();
                @let base_c = base_p + base_d;
                @let boost_c = boost_p + boost_d;
                "Combined score: "
                strong { (base_c) }
                @if base_c != boost_c {
                    " " span class="secondary" { "(" strong { (boost_c) } " " span class="upgrade-positive" { "↑" } ")" }
                }
                " = "
                (base_p)
                @if base_p != boost_p {
                    " " span class="secondary" { "(" (boost_p) " " span class="upgrade-positive" { "↑" } ")" }
                }
                " parts + "
                (base_d)
                @if base_d != boost_d {
                    " " span class="secondary" { "(" (boost_d) " " span class="upgrade-positive" { "↑" } ")" }
                }
                " drivers"
            }

            div class="grid" {
                div style="display: flex; flex-direction: column; width: fit-content; min-width: 250px;" {
                    h2 { "Parts" }
                    figure style="margin: 0;" {
                        table {
                            thead { tr { th { "Category" } th { "Part" } th { "Lvl" } } }
                            tbody {
                                @for (cat_name, part_name, level, rarity_class, boost_pct) in &slot_display {
                                    tr {
                                        td { (cat_name) }
                                        @if part_name == "Default" {
                                            td colspan="2" class="secondary" { "Default (1/1/1/1 · 1.00s pit)" }
                                        } @else {
                                            td { span class=(*rarity_class) { (part_name) } }
                                            td {
                                                (level.unwrap_or(0))
                                                @if let Some(pct) = boost_pct {
                                                    " " span class="secondary" {
                                                        "(+" (pct) "% " span class="upgrade-positive" { "↑" } ")"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div style="display: flex; flex-direction: column; width: fit-content; min-width: 250px;" {
                    h2 { "Part Stats" }
                    figure style="margin: 0;" {
                        table {
                            thead { tr { th { "Stat" } th { "Value" } } }
                            tbody {
                                @let ps = |label: &str, base_val: i32, boosted_val: i32| -> maud::Markup {
                                    maud::html! {
                                        tr {
                                            td { (label) }
                                            td {
                                                @if base_val != boosted_val {
                                                    (base_val) " "
                                                    span class="secondary" { "(" (boosted_val) " " span class="upgrade-positive" { "↑" } ")" }
                                                } @else { (boosted_val) }
                                            }
                                        }
                                    }
                                };
                                (ps("Speed", base_part_stats.speed, s.stats.speed))
                                (ps("Cornering", base_part_stats.cornering, s.stats.cornering))
                                (ps("Power Unit", base_part_stats.power_unit, s.stats.power_unit))
                                (ps("Qualifying", base_part_stats.qualifying, s.stats.qualifying))
                                tr {
                                    td { "Pit Stop Time" }
                                    td {
                                        @let base_pit = base_part_stats.pit_stop_time;
                                        @let boost_pit = s.stats.pit_stop_time;
                                        @if (base_pit - boost_pit).abs() > 0.001 {
                                            (format!("{:.2}s", base_pit)) " "
                                            span class="secondary" { "(" (format!("{:.2}s", boost_pit)) " " span class="upgrade-positive" { "↑" } ")" }
                                        } @else {
                                            (format!("{:.2}s", boost_pit))
                                        }
                                    }
                                }
                                @if s.stats.additional_stat_value > 0 {
                                    @let label = additional_stat_label.as_deref().unwrap_or("Special");
                                    tr { td { (label) } td { (s.stats.additional_stat_value) } }
                                }
                                tr {
                                    td { strong { "Total Performance" } }
                                    td {
                                        @let base_total = base_part_stats.total_performance();
                                        @let boost_total = s.stats.total_performance();
                                        @if base_total != boost_total {
                                            (base_total) " "
                                            span class="secondary" { "(" (boost_total) " " span class="upgrade-positive" { "↑" } ")" }
                                        } @else { strong { (boost_total) } }
                                    }
                                }
                            }
                        }
                    }
                }
                div style="display: flex; flex-direction: column; width: fit-content; min-width: 250px;" {
                    @if !driver_slot_display.is_empty() {
                        h2 { "Drivers" }
                        @if driver_slot_display.len() >= 2 {
                            @let d1 = &driver_slot_display[0];
                            @let d2 = &driver_slot_display[1];
                            figure style="margin:0" {
                                table style="width:100%" {
                                    thead {
                                        tr {
                                            th style="text-align:right;width:38%" {
                                                span class=(d1.1) { (&d1.0) }
                                            }
                                            th style="text-align:center;width:24%" {}
                                            th style="text-align:left;width:38%" {
                                                span class=(d2.1) { (&d2.0) }
                                            }
                                        }
                                    }
                                    tbody {
                                        @let dr = |label: &str, b1: i32, v1: i32, p1: Option<i32>, b2: i32, v2: i32, p2: Option<i32>| -> maud::Markup {
                                            maud::html! {
                                                tr {
                                                    td style="text-align:right" {
                                                        @if p1.is_some() && b1 != v1 {
                                                            (b1) " " span class="secondary" { "(" (v1) " " span class="upgrade-positive" { "↑" } ")" }
                                                        } @else { (v1) }
                                                    }
                                                    td style="text-align:center" class="secondary" { (label) }
                                                    td {
                                                        @if p2.is_some() && b2 != v2 {
                                                            (b2) " " span class="secondary" { "(" (v2) " " span class="upgrade-positive" { "↑" } ")" }
                                                        } @else { (v2) }
                                                    }
                                                }
                                            }
                                        };
                                        (dr("Overtaking",  d1.2.overtaking,      d1.3.overtaking,      d1.4, d2.2.overtaking,      d2.3.overtaking,      d2.4))
                                        (dr("Defending",   d1.2.defending,        d1.3.defending,        d1.4, d2.2.defending,        d2.3.defending,        d2.4))
                                        (dr("Qualifying",  d1.2.qualifying,       d1.3.qualifying,       d1.4, d2.2.qualifying,       d2.3.qualifying,       d2.4))
                                        (dr("Race Start",  d1.2.race_start,       d1.3.race_start,       d1.4, d2.2.race_start,       d2.3.race_start,       d2.4))
                                        (dr("Tyre Mgmt",   d1.2.tyre_management,  d1.3.tyre_management,  d1.4, d2.2.tyre_management,  d2.3.tyre_management,  d2.4))
                                    }
                                    tfoot {
                                        tr {
                                            td style="text-align:right" {
                                                @let (b1t, v1t) = (d1.2.total(), d1.3.total());
                                                @if d1.4.is_some() && b1t != v1t {
                                                    (b1t) " " span class="secondary" { "(" (v1t) " " span class="upgrade-positive" { "↑" } ")" }
                                                } @else { strong { (v1t) } }
                                            }
                                            td style="text-align:center" { strong { "Total" } }
                                            td {
                                                @let (b2t, v2t) = (d2.2.total(), d2.3.total());
                                                @if d2.4.is_some() && b2t != v2t {
                                                    (b2t) " " span class="secondary" { "(" (v2t) " " span class="upgrade-positive" { "↑" } ")" }
                                                } @else { strong { (v2t) } }
                                            }
                                        }
                                    }
                                }
                            }
                        } @else {
                            @let d = &driver_slot_display[0];
                            p style="margin:0 0 0.3rem" { span class=(d.1) { (&d.0) } }
                            figure style="margin:0" {
                                table {
                                    thead { tr { th { "Stat" } th { "Value" } } }
                                    tbody {
                                        @let ds = |label: &str, bv: i32, v: i32, pct: Option<i32>| -> maud::Markup {
                                            maud::html! {
                                                tr {
                                                    td { (label) }
                                                    td {
                                                        @if pct.is_some() && bv != v {
                                                            (bv) " " span class="secondary" { "(" (v) " " span class="upgrade-positive" { "↑" } ")" }
                                                        } @else { (v) }
                                                    }
                                                }
                                            }
                                        };
                                        (ds("Overtaking", d.2.overtaking,     d.3.overtaking,     d.4))
                                        (ds("Defending",  d.2.defending,       d.3.defending,       d.4))
                                        (ds("Qualifying", d.2.qualifying,      d.3.qualifying,      d.4))
                                        (ds("Race Start", d.2.race_start,      d.3.race_start,      d.4))
                                        (ds("Tyre Mgmt",  d.2.tyre_management, d.3.tyre_management, d.4))
                                        tr {
                                            td { strong { "Total" } }
                                            td {
                                                @let (bt, vt) = (d.2.total(), d.3.total());
                                                @if d.4.is_some() && bt != vt {
                                                    (bt) " " span class="secondary" { "(" (vt) " " span class="upgrade-positive" { "↑" } ")" }
                                                } @else { strong { (vt) } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div class="setup-actions" style="display: flex; justify-content: space-between; align-items: center;" {
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
