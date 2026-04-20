use crate::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;

use crate::auth::AuthStatus;
use crate::data::StatPriorities;
use crate::error::AppError;
use crate::get_session_season;
use crate::models::driver::{DriverBoost, DriverInventoryItem, DriverStats};
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem};
use crate::optimizer_core::{
    DriverPriorities, ResolvedDriver, ResolvedPart, prune_category, run_brute_force,
};
use crate::session::UserSession;
use crate::templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/optimizer", get(presets_form))
        .route("/optimizer/presets", get(run_presets))
        .route("/optimizer/custom", get(custom_form))
        .route("/optimizer/run", get(run))
        .route("/optimizer/save", axum::routing::post(save))
}

// ── Form handlers ─────────────────────────────────────────────────────────────

async fn presets_form(auth: AuthStatus) -> impl IntoResponse {
    templates::optimizer::presets_form_page(&auth)
}

async fn custom_form(auth: AuthStatus) -> impl IntoResponse {
    templates::optimizer::custom_form_page(&auth)
}

// ── Query structs ─────────────────────────────────────────────────────────────

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

#[derive(Deserialize)]
pub struct PresetsQuery {
    #[serde(default, deserialize_with = "deserialize_opt_i32")]
    pub max_part_series: Option<i32>,
}

fn deserialize_opt_i32<'de, D>(d: D) -> Result<Option<i32>, D::Error>
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

// ── Resolution helpers ────────────────────────────────────────────────────────

pub(crate) async fn resolve_parts(
    state: &AppState,
    season: &str,
    session_id: &str,
    max_part_series: i32,
) -> (Vec<Vec<ResolvedPart>>, Vec<PartCategory>) {
    #[cfg(debug_assertions)]
    let t = std::time::Instant::now();

    let catalog = state.catalog_for_season(season).await;
    #[cfg(debug_assertions)]
    eprintln!("[optimizer] catalog_for_season:    {:>8.2?}", t.elapsed());

    #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT * FROM inventory WHERE season = $1 AND session_id = $2 ORDER BY part_name",
    )
    .bind(season)
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] query inventory:        {:>8.2?}  ({} items)",
        t1.elapsed(),
        items.len()
    );

    #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();
    let boosts =
        sqlx::query_as::<_, Boost>("SELECT * FROM boosts WHERE season = $1 AND session_id = $2")
            .bind(season)
            .bind(session_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] query boosts:           {:>8.2?}  ({} boosts)",
        t1.elapsed(),
        boosts.len()
    );

    #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();
    let categories = state.categories_for_season(season).await;
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] categories_for_season: {:>8.2?}  ({} cats)",
        t1.elapsed(),
        categories.len()
    );

    #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();
    let parts_per_cat = categories
        .iter()
        .map(|cat| {
            let cat_parts: Vec<_> = items
                .iter()
                .filter_map(|item| {
                    let part_def = catalog.iter().find(|p| p.name == item.part_name)?;
                    if part_def.series > max_part_series {
                        return None;
                    }
                    if part_def.category != *cat {
                        return None;
                    }
                    let level_stats = part_def.stats_for_level(item.level)?;
                    let mut s = Stats {
                        speed: level_stats.speed,
                        cornering: level_stats.cornering,
                        power_unit: level_stats.power_unit,
                        qualifying: level_stats.qualifying,
                        pit_stop_time: level_stats.pit_stop_time,
                        additional_stat_value: level_stats.additional_stat_value,
                    };
                    if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                        s = s.boosted(b.percentage);
                    }
                    Some(ResolvedPart {
                        item: item.clone(),
                        stats: s,
                        rarity_css_class: part_def.rarity_css_class(),
                    })
                })
                .collect();
            #[cfg(debug_assertions)]
            eprintln!(
                "[optimizer] resolved parts before prune: {:>8.2?}  ({} cat_parts)",
                t1.elapsed(),
                cat_parts.len()
            );
            let mut cat_parts = prune_category(cat_parts);
            #[cfg(debug_assertions)]
            eprintln!(
                "[optimizer] resolved parts after prune: {:>8.2?}  ({} cat_parts)",
                t1.elapsed(),
                cat_parts.len()
            );
            // If no parts in this category, insert a zero placeholder so the
            // optimizer can still run. Stats are 1/1/1/1 with 1.0s pit stop.
            if cat_parts.is_empty() {
                cat_parts.push(ResolvedPart {
                    item: InventoryItem {
                        id: 0,
                        part_name: format!("(no {})", cat.display_name()),
                        level: 0,
                        cards_owned: 0,
                    },
                    stats: Stats {
                        speed: 1,
                        cornering: 1,
                        power_unit: 1,
                        qualifying: 1,
                        pit_stop_time: 1.0,
                        additional_stat_value: 0,
                    },
                    rarity_css_class: "secondary",
                });
            }
            cat_parts
        })
        .collect::<Vec<Vec<ResolvedPart>>>();
    #[cfg(debug_assertions)]
    {
        let counts: Vec<usize> = parts_per_cat.iter().map(|c| c.len()).collect();
        eprintln!(
            "[optimizer] resolve parts:          {:>8.2?}  combos: {}",
            t1.elapsed(),
            counts.iter().product::<usize>()
        );
    }

    (parts_per_cat, categories)
}

