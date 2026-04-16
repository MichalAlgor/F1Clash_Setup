use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;
use crate::AppState;

use crate::auth::AuthStatus;
use crate::data::StatPriorities;
use crate::drivers_data;
use crate::models::driver::{DriverBoost, DriverInventoryItem, DriverStats};
use crate::models::part::Stats;
use crate::models::setup::{Boost, InventoryItem};
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/optimizer", get(form))
        .route("/optimizer/run", get(run))
        .route("/optimizer/save", axum::routing::post(save))
}

async fn form(auth: AuthStatus) -> impl IntoResponse {
    templates::optimizer::form_page(&auth)
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
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub max_part_series: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub max_driver_series: Option<i32>,
}

fn deserialize_opt_i32<'de, D>(d: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    if s.trim().is_empty() {
        Ok(None)
    } else {
        s.trim().parse::<i32>().map(Some).map_err(serde::de::Error::custom)
    }
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
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = state.season().await;
    let catalog = state.catalog_for_season().await;

    let part_priorities = StatPriorities {
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

    let items = sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory WHERE season = $1 ORDER BY part_name")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();
    let boosts = sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE season = $1")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();

    let max_part_series = query.max_part_series.unwrap_or(i32::MAX);
    let max_driver_series = query.max_driver_series.unwrap_or(i32::MAX);

    let season_categories = state.categories_for_season().await;
    let categories = season_categories.as_slice();
    let mut parts_per_cat: Vec<Vec<ResolvedPart>> = Vec::new();
    for cat in categories {
        let cat_parts: Vec<ResolvedPart> = items.iter().filter_map(|item| {
            let part_def = catalog.iter().find(|p| p.name == item.part_name)?;
            if part_def.series > max_part_series { return None; }
            if part_def.category != *cat { return None; }
            let level_stats = part_def.stats_for_level(item.level)?;
            let mut s = Stats {
                speed: level_stats.speed, cornering: level_stats.cornering,
                power_unit: level_stats.power_unit, qualifying: level_stats.qualifying,
                pit_stop_time: level_stats.pit_stop_time,
                additional_stat_value: level_stats.additional_stat_value,
            };
            if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                s = s.boosted(b.percentage);
            }
            Some(ResolvedPart { item: item.clone(), stats: s })
        }).collect();
        parts_per_cat.push(cat_parts);
    }

    let driver_items = sqlx::query_as::<_, DriverInventoryItem>("SELECT * FROM driver_inventory WHERE season = $1 ORDER BY driver_name")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();
    let driver_boosts = sqlx::query_as::<_, DriverBoost>("SELECT * FROM driver_boosts WHERE season = $1")
        .bind(&season).fetch_all(&state.pool).await.unwrap_or_default();

    let resolved_drivers: Vec<ResolvedDriver> = driver_items.iter().filter_map(|item| {
        let def = drivers_data::find_driver_by_db(&item.driver_name, &item.rarity)?;
        let driver_series = def.series.parse::<i32>().unwrap_or(i32::MAX);
        if driver_series > max_driver_series { return None; }
        let ls = def.stats_for_level(item.level)?;
        let mut ds = ls.to_stats();
        if let Some(b) = driver_boosts.iter().find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity) {
            ds = ds.boosted(b.percentage);
        }
        Some(ResolvedDriver { item: item.clone(), stats: ds })
    }).collect();

    let mut driver_pairs: Vec<(Option<usize>, Option<usize>)> = vec![(None, None)];
    for i in 0..resolved_drivers.len() {
        driver_pairs.push((Some(i), None));
        for j in (i+1)..resolved_drivers.len() {
            driver_pairs.push((Some(i), Some(j)));
        }
    }

    if parts_per_cat.iter().any(|c| c.is_empty()) {
        return templates::optimizer::result_page(
            &part_priorities, &driver_priorities, &[], None, None,
            &Stats::default(), &DriverStats::default(), &auth,
        );
    }

    let sizes: Vec<usize> = parts_per_cat.iter().map(|c| c.len()).collect();
    let total_part_combos: usize = sizes.iter().product();

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

        let mut carry = true;
        for i in (0..part_indices.len()).rev() {
            if carry {
                part_indices[i] += 1;
                if part_indices[i] >= sizes[i] { part_indices[i] = 0; } else { carry = false; }
            }
        }
    }

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
        &total_parts, &total_drivers, &auth,
    )
}

