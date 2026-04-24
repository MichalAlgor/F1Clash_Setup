/// Pure upgrade advisor — no Axum or database dependencies.
/// Simulates upgrading each owned part by one level (or to the reachable level
/// if the user has enough cards) and re-runs the optimizer to measure the
/// score improvement. Returns ranked recommendations split into two groups:
/// "immediate" (enough cards now) and "planned" (what to aim for).
use crate::data::{StatPriorities, calculate_upgrade, format_coins, max_level_for_rarity};
use crate::models::part::{OwnedPartDefinition, PartCategory, Stats};
use crate::models::setup::{Boost, InventoryItem};
use crate::optimizer_core::{ResolvedPart, prune_category, run_brute_force, score_part_combo};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UpgradeCandidate {
    pub inventory_id: i32,
    pub part_name: String,
    pub category: PartCategory,
    pub rarity_css_class: &'static str,
    pub current_level: i32,
    /// Level being simulated (reachable_level for immediate, current+1 for planned).
    pub target_level: i32,
}

#[derive(Debug, Clone)]
pub struct UpgradeCost {
    pub cards_owned: i32,
    /// Cards needed from current_level → target_level.
    pub cards_needed: i32,
    /// Coins needed from current_level → target_level.
    pub coins_needed: u64,
    pub can_afford: bool,
    /// Human-readable coin string.
    pub coins_display: String,
}

#[derive(Debug, Clone, Default)]
pub struct StatDelta {
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
}

#[derive(Debug, Clone)]
pub struct UpgradeRecommendation {
    pub candidate: UpgradeCandidate,
    pub cost: UpgradeCost,
    /// Improvement in the primary score metric (min of priority stats, or total).
    pub score_delta: i32,
    /// Per-stat change on the part itself (not the full setup).
    pub stat_delta: StatDelta,
}

pub struct AdvisorResult {
    pub priorities: StatPriorities,
    /// Baseline score with current inventory.
    pub baseline_score: i32,
    /// Parts the user has enough cards to upgrade now, ranked by score_delta desc.
    pub immediate: Vec<UpgradeRecommendation>,
    /// All +1 upgrades regardless of cards, ranked by score_delta desc.
    pub planned: Vec<UpgradeRecommendation>,
}

// ── Core algorithm ────────────────────────────────────────────────────────────