async fn resolve_drivers(
    state: &AppState,
    season: &str,
    session_id: &str,
    max_driver_series: i32,
) -> Vec<ResolvedDriver> {
    #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();
    let driver_items = sqlx::query_as::<_, DriverInventoryItem>(
        "SELECT * FROM driver_inventory WHERE season = $1 AND session_id = $2 ORDER BY driver_name",
    )
    .bind(season)
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] query driver_inventory: {:>8.2?}  ({} drivers)",
        t1.elapsed(),
        driver_items.len()
    );

    #[cfg(debug_assertions)]
    let t1 = std::time::Instant::now();
    let driver_boosts = sqlx::query_as::<_, DriverBoost>(
        "SELECT * FROM driver_boosts WHERE season = $1 AND session_id = $2",
    )
    .bind(season)
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] query driver_boosts:    {:>8.2?}  ({} boosts)",
        t1.elapsed(),
        driver_boosts.len()
    );

    let drivers_catalog = state.drivers_catalog_for_season(season).await;

    driver_items
        .iter()
        .filter_map(|item| {
            let def = drivers_catalog
                .iter()
                .find(|d| d.name == item.driver_name && d.rarity == item.rarity)?;
            let driver_series = def.series.parse::<i32>().unwrap_or(i32::MAX);
            if driver_series > max_driver_series {
                return None;
            }
            let ls = def.stats_for_level(item.level)?;
            let mut ds = ls.to_stats();
            if let Some(b) = driver_boosts
                .iter()
                .find(|b| b.driver_name == item.driver_name && b.rarity == item.rarity)
            {
                ds = ds.boosted(b.percentage);
            }
            Some(ResolvedDriver {
                item: item.clone(),
                stats: ds,
            })
        })
        .collect()
}

fn build_driver_pairs(resolved_drivers: &[ResolvedDriver]) -> Vec<(Option<usize>, Option<usize>)> {
    let mut pairs = vec![(None, None)];
    for i in 0..resolved_drivers.len() {
        pairs.push((Some(i), None));
        for j in (i + 1)..resolved_drivers.len() {
            pairs.push((Some(i), Some(j)));
        }
    }
    pairs
}

// ── Route handlers ────────────────────────────────────────────────────────────

