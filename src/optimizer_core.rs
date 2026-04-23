/// Pure brute-force optimizer — no Axum or database dependencies.
/// Used by routes/optimizer.rs and benchmarks.
use crate::data::StatPriorities;
use crate::models::driver::{DriverInventoryItem, DriverStats};
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::InventoryItem;

// ── Pruning ───────────────────────────────────────────────────────────────────

/// Cap candidates per category so the brute-force stays tractable.
/// Always keeps the single best part for each of the 4 performance stats
/// (guaranteeing the optimal solution for any single-stat priority), then
/// fills remaining slots by total performance.
pub const MAX_PARTS_PER_CAT: usize = 10;

pub fn prune_category(mut parts: Vec<ResolvedPart>) -> Vec<ResolvedPart> {
    if parts.len() <= MAX_PARTS_PER_CAT {
        return parts;
    }
    let must: Vec<i32> = [
        parts.iter().max_by_key(|p| p.stats.speed),
        parts.iter().max_by_key(|p| p.stats.cornering),
        parts.iter().max_by_key(|p| p.stats.power_unit),
        parts.iter().max_by_key(|p| p.stats.qualifying),
    ]
    .into_iter()
    .flatten()
    .map(|p| p.item.id)
    .collect::<std::collections::HashSet<_>>()
    .into_iter()
    .collect();

    parts.sort_by(|a, b| {
        let a_must = must.contains(&a.item.id);
        let b_must = must.contains(&b.item.id);
        if a_must != b_must {
            return b_must.cmp(&a_must);
        }
        b.stats
            .total_performance()
            .cmp(&a.stats.total_performance())
    });
    parts.truncate(MAX_PARTS_PER_CAT);
    parts
}

pub struct ResolvedPart {
    pub item: InventoryItem,
    pub stats: Stats,
    pub rarity_css_class: &'static str,
}

pub struct ResolvedDriver {
    pub item: DriverInventoryItem,
    pub stats: DriverStats,
}

