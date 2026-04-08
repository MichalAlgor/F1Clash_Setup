use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;
use crate::AppState;

use crate::data;
use crate::drivers_data;
use crate::models::driver::{DriverBoost, DriverInventoryItem, DriverStats};
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem};
use crate::templates;

pub fn router() -> Router<AppState> {
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
    // Part priorities
    #[serde(default)]
    pub speed: bool,
    #[serde(default)]
    pub cornering: bool,
    #[serde(default)]
    pub power_unit: bool,
    #[serde(default)]
    pub qualifying: bool,
    // Driver priorities
    #[serde(default)]
    pub overtaking: bool,
    #[serde(default)]
    pub defending: bool,
    #[serde(default)]
    pub d_qualifying: bool,
    #[serde(default)]
    pub race_start: bool,
    #[serde(default)]
    pub tyre_management: bool,
}

struct ResolvedPart {
    item: InventoryItem,
    stats: Stats,
}

struct ResolvedDriver {
    item: DriverInventoryItem,
    stats: DriverStats,
}

#[derive(Clone, Default)]
pub struct DriverPriorities {
    pub overtaking: bool,
    pub defending: bool,
    pub qualifying: bool,
    pub race_start: bool,
    pub tyre_management: bool,
}

impl DriverPriorities {
    pub fn any_selected(&self) -> bool {
        self.overtaking || self.defending || self.qualifying || self.race_start || self.tyre_management
    }

    pub fn labels(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.overtaking { out.push("Overtaking"); }
        if self.defending { out.push("Defending"); }
        if self.qualifying { out.push("Qualifying"); }
        if self.race_start { out.push("Race Start"); }
        if self.tyre_management { out.push("Tyre Mgmt"); }
        out
    }

    fn score(&self, stats: &DriverStats) -> (i32, i32) {
        if !self.any_selected() {
            let total = stats.total();
            return (total, total);
        }
        let mut values = Vec::new();
        if self.overtaking { values.push(stats.overtaking); }
        if self.defending { values.push(stats.defending); }
        if self.qualifying { values.push(stats.qualifying); }
        if self.race_start { values.push(stats.race_start); }
        if self.tyre_management { values.push(stats.tyre_management); }
        let min = *values.iter().min().unwrap();
        let sum: i32 = values.iter().sum();
        (min, sum)
    }
}

