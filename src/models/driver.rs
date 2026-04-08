use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

    pub fn boosted(&self, percentage: i32) -> DriverStats {
        let mult = percentage as f64 / 100.0;
        DriverStats {
            overtaking: self.overtaking + (self.overtaking as f64 * mult).round() as i32,
            defending: self.defending + (self.defending as f64 * mult).round() as i32,
            qualifying: self.qualifying + (self.qualifying as f64 * mult).round() as i32,
            race_start: self.race_start + (self.race_start as f64 * mult).round() as i32,
            tyre_management: self.tyre_management + (self.tyre_management as f64 * mult).round() as i32,
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
}

/// A boost applied to a driver
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriverBoost {
    pub id: i32,
    pub driver_name: String,
    pub rarity: String,
    pub percentage: i32,
}
