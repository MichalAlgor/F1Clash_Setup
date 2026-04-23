use std::collections::HashMap;

use crate::data::StatPriorities;
use serde::{Deserialize, Serialize};

/// Car part categories in F1 Clash
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "part_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PartCategory {
    FrontWing,
    Brakes,
    Suspension,
    RearWing,
    Gearbox,
    Engine,
    Battery,
}

impl PartCategory {
    /// All known categories — used as a fallback; prefer `AppState::categories_for_season()`.
    pub fn all() -> &'static [PartCategory] {
        &[
            Self::FrontWing,
            Self::Brakes,
            Self::Suspension,
            Self::RearWing,
            Self::Gearbox,
            Self::Engine,
            Self::Battery,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FrontWing => "Front Wing",
            Self::Brakes => "Brakes",
            Self::Suspension => "Suspension",
            Self::RearWing => "Rear Wing",
            Self::Gearbox => "Gearbox",
            Self::Engine => "Engine",
            Self::Battery => "Battery",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::FrontWing => "front_wing",
            Self::Brakes => "brakes",
            Self::Suspension => "suspension",
            Self::RearWing => "rear_wing",
            Self::Gearbox => "gearbox",
            Self::Engine => "engine",
            Self::Battery => "battery",
        }
    }
}

/// Stats that car parts contribute to a setup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
    /// Generic secondary stat (DRS, Overtake Mode, …). Not included in total_performance.
    pub additional_stat_value: i32,
}

impl Stats {
    pub fn total_performance(&self) -> i32 {
        let pit = (7.0 + (29.0 * (7.0 - self.pit_stop_time))).round() as i32;
        self.speed + self.cornering + self.power_unit + self.qualifying + pit
    }

    pub fn add(&self, other: &Stats) -> Stats {
        Stats {
            speed: self.speed + other.speed,
            cornering: self.cornering + other.cornering,
            power_unit: self.power_unit + other.power_unit,
            qualifying: self.qualifying + other.qualifying,
            pit_stop_time: self.pit_stop_time + other.pit_stop_time,
            additional_stat_value: self.additional_stat_value + other.additional_stat_value,
        }
    }

    /// Apply a percentage boost to performance stats (pit_stop_time is reduced).
    /// Integer stats use ceiling so any non-zero bonus always adds at least 1.
    /// The additional stat is left unchanged.
    pub fn boosted(&self, percentage: i32) -> Stats {
        let mult = percentage as f64 / 100.0;
        Stats {
            speed: self.speed + (self.speed as f64 * mult).ceil() as i32,
            cornering: self.cornering + (self.cornering as f64 * mult).ceil() as i32,
            power_unit: self.power_unit + (self.power_unit as f64 * mult).ceil() as i32,
            qualifying: self.qualifying + (self.qualifying as f64 * mult).ceil() as i32,
            pit_stop_time: ((self.pit_stop_time - 0.7 * mult) * 100.0).round() / 100.0,
            additional_stat_value: self.additional_stat_value,
        }
    }
}

/// Level stats for a single upgrade level — owned version loaded from DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedLevelStats {
    pub level: i32,
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
    /// The primary value of the part's special secondary stat (e.g. DRS, Overtake Mode).
    #[serde(default)]
    pub additional_stat_value: i32,
    /// Named sub-stats (e.g. {"Impact": 15, "Duration": 10, "Recharge Rate": 14}).
    #[serde(default)]
    pub additional_stat_details: HashMap<String, i32>,
}

impl OwnedLevelStats {
    pub fn priority_score(&self, priorities: &StatPriorities) -> i32 {
        let mut score = 0;
        if priorities.speed {
            score += self.speed;
        }
        if priorities.cornering {
            score += self.cornering;
        }
        if priorities.power_unit {
            score += self.power_unit;
        }
        if priorities.qualifying {
            score += self.qualifying;
        }
        score
    }

    pub fn total_performance(&self) -> i32 {
        let pit = (1.0 + (1.0 - self.pit_stop_time) * 200.0 / 7.0).round() as i32;
        self.speed + self.cornering + self.power_unit + self.qualifying + pit
    }
}

/// A part definition loaded from the DB, scoped to a season.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedPartDefinition {
    pub id: i32,
    pub name: String,
    pub season: String,
    pub category: PartCategory,
    pub series: i32,
    pub rarity: String, // "Common" | "Rare" | "Epic"
    pub sort_order: i32,
    /// The name of this part's secondary stat, if any (e.g. "DRS", "Overtake Mode").
    pub additional_stat_name: Option<String>,
    pub levels: Vec<OwnedLevelStats>,
}

impl OwnedPartDefinition {
    pub fn stats_for_level(&self, level: i32) -> Option<&OwnedLevelStats> {
        self.levels.iter().find(|l| l.level == level)
    }

    pub fn max_level(&self) -> i32 {
        self.levels.last().map_or(1, |l| l.level)
    }

