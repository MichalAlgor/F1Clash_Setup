use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// DB-backed driver definition (one per name+rarity+season).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedDriverDefinition {
    pub id: i32,
    pub name: String,
    pub season: String,
    pub rarity: String,
    pub series: String,
    pub sort_order: i32,
    pub levels: Vec<OwnedDriverLevelStats>,
}

impl OwnedDriverDefinition {
    pub fn stats_for_level(&self, level: i32) -> Option<&OwnedDriverLevelStats> {
        self.levels.iter().find(|l| l.level == level)
    }
    pub fn max_level(&self) -> i32 {
        self.levels.last().map_or(1, |l| l.level)
    }
}

/// Per-level stats for a driver definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedDriverLevelStats {
    pub level: i32,
    pub overtaking: i32,
    pub defending: i32,
    pub qualifying: i32,
    pub race_start: i32,
    pub tyre_management: i32,
    pub cards_required: i32,
    pub coins_cost: i64,
    pub legacy_points: i32,
}

impl OwnedDriverLevelStats {
    pub fn total(&self) -> i32 {
        self.overtaking + self.defending + self.qualifying + self.race_start + self.tyre_management
    }

    pub fn to_stats(&self) -> DriverStats {
        DriverStats {
            overtaking: self.overtaking,
            defending: self.defending,
            qualifying: self.qualifying,
            race_start: self.race_start,
            tyre_management: self.tyre_management,
        }
    }
}

/// Driver stats — separate from part stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriverStats {
    pub overtaking: i32,
    pub defending: i32,
    pub qualifying: i32,
    pub race_start: i32,
    pub tyre_management: i32,
}

impl DriverStats {
    pub fn total(&self) -> i32 {
        self.overtaking + self.defending + self.qualifying + self.race_start + self.tyre_management
    }

    pub fn add(&self, other: &DriverStats) -> DriverStats {
        DriverStats {
            overtaking: self.overtaking + other.overtaking,
            defending: self.defending + other.defending,
            qualifying: self.qualifying + other.qualifying,
            race_start: self.race_start + other.race_start,
            tyre_management: self.tyre_management + other.tyre_management,
        }
    }

    /// Integer stats use ceiling so any non-zero boost always adds at least 1.
    pub fn boosted(&self, percentage: i32) -> DriverStats {
        let mult = percentage as f64 / 100.0;
        DriverStats {
            overtaking: self.overtaking + (self.overtaking as f64 * mult).ceil() as i32,
            defending: self.defending + (self.defending as f64 * mult).ceil() as i32,
            qualifying: self.qualifying + (self.qualifying as f64 * mult).ceil() as i32,
            race_start: self.race_start + (self.race_start as f64 * mult).ceil() as i32,
            tyre_management: self.tyre_management
                + (self.tyre_management as f64 * mult).ceil() as i32,
        }
    }
}

/// A driver the player owns
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriverInventoryItem {
    pub id: i32,
    pub driver_name: String,
    pub rarity: String,
    pub level: i32,
    pub cards_owned: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn total_sums_all_five_stats() {
        assert_eq!(ds(10, 20, 30, 40, 50).total(), 150);
    }

    #[test]
    fn total_with_zeros() {
        assert_eq!(ds(0, 0, 0, 0, 0).total(), 0);
    }

    #[test]
    fn add_combines_all_fields() {
        let a = ds(10, 20, 30, 40, 50);
        let b = ds(1, 2, 3, 4, 5);
        let c = a.add(&b);
        assert_eq!(c.overtaking, 11);
        assert_eq!(c.defending, 22);
        assert_eq!(c.qualifying, 33);
        assert_eq!(c.race_start, 44);
        assert_eq!(c.tyre_management, 55);
    }

    #[test]
    fn boosted_zero_percent_is_identity() {
        let s = ds(30, 25, 20, 15, 10);
        let b = s.boosted(0);
        assert_eq!(b.overtaking, 30);
        assert_eq!(b.defending, 25);
        assert_eq!(b.qualifying, 20);
        assert_eq!(b.race_start, 15);
        assert_eq!(b.tyre_management, 10);
    }

    #[test]
    fn boosted_100_percent_doubles_all_stats() {
        let s = ds(10, 20, 30, 40, 50);
        let b = s.boosted(100);
        assert_eq!(b.overtaking, 20);
        assert_eq!(b.defending, 40);
        assert_eq!(b.qualifying, 60);
        assert_eq!(b.race_start, 80);
        assert_eq!(b.tyre_management, 100);
    }

    #[test]
    fn boosted_10_percent_ceils_small_bonus() {
        // 3 * 10% = 0.3 → ceil = 1, so 3 + 1 = 4 (not 3)
        let s = ds(3, 0, 0, 0, 0);
        let b = s.boosted(10);
        assert_eq!(b.overtaking, 4);
    }

    #[test]
    fn boosted_50_percent_ceils_bonus() {
        // 3 * 0.5 = 1.5 → ceil = 2, so 3 + 2 = 5
        let s = ds(3, 0, 0, 0, 0);
        let b = s.boosted(50);
        assert_eq!(b.overtaking, 5);
    }
}

/// A boost applied to a driver
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriverBoost {
    pub id: i32,
    pub driver_name: String,
    pub rarity: String,
    pub percentage: i32,
}
