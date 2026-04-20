use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::driver::DriverStats;
use super::part::Stats;

/// A saved car setup (part slots + 2 drivers). All part slots are nullable
/// (NULL means "use default placeholder" for that category).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setup {
    pub id: i32,
    pub name: String,
    pub engine_id: Option<i32>,
    pub front_wing_id: Option<i32>,
    pub rear_wing_id: Option<i32>,
    pub suspension_id: Option<i32>,
    pub brakes_id: Option<i32>,
    pub gearbox_id: Option<i32>,
    pub battery_id: Option<i32>,
    pub driver1_id: Option<i32>,
    pub driver2_id: Option<i32>,
}

/// A setup with its computed total stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupWithStats {
    pub setup: Setup,
    pub stats: Stats,
    pub driver_stats: DriverStats,
}

/// An inventory item — a part the player owns at a specific level
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: i32,
    pub part_name: String,
    pub level: i32,
    pub cards_owned: i32,
}

/// A global boost applied to a specific part
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Boost {
    pub id: i32,
    pub part_name: String,
    pub percentage: i32,
}