/// Run the upgrade advisor.
///
/// `parts_per_cat` and `categories` come from `resolve_parts()` in the optimizer route
/// (already resolved with boosts applied and pruned). We need the raw `inventory`,
/// `catalog`, and `boosts` to simulate individual part upgrades.
pub fn run_upgrade_advisor(
    parts_per_cat: &[Vec<ResolvedPart>],
    categories: &[PartCategory],
    catalog: &[OwnedPartDefinition],
    inventory: &[InventoryItem],
    boosts: &[Boost],
    priorities: &StatPriorities,
    season: &str,
) -> AdvisorResult {
    // ── Baseline ──────────────────────────────────────────────────────────────
    let no_drivers = vec![(None, None)];
    let baseline_result = run_brute_force(
        parts_per_cat,
        categories,
        &no_drivers,
        &[],
        priorities,
        &Default::default(),
    );
    let baseline_score = baseline_result
        .as_ref()
        .map(|r| score_part_combo(&r.total_parts, priorities).0)
        .unwrap_or(0);

    // ── Enumerate candidates ──────────────────────────────────────────────────
    let mut all_recs: Vec<UpgradeRecommendation> = Vec::new();

    for item in inventory {
        let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name) else {
            continue;
        };
        let max_lvl = max_level_for_rarity(&part_def.rarity);
        if item.level >= max_lvl {
            continue; // already maxed
        }

        // Find which category index this part belongs to
        let Some(cat_idx) = categories.iter().position(|c| *c == part_def.category) else {
            continue;
        };

        // "Immediate" target: highest reachable level with current cards
        let upgrade_info = calculate_upgrade(
            item.level,
            item.cards_owned,
            part_def.series,
            &part_def.rarity,
            season,
        );
        let immediate_target = upgrade_info.reachable_level;
        let can_afford = immediate_target > item.level;

        // "Planning" target: always current + 1
        let planning_target = item.level + 1;

        // For immediate: simulate at reachable_level (if different from planning).
        // For planning: always simulate +1.
        // We collect both, deduplicating where immediate == planning.
        let targets: Vec<(i32, bool)> = if can_afford && immediate_target != planning_target {
            vec![(immediate_target, true), (planning_target, false)]
        } else if can_afford {
            vec![(immediate_target, true)]
        } else {
            vec![(planning_target, false)]
        };

        for (target_level, _is_immediate_specific) in targets {
            let Some(target_stats) = part_def.stats_for_level(target_level) else {
                continue;
            };
            let Some(current_stats) = part_def.stats_for_level(item.level) else {
                continue;
            };

            // Build upgraded Stats (with boost applied if active)
            let mut upgraded_s = Stats {
                speed: target_stats.speed,
                cornering: target_stats.cornering,
                power_unit: target_stats.power_unit,
                qualifying: target_stats.qualifying,
                pit_stop_time: target_stats.pit_stop_time,
                additional_stat_value: target_stats.additional_stat_value,
            };
            if let Some(b) = boosts.iter().find(|b| b.part_name == item.part_name) {
                upgraded_s = upgraded_s.boosted(b.percentage);
            }

            // Stat delta (raw, pre-boost — shows what the upgrade does to the part)
            let stat_delta = StatDelta {
                speed: target_stats.speed - current_stats.speed,
                cornering: target_stats.cornering - current_stats.cornering,
                power_unit: target_stats.power_unit - current_stats.power_unit,
                qualifying: target_stats.qualifying - current_stats.qualifying,
                pit_stop_time: target_stats.pit_stop_time - current_stats.pit_stop_time,
            };

            // Build a modified parts_per_cat with the upgraded part substituted
            let mut modified: Vec<Vec<ResolvedPart>> = parts_per_cat
                .iter()
                .map(|cat| cat.iter().map(|rp| rp.clone_shallow()).collect())
                .collect();

            // Find the part in its category and replace stats
            if let Some(rp) = modified[cat_idx]
                .iter_mut()
                .find(|rp| rp.item.id == item.id)
            {
                rp.stats = upgraded_s.clone();
                rp.item.level = target_level;
            } else {
                // Part not in pruned pool — add it and re-prune
                modified[cat_idx].push(ResolvedPart {
                    item: InventoryItem {
                        id: item.id,
                        part_name: item.part_name.clone(),
                        level: target_level,
                        cards_owned: item.cards_owned,
                    },
                    stats: upgraded_s.clone(),
                    rarity_css_class: part_def.rarity_css_class(),
                });
            }
            // Re-prune after modification (upgraded part might change rankings)
            let repruned = prune_category(modified[cat_idx].drain(..).collect());
            modified[cat_idx] = repruned;

            // Run optimizer with modified pool
            let sim_result = run_brute_force(
                &modified,
                categories,
                &no_drivers,
                &[],
                priorities,
                &Default::default(),
            );
            let sim_score = sim_result
                .as_ref()
                .map(|r| score_part_combo(&r.total_parts, priorities).0)
                .unwrap_or(0);
            let score_delta = sim_score - baseline_score;

            // Compute upgrade cost for this specific target level
            let cost = compute_cost(item, part_def, target_level, season);

            all_recs.push(UpgradeRecommendation {
                candidate: UpgradeCandidate {
                    inventory_id: item.id,
                    part_name: item.part_name.clone(),
                    category: part_def.category,
                    rarity_css_class: part_def.rarity_css_class(),
                    current_level: item.level,
                    target_level,
                },
                cost,
                score_delta,
                stat_delta,
            });
        }
    }

    // ── Partition and sort ────────────────────────────────────────────────────
    // "immediate": can afford + simulate at reachable level
    let mut immediate: Vec<_> = all_recs
        .iter()
        .filter(|r| r.cost.can_afford)
        .cloned()
        .collect();
    immediate.sort_by(|a, b| b.score_delta.cmp(&a.score_delta));

    // "planned": all +1 upgrades (target == current + 1)
    let mut planned: Vec<_> = all_recs
        .into_iter()
        .filter(|r| r.candidate.target_level == r.candidate.current_level + 1)
        .collect();
    planned.sort_by(|a, b| b.score_delta.cmp(&a.score_delta));

    AdvisorResult {
        priorities: priorities.clone(),
        baseline_score,
        immediate,
        planned,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn compute_cost(
    item: &InventoryItem,
    part_def: &OwnedPartDefinition,
    target_level: i32,
    season: &str,
) -> UpgradeCost {
    // Cards needed from current → target (sum of per-level costs)
    let cards_needed: i32 = (item.level..target_level)
        .map(|lvl| {
            let idx = (lvl - 1) as usize;
            crate::data::CARD_COSTS.get(idx).copied().unwrap_or(0)
        })
        .sum();
    let can_afford = item.cards_owned >= cards_needed;

    let coins_to_target: u64 = compute_coin_cost(item.level, target_level, part_def.series, season);

    UpgradeCost {
        cards_owned: item.cards_owned,
        cards_needed,
        coins_needed: coins_to_target,
        can_afford,
        coins_display: format_coins(coins_to_target),
    }
}

fn compute_coin_cost(from_level: i32, to_level: i32, series: i32, season: &str) -> u64 {
    let coin_table = crate::data::coin_costs_for_season(season)
        .get((series - 1) as usize)
        .copied()
        .unwrap_or(&[]);
    (from_level..to_level)
        .map(|lvl| {
            let idx = (lvl - 1) as usize;
            coin_table.get(idx).copied().unwrap_or(0)
        })
        .sum()
}

// ── ResolvedPart shallow clone ────────────────────────────────────────────────

trait ShallowClone {
    fn clone_shallow(&self) -> Self;
}

impl ShallowClone for ResolvedPart {
    fn clone_shallow(&self) -> Self {
        ResolvedPart {
            item: self.item.clone(),
            stats: self.stats.clone(),
            rarity_css_class: self.rarity_css_class,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::StatPriorities;
    use crate::models::part::{PartCategory, Stats};
    use crate::models::setup::InventoryItem;
    use crate::optimizer_core::ResolvedPart;

    fn make_part(
        id: i32,
        name: &str,
        level: i32,
        cards: i32,
        speed: i32,
        total: i32,
    ) -> (InventoryItem, ResolvedPart) {
        let item = InventoryItem {
            id,
            part_name: name.to_string(),
            level,
            cards_owned: cards,
        };
        let stats = Stats {
            speed,
            cornering: total / 4,
            power_unit: total / 4,
            qualifying: total / 4,
            pit_stop_time: 0.5,
            additional_stat_value: 0,
        };
        let rp = ResolvedPart {
            item: item.clone(),
            stats,
            rarity_css_class: "rarity-common",
        };
        (item, rp)
    }

    fn make_catalog_part(name: &str, category: PartCategory, series: i32) -> OwnedPartDefinition {
        use crate::models::part::OwnedLevelStats;
        let levels: Vec<OwnedLevelStats> = (1..=9)
            .map(|l| OwnedLevelStats {
                level: l,
                speed: l * 5,
                cornering: l * 4,
                power_unit: l * 3,
                qualifying: l * 3,
                pit_stop_time: 1.0 - (l as f64 * 0.05),
                additional_stat_value: 0,
                additional_stat_details: Default::default(),
            })
            .collect();
        OwnedPartDefinition {
            id: 1,
            name: name.to_string(),
            season: "2026".to_string(),
            category,
            series,
            rarity: "Rare".to_string(),
            sort_order: 0,
            additional_stat_name: None,
            levels,
        }
    }

    #[test]
    fn maxed_part_produces_no_candidate() {
        let item = InventoryItem {
            id: 1,
            part_name: "Engine".to_string(),
            level: 9,
            cards_owned: 500,
        };
        let part_def = make_catalog_part("Engine", PartCategory::Engine, 1);
        // Rare max = 9
        let rp = ResolvedPart {
            item: item.clone(),
            stats: Stats {
                speed: 50,
                cornering: 40,
                power_unit: 30,
                qualifying: 30,
                pit_stop_time: 0.4,
                additional_stat_value: 0,
            },
            rarity_css_class: "rarity-rare",
        };

        // Single category, single part
        let parts_per_cat = vec![vec![rp]];
        let categories = vec![PartCategory::Engine];
        let catalog = vec![part_def];
        let inventory = vec![item];
        let priorities = StatPriorities::default();

        let result = run_upgrade_advisor(
            &parts_per_cat,
            &categories,
            &catalog,
            &inventory,
            &[],
            &priorities,
            "2025",
        );
        assert!(
            result.immediate.is_empty(),
            "maxed part should not appear in immediate"
        );
        assert!(
            result.planned.is_empty(),
            "maxed part should not appear in planned"
        );
    }

    #[test]
    fn upgrade_with_enough_cards_appears_in_immediate() {
        let (item, rp) = make_part(1, "Engine", 3, 500, 30, 60);
        let part_def = make_catalog_part("Engine", PartCategory::Engine, 1);
        let parts_per_cat = vec![vec![rp]];
        let categories = vec![PartCategory::Engine];

        let result = run_upgrade_advisor(
            &parts_per_cat,
            &categories,
            &[part_def],
            &[item],
            &[],
            &StatPriorities::default(),
            "2025",
        );
        // Should appear in immediate (500 cards >> cost for next levels)
        assert!(
            !result.immediate.is_empty(),
            "should have immediate upgrades"
        );
    }

    #[test]
    fn no_cards_part_not_in_immediate_but_in_planned() {
        let (mut item, rp) = make_part(1, "Engine", 3, 0, 30, 60);
        item.cards_owned = 0;
        let part_def = make_catalog_part("Engine", PartCategory::Engine, 1);
        let parts_per_cat = vec![vec![rp]];
        let categories = vec![PartCategory::Engine];

        let result = run_upgrade_advisor(
            &parts_per_cat,
            &categories,
            &[part_def],
            &[item],
            &[],
            &StatPriorities::default(),
            "2025",
        );
        assert!(
            result.immediate.is_empty(),
            "0 cards should not be immediate"
        );
        assert!(!result.planned.is_empty(), "should still appear in planned");
    }

    #[test]
    fn planned_target_is_always_current_plus_one() {
        let (item, rp) = make_part(1, "Engine", 3, 0, 30, 60);
        let part_def = make_catalog_part("Engine", PartCategory::Engine, 1);
        let parts_per_cat = vec![vec![rp]];
        let categories = vec![PartCategory::Engine];

        let result = run_upgrade_advisor(
            &parts_per_cat,
            &categories,
            &[part_def],
            &[item],
            &[],
            &StatPriorities::default(),
            "2025",
        );
        for rec in &result.planned {
            assert_eq!(
                rec.candidate.target_level,
                rec.candidate.current_level + 1,
                "planned target should always be current+1"
            );
        }
    }
}
