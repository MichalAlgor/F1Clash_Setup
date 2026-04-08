use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Car part categories in F1 Clash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "part_category", rename_all = "snake_case")]
pub enum PartCategory {
    Engine,
    FrontWing,
    RearWing,
    Sidepod,
    Underbody,
    Suspension,
    Brakes,
}

impl PartCategory {
    pub fn all() -> &'static [PartCategory] {
        &[
            Self::Engine,
            Self::FrontWing,
            Self::RearWing,
            Self::Sidepod,
            Self::Underbody,
            Self::Suspension,
            Self::Brakes,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Engine => "Engine",
            Self::FrontWing => "Front Wing",
            Self::RearWing => "Rear Wing",
            Self::Sidepod => "Sidepod",
            Self::Underbody => "Underbody",
            Self::Suspension => "Suspension",
            Self::Brakes => "Brakes",
        }
    }
}

/// Stats that car parts contribute to
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromRow)]
pub struct Stats {
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
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
        }
    }
}

/// A car part the player owns
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Part {
    pub id: i32,
    pub name: String,
    pub category: PartCategory,
    pub level: i32,
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
}

impl Part {
    pub fn stats(&self) -> Stats {
        Stats {
            speed: self.speed,
            cornering: self.cornering,
            power_unit: self.power_unit,
            qualifying: self.qualifying,
            pit_stop_time: self.pit_stop_time,
        }
    }
}
