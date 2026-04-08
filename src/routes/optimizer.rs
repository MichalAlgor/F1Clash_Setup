use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;
use sqlx::PgPool;

use crate::data::{self, StatPriorities};
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem};
use crate::templates;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/optimizer", get(form))
        .route("/optimizer/run", get(run))
        .route("/optimizer/save", axum::routing::post(save))
}

async fn form() -> impl IntoResponse {
    templates::optimizer::form_page()
}

#[derive(Deserialize)]
pub struct OptimizerQuery {
    #[serde(default)]
    pub speed: bool,
    #[serde(default)]
    pub cornering: bool,
    #[serde(default)]
    pub power_unit: bool,
    #[serde(default)]
    pub qualifying: bool,
}

/// Resolved inventory part with its boosted stats
struct ResolvedPart {
    item: InventoryItem,
    stats: Stats,
}

/// Brute-force all combinations of one part per category,
/// score by: maximize minimum prioritized stat (balance), then total prioritized sum.
async fn run(
    State(pool): State<PgPool>,
    axum::extract::Query(query): axum::extract::Query<OptimizerQuery>,
) -> impl IntoResponse {
    let priorities = StatPriorities {
        speed: query.speed,
        cornering: query.cornering,
        power_unit: query.power_unit,
        qualifying: query.qualifying,
    };

    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory ORDER BY part_name",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    // Group resolved parts by category (in display order)
    let categories = PartCategory::all();
    let mut parts_per_cat: Vec<Vec<ResolvedPart>> = Vec::new();

    for cat in categories {
        let cat_parts: Vec<ResolvedPart> = items
            .iter()
            .filter_map(|item| {
                let part_def = data::find_part(&item.part_name)?;
                if part_def.category != *cat {
                    return None;
                }
                let level_stats = part_def.stats_for_level(item.level)?;
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
                Some(ResolvedPart { item: item.clone(), stats: part_stats })
            })
            .collect();
        parts_per_cat.push(cat_parts);
    }

    // Check if any category is empty
    if parts_per_cat.iter().any(|c| c.is_empty()) {
        return templates::optimizer::result_page(&priorities, &[], &Stats::default());
    }

    // Brute-force all combinations
    let mut best_combo: Option<(Vec<usize>, i32, i32)> = None; // (indices, min_priority, sum_priority)

    let sizes: Vec<usize> = parts_per_cat.iter().map(|c| c.len()).collect();
    let total_combos: usize = sizes.iter().product();
    let mut indices = vec![0usize; categories.len()];

    for _ in 0..total_combos {
        // Compute combined stats for this combination
        let mut combined = Stats::default();
        for (cat_idx, &part_idx) in indices.iter().enumerate() {
            combined = combined.add(&parts_per_cat[cat_idx][part_idx].stats);
        }

        let (min_pri, sum_pri) = score_combo(&combined, &priorities);

        let is_better = match &best_combo {
            None => true,
            Some((_, best_min, best_sum)) => {
                (min_pri, sum_pri) > (*best_min, *best_sum)
            }
        };

        if is_better {
            best_combo = Some((indices.clone(), min_pri, sum_pri));
        }

        // Increment indices (odometer-style)
        let mut carry = true;
        for i in (0..indices.len()).rev() {
            if carry {
                indices[i] += 1;
                if indices[i] >= sizes[i] {
                    indices[i] = 0;
                } else {
                    carry = false;
                }
            }
        }
    }

    let picks: Vec<(PartCategory, InventoryItem, Stats)> = match best_combo {
        Some((best_indices, _, _)) => {
            best_indices
                .iter()
                .enumerate()
                .map(|(cat_idx, &part_idx)| {
                    let rp = &parts_per_cat[cat_idx][part_idx];
                    (categories[cat_idx], rp.item.clone(), rp.stats.clone())
                })
                .collect()
        }
        None => Vec::new(),
    };

    let total_stats = picks.iter().fold(Stats::default(), |acc, (_, _, s)| acc.add(s));

    templates::optimizer::result_page(&priorities, &picks, &total_stats)
}

/// Score a combination: (minimum of prioritized stats, sum of prioritized stats).
/// If no priorities selected, use (total_performance, total_performance).
fn score_combo(stats: &Stats, priorities: &StatPriorities) -> (i32, i32) {
    if !priorities.any_selected() {
        let total = stats.total_performance();
        return (total, total);
    }

    let mut values = Vec::new();
    if priorities.speed { values.push(stats.speed); }
    if priorities.cornering { values.push(stats.cornering); }
    if priorities.power_unit { values.push(stats.power_unit); }
    if priorities.qualifying { values.push(stats.qualifying); }

    let min = *values.iter().min().unwrap();
    let sum: i32 = values.iter().sum();
    (min, sum)
}

#[derive(Deserialize)]
pub struct SaveForm {
    pub name: String,
    pub brakes_id: i32,
    pub gearbox_id: i32,
    pub rear_wing_id: i32,
    pub front_wing_id: i32,
    pub suspension_id: i32,
    pub engine_id: i32,
}

async fn save(State(pool): State<PgPool>, Form(form): Form<SaveForm>) -> impl IntoResponse {
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
