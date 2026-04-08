use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get};
use axum::{Form, Router};
use maud::html;
use serde::Deserialize;
use sqlx::PgPool;

use crate::data;
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem, Setup, SetupWithStats};
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/setups", get(list).post(create))
        .route("/setups/new", get(new))
        .route("/setups/{id}", get(show).post(update))
        .route("/setups/{id}", delete(destroy))
}

async fn list(State(pool): State<PgPool>) -> impl IntoResponse {
    let setups = sqlx::query_as::<_, Setup>("SELECT * FROM setups ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let mut with_stats = Vec::new();
    for setup in setups {
        let stats = compute_setup_stats(&pool, &setup).await;
        with_stats.push(SetupWithStats { setup, stats });
    }

    templates::setups::list_page(&with_stats)
}

async fn new(State(pool): State<PgPool>) -> impl IntoResponse {
    let inventory_by_category = load_inventory_by_category(&pool).await;
    templates::setups::form_page(&inventory_by_category, None)
}

#[derive(Deserialize)]
pub struct SetupForm {
    pub name: String,
    #[serde(rename = "engine")]
    pub engine_id: i32,
    #[serde(rename = "front_wing")]
    pub front_wing_id: i32,
    #[serde(rename = "rear_wing")]
    pub rear_wing_id: i32,
    #[serde(rename = "suspension")]
    pub suspension_id: i32,
    #[serde(rename = "brakes")]
    pub brakes_id: i32,
    #[serde(rename = "gearbox")]
    pub gearbox_id: i32,
}

async fn create(State(pool): State<PgPool>, Form(form): Form<SetupForm>) -> impl IntoResponse {
    sqlx::query(
        "INSERT INTO setups (name, engine_id, front_wing_id, rear_wing_id, suspension_id, brakes_id, gearbox_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .bind(form.gearbox_id)
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}

async fn show(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    let setup = sqlx::query_as::<_, Setup>("SELECT * FROM setups WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let stats = compute_setup_stats(&pool, &setup).await;
    let s = SetupWithStats { setup, stats };

    templates::layout::page(
        &s.setup.name,
        html! {
            h1 { (&s.setup.name) }
            figure {
                table {
                    thead {
                        tr { th { "Stat" } th { "Value" } }
                    }
                    tbody {
                        tr { td { "Speed" } td { (s.stats.speed) } }
                        tr { td { "Cornering" } td { (s.stats.cornering) } }
                        tr { td { "Power Unit" } td { (s.stats.power_unit) } }
                        tr { td { "Qualifying" } td { (s.stats.qualifying) } }
                        tr { td { "Pit Stop Time" } td { (format!("{:.2}s", s.stats.pit_stop_time)) } }
                        @if s.stats.drs > 0 {
                            tr { td { "DRS" } td { (s.stats.drs) } }
                        }
                        tr { td { strong { "Total Performance" } } td { strong { (s.stats.total_performance()) } } }
                    }
                }
            }
            a href="/setups" role="button" class="outline" { "← Back to setups" }
        },
    )
}

async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Form(form): Form<SetupForm>,
) -> impl IntoResponse {
    sqlx::query(
        "UPDATE setups SET name=$1, engine_id=$2, front_wing_id=$3, rear_wing_id=$4, suspension_id=$5, brakes_id=$6, gearbox_id=$7
         WHERE id=$8",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .bind(form.gearbox_id)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}

async fn destroy(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    sqlx::query("DELETE FROM setups WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    html! {}
}

async fn compute_setup_stats(pool: &PgPool, setup: &Setup) -> Stats {
    let part_ids = [
        setup.engine_id,
        setup.front_wing_id,
        setup.rear_wing_id,
        setup.suspension_id,
        setup.brakes_id,
        setup.gearbox_id,
    ];

    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE id = ANY($1)",
    )
    .bind(&part_ids[..])
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts")
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    let mut stats = Stats::default();
    for item in &items {
        if let Some(part_def) = data::find_part(&item.part_name) {
            if let Some(level_stats) = part_def.stats_for_level(item.level) {
                let mut part_stats = Stats {
                    speed: level_stats.speed,
                    cornering: level_stats.cornering,
                    power_unit: level_stats.power_unit,
                    qualifying: level_stats.qualifying,
                    pit_stop_time: level_stats.pit_stop_time,
                    drs: level_stats.drs,
                };
                if let Some(boost) = boosts.iter().find(|b| b.part_name == item.part_name) {
                    part_stats = part_stats.boosted(boost.percentage);
                }
                stats = stats.add(&part_stats);
            }
        }
    }
    stats
}

/// For each category, load inventory items that belong to that category
async fn load_inventory_by_category(pool: &PgPool) -> Vec<(PartCategory, Vec<(InventoryItem, &'static data::LevelStats)>)> {
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory ORDER BY part_name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    PartCategory::all()
        .iter()
        .map(|cat| {
            let cat_items: Vec<_> = items
                .iter()
                .filter_map(|item| {
                    let part_def = data::find_part(&item.part_name)?;
                    if part_def.category != *cat {
                        return None;
                    }
                    let level_stats = part_def.stats_for_level(item.level)?;
                    Some((item.clone(), level_stats))
                })
                .collect();
            (*cat, cat_items)
        })
        .collect()
}