async fn run(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<OptimizerQuery>,
) -> impl IntoResponse {
    let season = state.season().await;
    let part_priorities = crate::data::StatPriorities {
        speed: query.speed,
        cornering: query.cornering,
        power_unit: query.power_unit,
        qualifying: query.qualifying,
    };
    let driver_priorities = DriverPriorities {
        overtaking: query.overtaking,
        defending: query.defending,
        qualifying: query.d_qualifying,
        race_start: query.race_start,
        tyre_management: query.tyre_management,
    };

    // === Resolve parts ===
    let items = sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory WHERE season = $1 ORDER BY part_name")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();
    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE season = $1")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();

    let categories = PartCategory::all();
    let mut parts_per_cat: Vec<Vec<ResolvedPart>> = Vec::new();
    for cat in categories {
        let cat_parts: Vec<ResolvedPart> = items.iter().filter_map(|item| {
            let part_def = data::find_part(&item.part_name)?;
            if part_def.category != *cat { return None; }
            let level_stats = part_def.stats_for_level(item.level)?;
            let mut s = Stats {
                speed: level_stats.speed, cornering: level_stats.cornering,
                power_unit: level_stats.power_unit, qualifying: level_stats.qualifying,
                pit_stop_time: level_stats.pit_stop_time, drs: level_stats.drs,
            };
            if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                s = s.boosted(b.percentage);
            }
            Some(ResolvedPart { item: item.clone(), stats: s })
        }).collect();
        parts_per_cat.push(cat_parts);
    }

    // === Resolve drivers ===
    let driver_items = sqlx::query_as::<_, DriverInventoryItem>("SELECT * FROM driver_inventory WHERE season = $1 ORDER BY driver_name")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();
    let driver_boosts = sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts WHERE season = $1")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();

    let resolved_drivers: Vec<ResolvedDriver> = driver_items.iter().filter_map(|item| {
        let def = drivers_data::find_driver_by_db(&item.driver_name, &item.rarity)?;
        let ls = def.stats_for_level(item.level)?;
        let mut ds = ls.to_stats();
        if let Some(b) = driver_boosts.iter().find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity) {
            ds = ds.boosted(b.percentage);
        }
        Some(ResolvedDriver { item: item.clone(), stats: ds })
    }).collect();

    // === Generate driver pairs (including "no drivers") ===
    // Each pair is (Option<idx>, Option<idx>)
    let mut driver_pairs: Vec<(Option<usize>, Option<usize>)> = vec![(None, None)];
    for i in 0..resolved_drivers.len() {
        driver_pairs.push((Some(i), None));
        for j in (i+1)..resolved_drivers.len() {
            driver_pairs.push((Some(i), Some(j)));
        }
    }

    // === Check parts feasibility ===
    if parts_per_cat.iter().any(|c| c.is_empty()) {
        return templates::optimizer::result_page(
            &part_priorities, &driver_priorities, &[], None, None,
            &Stats::default(), &DriverStats::default(),
        );
    }

    // === Brute-force ===
    let sizes: Vec<usize> = parts_per_cat.iter().map(|c| c.len()).collect();
    let total_part_combos: usize = sizes.iter().product();

    // Best: (part_indices, driver_pair_idx, score tuple)
    let mut best: Option<(Vec<usize>, usize, (i32, i32, i32, i32))> = None;

    let mut part_indices = vec![0usize; categories.len()];

    for _ in 0..total_part_combos {
        let mut part_stats = Stats::default();
        for (cat_idx, &pi) in part_indices.iter().enumerate() {
            part_stats = part_stats.add(&parts_per_cat[cat_idx][pi].stats);
        }
        let (p_min, p_sum) = score_part_combo(&part_stats, &part_priorities);

        for (dp_idx, (d1, d2)) in driver_pairs.iter().enumerate() {
            let mut ds = DriverStats::default();
            if let Some(i) = d1 { ds = ds.add(&resolved_drivers[*i].stats); }
            if let Some(i) = d2 { ds = ds.add(&resolved_drivers[*i].stats); }
            let (d_min, d_sum) = driver_priorities.score(&ds);

            let score = (p_min, p_sum, d_min, d_sum);
            let is_better = match &best {
                None => true,
                Some((_, _, best_score)) => score > *best_score,
            };
            if is_better {
                best = Some((part_indices.clone(), dp_idx, score));
            }
        }

        // Odometer increment
        let mut carry = true;
        for i in (0..part_indices.len()).rev() {
            if carry {
                part_indices[i] += 1;
                if part_indices[i] >= sizes[i] { part_indices[i] = 0; } else { carry = false; }
            }
        }
    }

    // === Build results ===
    let (part_picks, driver1, driver2, total_parts, total_drivers) = match best {
        Some((pidx, dp_idx, _)) => {
            let picks: Vec<_> = pidx.iter().enumerate().map(|(ci, &pi)| {
                let rp = &parts_per_cat[ci][pi];
                (categories[ci], rp.item.clone(), rp.stats.clone())
            }).collect();
            let (d1_idx, d2_idx) = driver_pairs[dp_idx];
            let d1 = d1_idx.map(|i| &resolved_drivers[i]);
            let d2 = d2_idx.map(|i| &resolved_drivers[i]);
            let ts = picks.iter().fold(Stats::default(), |a, (_, _, s)| a.add(s));
            let mut ds = DriverStats::default();
            if let Some(d) = d1 { ds = ds.add(&d.stats); }
            if let Some(d) = d2 { ds = ds.add(&d.stats); }
            (
                picks,
                d1.map(|d| (d.item.clone(), d.stats.clone())),
                d2.map(|d| (d.item.clone(), d.stats.clone())),
                ts, ds,
            )
        }
        None => (Vec::new(), None, None, Stats::default(), DriverStats::default()),
    };

    templates::optimizer::result_page(
        &part_priorities, &driver_priorities,
        &part_picks, driver1.as_ref(), driver2.as_ref(),
        &total_parts, &total_drivers,
    )
}

fn score_part_combo(stats: &Stats, priorities: &crate::data::StatPriorities) -> (i32, i32) {
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
    pub driver1_id: Option<i32>,
    pub driver2_id: Option<i32>,
}

async fn save(State(state): State<AppState>, Form(form): Form<SaveForm>) -> impl IntoResponse {
    let season = state.season().await;
    sqlx::query(
        "INSERT INTO setups (name, engine_id, front_wing_id, rear_wing_id, suspension_id, brakes_id, gearbox_id, driver1_id, driver2_id, season)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&form.name)
    .bind(form.engine_id)
    .bind(form.front_wing_id)
    .bind(form.rear_wing_id)
    .bind(form.suspension_id)
    .bind(form.brakes_id)
    .bind(form.gearbox_id)
    .bind(form.driver1_id)
    .bind(form.driver2_id)
    .bind(&season)
    .execute(&state.pool)
    .await
    .unwrap();

    Redirect::to("/setups")
}