async fn run_presets(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    axum::extract::Query(query): axum::extract::Query<PresetsQuery>,
    auth: AuthStatus,
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    let t_total = std::time::Instant::now();
    let season = get_session_season(&state.pool, &session_id).await;
    let max_part_series = query.max_part_series.unwrap_or(i32::MAX);
    #[cfg(debug_assertions)]
    eprintln!("[optimizer] === run_presets start (series ≤{max_part_series}) ===");

    #[cfg(debug_assertions)]
    let t = std::time::Instant::now();
    let (parts_per_cat, categories) =
        resolve_parts(&state, &season, &session_id, max_part_series).await;
    #[cfg(debug_assertions)]
    eprintln!("[optimizer] resolve_parts total:    {:>8.2?}", t.elapsed());

    // Presets optimise parts only — no drivers.
    let driver_pairs: Vec<(Option<usize>, Option<usize>)> = vec![(None, None)];
    let resolved_drivers: Vec<ResolvedDriver> = Vec::new();
    let driver_priorities = DriverPriorities::default();

    let preset_defs: [(&str, StatPriorities); 6] = [
        (
            "Speed",
            StatPriorities {
                speed: true,
                ..Default::default()
            },
        ),
        (
            "Speed + Qualifying",
            StatPriorities {
                speed: true,
                qualifying: true,
                ..Default::default()
            },
        ),
        (
            "Cornering",
            StatPriorities {
                cornering: true,
                ..Default::default()
            },
        ),
        (
            "Cornering + Qualifying",
            StatPriorities {
                cornering: true,
                qualifying: true,
                ..Default::default()
            },
        ),
        (
            "Power Unit",
            StatPriorities {
                power_unit: true,
                ..Default::default()
            },
        ),
        (
            "Power Unit + Qualifying",
            StatPriorities {
                power_unit: true,
                qualifying: true,
                ..Default::default()
            },
        ),
    ];

    #[cfg(debug_assertions)]
    let t_all = std::time::Instant::now();
    let presets: Vec<templates::optimizer::PresetResult> = preset_defs
        .into_iter()
        .map(|(label, prio)| {
            #[cfg(debug_assertions)]
            let t = std::time::Instant::now();
            let result = run_brute_force(
                &parts_per_cat,
                &categories,
                &driver_pairs,
                &resolved_drivers,
                &prio,
                &driver_priorities,
            );
            #[cfg(debug_assertions)]
            eprintln!(
                "[optimizer] brute_force [{label:<26}]: {:>8.2?}",
                t.elapsed()
            );
            templates::optimizer::PresetResult { label, result }
        })
        .collect();
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] all 6 brute_force:      {:>8.2?}",
        t_all.elapsed()
    );
    #[cfg(debug_assertions)]
    eprintln!(
        "[optimizer] === run_presets TOTAL:  {:>8.2?} ===",
        t_total.elapsed()
    );
    templates::optimizer::presets_result_page(&presets, &auth)
}

