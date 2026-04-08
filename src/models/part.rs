use serde::{Deserialize, Serialize};

/// Car part categories in F1 Clash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "part_category", rename_all = "snake_case")]
pub enum PartCategory {
    Engine,
    FrontWing,
    RearWing,
    Suspension,
    Brakes,
    Gearbox,
}

impl PartCategory {
    pub fn all() -> &'static [PartCategory] {
        &[
            Self::Brakes,
            Self::Gearbox,
            Self::RearWing,
            Self::FrontWing,
            Self::Suspension,
            Self::Engine,
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
        }
    }
}

/// Stats that car parts contribute to
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
    pub drs: i32,
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
            drs: self.drs + other.drs,
        }
    }

    /// Apply a percentage boost to performance stats (pit_stop_time is reduced)
    pub fn boosted(&self, percentage: i32) -> Stats {
        let mult = percentage as f64 / 100.0;
        Stats {
            speed: self.speed + (self.speed as f64 * mult).round() as i32,
            cornering: self.cornering + (self.cornering as f64 * mult).round() as i32,
            power_unit: self.power_unit + (self.power_unit as f64 * mult).round() as i32,
            qualifying: self.qualifying + (self.qualifying as f64 * mult).round() as i32,
            pit_stop_time: self.pit_stop_time * (1.0 - mult),
            drs: self.drs,
        }
    }
}