pub struct OptimizeResult {
    pub part_picks: Vec<(PartCategory, InventoryItem, Stats, &'static str)>,
    pub driver1: Option<(DriverInventoryItem, DriverStats)>,
    pub driver2: Option<(DriverInventoryItem, DriverStats)>,
    pub total_parts: Stats,
    pub total_drivers: DriverStats,
}

/// Run the brute-force optimizer with the given part and driver priorities.
/// Returns `None` if any category has no available parts.
pub fn run_brute_force(
    parts_per_cat: &[Vec<ResolvedPart>],
    categories: &[PartCategory],
    driver_pairs: &[(Option<usize>, Option<usize>)],
    resolved_drivers: &[ResolvedDriver],
    part_priorities: &StatPriorities,
    driver_priorities: &DriverPriorities,
) -> Option<OptimizeResult> {
    if parts_per_cat.iter().any(|c| c.is_empty()) {
        return None;
    }

    // The score tuple is (p_min, p_sum, d_min, d_sum) compared lexicographically.
    // Part score always dominates driver score, so we can optimise them independently:
    //   1. Find the best part combo  →  O(combos)
    //   2. Find the best driver pair →  O(driver_pairs)
    // This avoids the O(combos × driver_pairs) inner loop.

    // Step 1: find the best part combo
    let sizes: Vec<usize> = parts_per_cat.iter().map(|c| c.len()).collect();
    let total_part_combos: usize = sizes.iter().product();

    let mut best_part: Option<(Vec<usize>, (i32, i32))> = None;
    let mut part_indices = vec![0usize; categories.len()];

    for _ in 0..total_part_combos {
        let mut part_stats = Stats::default();
        for (cat_idx, &pi) in part_indices.iter().enumerate() {
            part_stats = part_stats.add(&parts_per_cat[cat_idx][pi].stats);
        }
        let score = score_part_combo(&part_stats, part_priorities);
        let is_better = best_part.as_ref().is_none_or(|(_, s)| score > *s);
        if is_better {
            best_part = Some((part_indices.clone(), score));
        }

        // Odometer increment
        let mut carry = true;
        for i in (0..part_indices.len()).rev() {
            if carry {
                part_indices[i] += 1;
                if part_indices[i] >= sizes[i] {
                    part_indices[i] = 0;
                } else {
                    carry = false;
                }
            }
        }
    }

    let (best_pidx, _) = best_part?;

    // Step 2: find the best driver pair (independent of part choice)
    let best_dp_idx = driver_pairs
        .iter()
        .enumerate()
        .max_by_key(|(_, (d1, d2))| {
            let mut ds = DriverStats::default();
            if let Some(i) = d1 {
                ds = ds.add(&resolved_drivers[*i].stats);
            }
            if let Some(i) = d2 {
                ds = ds.add(&resolved_drivers[*i].stats);
            }
            driver_priorities.score(&ds)
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    // Build result
    let picks: Vec<_> = {
        let mut temp: Vec<_> = best_pidx
        .iter()
        .enumerate()
        .map(|(ci, &pi)| {
            let rp = &parts_per_cat[ci][pi];
            (
                categories[ci],
                rp.item.clone(),
                rp.stats.clone(),
                rp.rarity_css_class,
            )
        })
        .collect();
        temp.sort_by_key(|d| d.0);
        temp
    };
    let (d1_idx, d2_idx) = driver_pairs[best_dp_idx];
    let d1 = d1_idx.map(|i| &resolved_drivers[i]);
    let d2 = d2_idx.map(|i| &resolved_drivers[i]);
    let ts = picks
        .iter()
        .fold(Stats::default(), |a, (_, _, s, _)| a.add(s));
    let mut ds = DriverStats::default();
    if let Some(d) = d1 {
        ds = ds.add(&d.stats);
    }
    if let Some(d) = d2 {
        ds = ds.add(&d.stats);
    }

    Some(OptimizeResult {
        part_picks: picks,
        driver1: d1.map(|d| (d.item.clone(), d.stats.clone())),
        driver2: d2.map(|d| (d.item.clone(), d.stats.clone())),
        total_parts: ts,
        total_drivers: ds,
    })
}

pub fn score_part_combo(stats: &Stats, priorities: &StatPriorities) -> (i32, i32) {
    if !priorities.any_selected() {
        let total = stats.total_performance();
        return (total, total);
    }
    let mut values = Vec::new();
    if priorities.speed {
        values.push(stats.speed);
    }
    if priorities.cornering {
        values.push(stats.cornering);
    }
    if priorities.power_unit {
        values.push(stats.power_unit);
    }
    if priorities.qualifying {
        values.push(stats.qualifying);
    }
    let min = *values.iter().min().unwrap();
    let sum: i32 = values.iter().sum();
    let sum = sum + stats.total_performance();
    (min, sum)
}

// ── Driver priorities ─────────────────────────────────────────────────────────

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
        self.overtaking
            || self.defending
            || self.qualifying
            || self.race_start
            || self.tyre_management
    }

    pub fn labels(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.overtaking {
            out.push("Overtaking");
        }
        if self.defending {
            out.push("Defending");
        }
        if self.qualifying {
            out.push("Qualifying");
        }
        if self.race_start {
            out.push("Race Start");
        }
        if self.tyre_management {
            out.push("Tyre Mgmt");
        }
        out
    }

    pub fn score(&self, stats: &DriverStats) -> (i32, i32) {
        if !self.any_selected() {
            let total = stats.total();
            return (total, total);
        }
        let mut values = Vec::new();
        if self.overtaking {
            values.push(stats.overtaking);
        }
        if self.defending {
            values.push(stats.defending);
        }
        if self.qualifying {
            values.push(stats.qualifying);
        }
        if self.race_start {
            values.push(stats.race_start);
        }
        if self.tyre_management {
            values.push(stats.tyre_management);
        }
        let min = *values.iter().min().unwrap();
        let sum: i32 = values.iter().sum();
        (min, sum)
    }
}