async fn run(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    axum::extract::Query(query): axum::extract::Query<OptimizerQuery>,
    auth: AuthStatus,
) -> impl IntoResponse {
    let season = get_session_season(&state.pool, &session_id).await;
    let max_part_series = query.max_part_series.unwrap_or(i32::MAX);
    let max_driver_series = query.max_driver_series.unwrap_or(i32::MAX);

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

    let (parts_per_cat, categories) =
        resolve_parts(&state, &season, &session_id, max_part_series).await;
    let resolved_drivers = resolve_drivers(&state, &season, &session_id, max_driver_series).await;
    let driver_pairs = build_driver_pairs(&resolved_drivers);

    match run_brute_force(
        &parts_per_cat,
        &categories,
        &driver_pairs,
        &resolved_drivers,
        &part_priorities,
        &driver_priorities,
    ) {
        Some(r) => templates::optimizer::result_page(
            &part_priorities,
            &driver_priorities,
            &r.part_picks,
            r.driver1.as_ref(),
            r.driver2.as_ref(),
            &r.total_parts,
            &r.total_drivers,
            &auth,
        ),
        None => templates::optimizer::result_page(
            &part_priorities,
            &driver_priorities,
            &[],
            None,
            None,
            &Stats::default(),
            &DriverStats::default(),
            &auth,
        ),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer_core::score_part_combo;

    fn ds(
        overtaking: i32,
        defending: i32,
        qualifying: i32,
        race_start: i32,
        tyre_management: i32,
    ) -> DriverStats {
        DriverStats {
            overtaking,
            defending,
            qualifying,
            race_start,
            tyre_management,
        }
    }

    fn part_stats(speed: i32, cornering: i32, power_unit: i32, qualifying: i32) -> Stats {
        Stats {
            speed,
            cornering,
            power_unit,
            qualifying,
            pit_stop_time: 1.0,
            additional_stat_value: 0,
        }
    }

    #[test]
    fn driver_priorities_any_selected_false_when_all_false() {
        assert!(!DriverPriorities::default().any_selected());
    }

    #[test]
    fn driver_priorities_any_selected_true_for_each_field() {
        assert!(
            DriverPriorities {
                overtaking: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            DriverPriorities {
                defending: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            DriverPriorities {
                qualifying: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            DriverPriorities {
                race_start: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            DriverPriorities {
                tyre_management: true,
                ..Default::default()
            }
            .any_selected()
        );
    }

    #[test]
    fn driver_priorities_labels_empty_when_none_selected() {
        assert!(DriverPriorities::default().labels().is_empty());
    }

    #[test]
    fn driver_priorities_labels_correct_order() {
        let p = DriverPriorities {
            overtaking: true,
            defending: true,
            qualifying: true,
            race_start: true,
            tyre_management: true,
        };
        assert_eq!(
            p.labels(),
            vec![
                "Overtaking",
                "Defending",
                "Qualifying",
                "Race Start",
                "Tyre Mgmt"
            ]
        );
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
        let p = DriverPriorities {
            overtaking: true,
            ..Default::default()
        };
        assert_eq!(p.score(&stats), (10, 10));
    }

    #[test]
    fn driver_score_multiple_priorities_returns_min_and_sum() {
        let stats = ds(10, 20, 30, 40, 50);
        let p = DriverPriorities {
            overtaking: true,
            defending: true,
            ..Default::default()
        };
        assert_eq!(p.score(&stats), (10, 30));
    }

    #[test]
    fn driver_score_prefers_higher_min_first() {
        let high_min = ds(15, 15, 0, 0, 0);
        let low_min = ds(10, 20, 0, 0, 0);
        let p = DriverPriorities {
            overtaking: true,
            defending: true,
            ..Default::default()
        };
        let s_high = p.score(&high_min);
        let s_low = p.score(&low_min);
        assert!(s_high > s_low);
    }

    #[test]
    fn score_part_combo_no_priorities_returns_total_performance_twice() {
        // pit=1.0 → round(7 + (7-1)*200/7) = 178; total = 100 + 178 = 278
        let stats = part_stats(10, 20, 30, 40);
        let p = StatPriorities::default();
        assert_eq!(score_part_combo(&stats, &p), (278, 278));
    }

    #[test]
    fn score_part_combo_speed_only_returns_speed_twice() {
        let stats = part_stats(50, 20, 10, 5);
        let p = StatPriorities {
            speed: true,
            ..Default::default()
        };
        assert_eq!(score_part_combo(&stats, &p), (50, 50));
    }

    #[test]
    fn score_part_combo_multiple_priorities_returns_min_and_sum() {
        let stats = part_stats(100, 50, 0, 0);
        let p = StatPriorities {
            speed: true,
            cornering: true,
            ..Default::default()
        };
        assert_eq!(score_part_combo(&stats, &p), (50, 150));
    }

    #[test]
    fn score_part_combo_all_priorities_matches_total_when_equal_stats() {
        let stats = part_stats(25, 25, 25, 25);
        let p = StatPriorities {
            speed: true,
            cornering: true,
            power_unit: true,
            qualifying: true,
        };
        assert_eq!(score_part_combo(&stats, &p), (25, 100));
    }
}

// ── Save ──────────────────────────────────────────────────────────────────────

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

async fn save(
    State(state): State<AppState>,
    UserSession(session_id): UserSession,
    Form(form): Form<SaveForm>,
) -> Result<impl IntoResponse, AppError> {
    let season = get_session_season(&state.pool, &session_id).await;
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
