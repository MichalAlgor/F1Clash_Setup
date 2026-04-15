use std::collections::HashMap;

use crate::data::StatPriorities;
use serde::{Deserialize, Serialize};

/// Car part categories in F1 Clash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "part_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PartCategory {
    Engine,
    FrontWing,
    RearWing,
    Suspension,
    Brakes,
    Gearbox,
    Battery,
}

impl PartCategory {
    /// All known categories — used as a fallback; prefer `AppState::categories_for_season()`.
    pub fn all() -> &'static [PartCategory] {
        &[
            Self::Brakes,
            Self::Gearbox,
            Self::RearWing,
            Self::FrontWing,
            Self::Suspension,
            Self::Engine,
            Self::Battery,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Engine => "Engine",
            Self::FrontWing => "Front Wing",
            Self::RearWing => "Rear Wing",
            Self::Suspension => "Suspension",
            Self::Brakes => "Brakes",
            Self::Gearbox => "Gearbox",
            Self::Battery => "Battery",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::FrontWing => "front_wing",
            Self::RearWing => "rear_wing",
            Self::Suspension => "suspension",
            Self::Brakes => "brakes",
            Self::Gearbox => "gearbox",
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
        self.speed + self.cornering + self.power_unit + self.qualifying
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
    /// The additional stat is left unchanged.
    pub fn boosted(&self, percentage: i32) -> Stats {
        let mult = percentage as f64 / 100.0;
        Stats {
            speed: self.speed + (self.speed as f64 * mult).round() as i32,
            cornering: self.cornering + (self.cornering as f64 * mult).round() as i32,
            power_unit: self.power_unit + (self.power_unit as f64 * mult).round() as i32,
            qualifying: self.qualifying + (self.qualifying as f64 * mult).round() as i32,
            pit_stop_time: self.pit_stop_time * (1.0 - mult),
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
        if priorities.speed { score += self.speed; }
        if priorities.cornering { score += self.cornering; }
        if priorities.power_unit { score += self.power_unit; }
        if priorities.qualifying { score += self.qualifying; }
        score
    }

    pub fn total_performance(&self) -> i32 {
        self.speed + self.cornering + self.power_unit + self.qualifying
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

    fn stats(speed: i32, cornering: i32, power_unit: i32, qualifying: i32, pit_stop_time: f64, additional_stat_value: i32) -> Stats {
        Stats { speed, cornering, power_unit, qualifying, pit_stop_time, additional_stat_value }
    }

    #[test]
    fn total_performance_sums_four_stats() {
        let s = stats(10, 20, 30, 40, 1.0, 5);
        assert_eq!(s.total_performance(), 100);
    }

    #[test]
    fn total_performance_excludes_pit_stop_and_additional_stat() {
        let s = stats(0, 0, 0, 0, 99.9, 999);
        assert_eq!(s.total_performance(), 0);
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
        let s = stats(50, 40, 30, 20, 2.0, 5);
        let b = s.boosted(100);
        assert_eq!(b.speed, 100);
        assert_eq!(b.cornering, 80);
        assert_eq!(b.power_unit, 60);
        assert_eq!(b.qualifying, 40);
        assert!((b.pit_stop_time - 0.0).abs() < 1e-9);
        assert_eq!(b.additional_stat_value, 5); // unchanged
    }

    #[test]
    fn boosted_50_percent_rounds_half_away_from_zero() {
        let s = stats(3, 0, 0, 0, 1.0, 0);
        let b = s.boosted(50);
        assert_eq!(b.speed, 5); // 3 + round(1.5) = 3 + 2 = 5
    }

    #[test]
    fn boosted_additional_stat_is_always_unchanged() {
        let s = stats(100, 100, 100, 100, 1.0, 42);
        assert_eq!(s.boosted(25).additional_stat_value, 42);
        assert_eq!(s.boosted(100).additional_stat_value, 42);
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