fn score_part_combo(stats: &Stats, priorities: &StatPriorities) -> (i32, i32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ds(overtaking: i32, defending: i32, qualifying: i32, race_start: i32, tyre_management: i32) -> DriverStats {
        DriverStats { overtaking, defending, qualifying, race_start, tyre_management }
    }

    fn part_stats(speed: i32, cornering: i32, power_unit: i32, qualifying: i32) -> Stats {
        Stats { speed, cornering, power_unit, qualifying, pit_stop_time: 1.0, additional_stat_value: 0 }
    }

    #[test]
    fn driver_priorities_any_selected_false_when_all_false() {
        assert!(!DriverPriorities::default().any_selected());
    }

    #[test]
    fn driver_priorities_any_selected_true_for_each_field() {
        assert!(DriverPriorities { overtaking: true, ..Default::default() }.any_selected());
        assert!(DriverPriorities { defending: true, ..Default::default() }.any_selected());
        assert!(DriverPriorities { qualifying: true, ..Default::default() }.any_selected());
        assert!(DriverPriorities { race_start: true, ..Default::default() }.any_selected());
        assert!(DriverPriorities { tyre_management: true, ..Default::default() }.any_selected());
    }

    #[test]
    fn driver_priorities_labels_empty_when_none_selected() {
        assert!(DriverPriorities::default().labels().is_empty());
    }

    #[test]
    fn driver_priorities_labels_correct_order() {
        let p = DriverPriorities {
            overtaking: true, defending: true, qualifying: true,
            race_start: true, tyre_management: true,
        };
        assert_eq!(p.labels(), vec!["Overtaking", "Defending", "Qualifying", "Race Start", "Tyre Mgmt"]);
    }

    #[test]
    fn driver_score_no_priorities_returns_total_twice() {
        let stats = ds(10, 20, 30, 40, 50);
        let p = DriverPriorities::default();
        assert_eq!(p.score(&stats), (150, 150));
    }

    #[test]
    fn driver_score_single_priority_returns_that_stat_twice() {
        let stats = ds(10, 20, 30, 40, 50);
        let p = DriverPriorities { overtaking: true, ..Default::default() };
        assert_eq!(p.score(&stats), (10, 10));
    }

    #[test]
    fn driver_score_multiple_priorities_returns_min_and_sum() {
        let stats = ds(10, 20, 30, 40, 50);
        let p = DriverPriorities { overtaking: true, defending: true, ..Default::default() };
        assert_eq!(p.score(&stats), (10, 30));
    }

    #[test]
    fn driver_score_prefers_higher_min_first() {
        let high_min = ds(15, 15, 0, 0, 0);
        let low_min  = ds(10, 20, 0, 0, 0);
        let p = DriverPriorities { overtaking: true, defending: true, ..Default::default() };
        let s_high = p.score(&high_min);
        let s_low  = p.score(&low_min);
        assert!(s_high > s_low);
    }

    #[test]
    fn score_part_combo_no_priorities_returns_total_performance_twice() {
        let stats = part_stats(10, 20, 30, 40);
        let p = StatPriorities::default();
        assert_eq!(score_part_combo(&stats, &p), (100, 100));
    }

    #[test]
    fn score_part_combo_speed_only_returns_speed_twice() {
        let stats = part_stats(50, 20, 10, 5);
        let p = StatPriorities { speed: true, ..Default::default() };
        assert_eq!(score_part_combo(&stats, &p), (50, 50));
    }

    #[test]
    fn score_part_combo_multiple_priorities_returns_min_and_sum() {
        let stats = part_stats(100, 50, 0, 0);
        let p = StatPriorities { speed: true, cornering: true, ..Default::default() };
        assert_eq!(score_part_combo(&stats, &p), (50, 150));
    }

    #[test]
    fn score_part_combo_all_priorities_matches_total_when_equal_stats() {
        let stats = part_stats(25, 25, 25, 25);
        let p = StatPriorities { speed: true, cornering: true, power_unit: true, qualifying: true };
        assert_eq!(score_part_combo(&stats, &p), (25, 100));
    }
}

#[derive(Deserialize)]
pub struct SaveForm {
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
    pub driver1_id: Option<i32>,
    pub driver2_id: Option<i32>,
}

async fn save(State(state): State<AppState>, Form(form): Form<SaveForm>) -> impl IntoResponse {
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