    pub fn rarity_css_class(&self) -> &'static str {
        match self.rarity.as_str() {
            "Rare" => "rarity-rare",
            "Epic" => "rarity-epic",
            _ => "rarity-common",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(
        speed: i32,
        cornering: i32,
        power_unit: i32,
        qualifying: i32,
        pit_stop_time: f64,
        additional_stat_value: i32,
    ) -> Stats {
        Stats {
            speed,
            cornering,
            power_unit,
            qualifying,
            pit_stop_time,
            additional_stat_value,
        }
    }

    fn level_stats(
        level: i32,
        speed: i32,
        cornering: i32,
        power_unit: i32,
        qualifying: i32,
    ) -> OwnedLevelStats {
        OwnedLevelStats {
            level,
            speed,
            cornering,
            power_unit,
            qualifying,
            pit_stop_time: 1.0,
            additional_stat_value: 0,
            additional_stat_details: HashMap::new(),
        }
    }

    fn part_def(
        levels: Vec<OwnedLevelStats>,
        additional_stat_name: Option<&str>,
        rarity: &str,
    ) -> OwnedPartDefinition {
        OwnedPartDefinition {
            id: 1,
            name: "Test".to_string(),
            season: "2025".to_string(),
            category: PartCategory::Engine,
            series: 1,
            rarity: rarity.to_string(),
            sort_order: 0,
            additional_stat_name: additional_stat_name.map(|s| s.to_string()),
            levels,
        }
    }

    #[test]
    fn total_performance_sums_four_stats_and_pit_contribution() {
        // pit=1.0 → round(7 + 29*(7-1)) = 181
        let s = stats(10, 20, 30, 40, 1.0, 5);
        assert_eq!(s.total_performance(), 281);
    }

    #[test]
    fn total_performance_includes_pit_stop_excludes_additional_stat() {
        // pit=7.0 (baseline) → 7 + 29*0 = 7
        let s = stats(0, 0, 0, 0, 7.0, 999);
        assert_eq!(s.total_performance(), 7);
        // pit=0.0 (fastest possible) → 7 + 29*7 = 210
        let s_fast = stats(0, 0, 0, 0, 0.0, 0);
        assert_eq!(s_fast.total_performance(), 210);
    }

    #[test]
    fn add_combines_all_fields() {
        let a = stats(10, 20, 30, 40, 1.0, 5);
        let b = stats(1, 2, 3, 4, 0.5, 1);
        let c = a.add(&b);
        assert_eq!(c.speed, 11);
        assert_eq!(c.cornering, 22);
        assert_eq!(c.power_unit, 33);
        assert_eq!(c.qualifying, 44);
        assert!((c.pit_stop_time - 1.5).abs() < 1e-9);
        assert_eq!(c.additional_stat_value, 6);
    }

    #[test]
    fn boosted_zero_percent_is_identity() {
        let s = stats(100, 50, 30, 20, 2.0, 7);
        let b = s.boosted(0);
        assert_eq!(b.speed, 100);
        assert_eq!(b.cornering, 50);
        assert_eq!(b.power_unit, 30);
        assert_eq!(b.qualifying, 20);
        assert!((b.pit_stop_time - 2.0).abs() < 1e-9);
        assert_eq!(b.additional_stat_value, 7);
    }

    #[test]
    fn boosted_100_percent_doubles_performance_stats() {
        // pit_stop_time: 2.0 - 0.7*1.0 = 1.3 (fixed 0.7s reduction per 100%)
        let s = stats(50, 40, 30, 20, 2.0, 5);
        let b = s.boosted(100);
        assert_eq!(b.speed, 100);
        assert_eq!(b.cornering, 80);
        assert_eq!(b.power_unit, 60);
        assert_eq!(b.qualifying, 40);
        assert!((b.pit_stop_time - 1.3).abs() < 1e-9);
        assert_eq!(b.additional_stat_value, 5); // unchanged
    }

    #[test]
    fn boosted_10_percent_ceils_small_bonus_and_reduces_pit_by_0_07s() {
        // Integer stats: 3 * 10% = 0.3 → ceil = 1, so 3 + 1 = 4
        // Pit stop:      0.81 - 0.7 * 0.1 = 0.81 - 0.07 = 0.74
        let s = stats(3, 0, 0, 0, 0.81, 0);
        let b = s.boosted(10);
        assert_eq!(b.speed, 4);
        assert!((b.pit_stop_time - 0.74).abs() < 0.005);
    }

    #[test]
    fn boosted_50_percent_ceils_bonus() {
        let s = stats(3, 0, 0, 0, 1.0, 0);
        let b = s.boosted(50);
        assert_eq!(b.speed, 5); // 3 + ceil(1.5) = 3 + 2 = 5
    }

    #[test]
    fn boosted_additional_stat_is_always_unchanged() {
        let s = stats(100, 100, 100, 100, 1.0, 42);
        assert_eq!(s.boosted(25).additional_stat_value, 42);
        assert_eq!(s.boosted(100).additional_stat_value, 42);
    }

    // --- OwnedLevelStats ---

    #[test]
    fn owned_level_stats_total_performance_sums_four_stats_and_pit_contribution() {
        // pit=1.0 (baseline per part) → round(1 + 0) = 1
        let ls = level_stats(1, 10, 20, 30, 40);
        assert_eq!(ls.total_performance(), 101);
    }

    #[test]
    fn owned_level_stats_total_performance_includes_pit_stop_excludes_additional_stat() {
        // pit=1.0 baseline → contribution = 1; additional_stat excluded
        let mut ls = level_stats(1, 0, 0, 0, 0);
        ls.additional_stat_value = 999;
        assert_eq!(ls.total_performance(), 1);
    }

    #[test]
    fn owned_level_stats_priority_score_no_priorities() {
        let ls = level_stats(1, 10, 20, 30, 40);
        assert_eq!(ls.priority_score(&StatPriorities::default()), 0);
    }

    #[test]
    fn owned_level_stats_priority_score_all_priorities() {
        let ls = level_stats(1, 10, 20, 30, 40);
        let p = StatPriorities {
            speed: true,
            cornering: true,
            power_unit: true,
            qualifying: true,
        };
        // priority_score sums only the selected stats, not pit stop
        assert_eq!(ls.priority_score(&p), 100);
    }

    #[test]
    fn owned_level_stats_priority_score_partial() {
        let ls = level_stats(1, 10, 20, 30, 40);
        let p = StatPriorities {
            speed: true,
            qualifying: true,
            ..Default::default()
        };
        assert_eq!(ls.priority_score(&p), 50); // 10 + 40
    }

    // --- OwnedPartDefinition ---

    #[test]
    fn stats_for_level_found() {
        let part = part_def(
            vec![level_stats(1, 5, 5, 5, 5), level_stats(2, 10, 10, 10, 10)],
            None,
            "Common",
        );
        assert!(part.stats_for_level(1).is_some());
        assert_eq!(part.stats_for_level(1).unwrap().level, 1);
        assert_eq!(part.stats_for_level(2).unwrap().speed, 10);
    }

    #[test]
    fn stats_for_level_not_found() {
        let part = part_def(vec![level_stats(1, 5, 5, 5, 5)], None, "Common");
        assert!(part.stats_for_level(99).is_none());
    }

    #[test]
    fn max_level_returns_last() {
        let part = part_def(
            vec![
                level_stats(1, 0, 0, 0, 0),
                level_stats(5, 0, 0, 0, 0),
                level_stats(8, 0, 0, 0, 0),
            ],
            None,
            "Rare",
        );
        assert_eq!(part.max_level(), 8);
    }

    #[test]
    fn max_level_empty_returns_1() {
        let part = part_def(vec![], None, "Common");
        assert_eq!(part.max_level(), 1);
    }

    #[test]
    fn rarity_css_class_common() {
        assert_eq!(
            part_def(vec![], None, "Common").rarity_css_class(),
            "rarity-common"
        );
    }

    #[test]
    fn rarity_css_class_rare() {
        assert_eq!(
            part_def(vec![], None, "Rare").rarity_css_class(),
            "rarity-rare"
        );
    }

    #[test]
    fn rarity_css_class_epic() {
        assert_eq!(
            part_def(vec![], None, "Epic").rarity_css_class(),
            "rarity-epic"
        );
    }

    #[test]
    fn rarity_css_class_unknown_falls_back_to_common() {
        assert_eq!(
            part_def(vec![], None, "Legendary").rarity_css_class(),
            "rarity-common"
        );
    }

    #[test]
    fn additional_stat_name_is_none_by_default() {
        let part = part_def(vec![], None, "Common");
        assert!(part.additional_stat_name.is_none());
    }

    #[test]
    fn additional_stat_name_is_some_when_set() {
        let part = part_def(vec![], Some("Overtake Mode"), "Epic");
        assert_eq!(part.additional_stat_name.as_deref(), Some("Overtake Mode"));
    }

    // --- PartCategory::Battery ---

    #[test]
    fn battery_slug_is_battery() {
        assert_eq!(PartCategory::Battery.slug(), "battery");
    }

    #[test]
    fn battery_display_name_is_battery() {
        assert_eq!(PartCategory::Battery.display_name(), "Battery");
    }

    #[test]
    fn part_category_all_contains_all_variants() {
        let all = PartCategory::all();
        assert!(all.contains(&PartCategory::Engine));
        assert!(all.contains(&PartCategory::FrontWing));
        assert!(all.contains(&PartCategory::RearWing));
        assert!(all.contains(&PartCategory::Suspension));
        assert!(all.contains(&PartCategory::Brakes));
        assert!(all.contains(&PartCategory::Gearbox));
        assert!(all.contains(&PartCategory::Battery));
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn part_category_display_name_and_slug_are_consistent() {
        for cat in PartCategory::all() {
            assert!(!cat.display_name().is_empty());
            assert!(!cat.slug().is_empty());
        }
    }
}
