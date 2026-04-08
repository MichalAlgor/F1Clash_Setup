use crate::models::part::PartCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
}

impl Rarity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Rare => "Rare",
            Self::Epic => "Epic",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Common => "rarity-common",
            Self::Rare => "rarity-rare",
            Self::Epic => "rarity-epic",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartDefinition {
    pub name: &'static str,
    pub category: PartCategory,
    pub series: i32,
    pub rarity: Rarity,
    pub levels: &'static [LevelStats],
}

#[derive(Debug, Clone, Copy)]
pub struct LevelStats {
    pub level: i32,
    pub speed: i32,
    pub cornering: i32,
    pub power_unit: i32,
    pub qualifying: i32,
    pub pit_stop_time: f64,
    pub drs: i32,
}

impl LevelStats {
    const fn new(level: i32, speed: i32, cornering: i32, power_unit: i32, qualifying: i32, pit_stop_time: f64, drs: i32) -> Self {
        Self { level, speed, cornering, power_unit, qualifying, pit_stop_time, drs }
    }
}

impl LevelStats {
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

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct StatPriorities {
    #[serde(default)]
    pub speed: bool,
    #[serde(default)]
    pub cornering: bool,
    #[serde(default)]
    pub power_unit: bool,
    #[serde(default)]
    pub qualifying: bool,
}

impl StatPriorities {
    pub fn any_selected(&self) -> bool {
        self.speed || self.cornering || self.power_unit || self.qualifying
    }

    pub fn labels(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.speed { out.push("Speed"); }
        if self.cornering { out.push("Cornering"); }
        if self.power_unit { out.push("Power Unit"); }
        if self.qualifying { out.push("Qualifying"); }
        out
    }
}

impl PartDefinition {
    pub fn stats_for_level(&self, level: i32) -> Option<&LevelStats> {
        self.levels.iter().find(|l| l.level == level)
    }

    pub fn max_level(&self) -> i32 {
        self.levels.last().map_or(1, |l| l.level)
    }
}

pub fn catalog() -> &'static [PartDefinition] {
    CATALOG
}

pub fn find_part(name: &str) -> Option<&'static PartDefinition> {
    CATALOG.iter().find(|p| p.name == name)
}

pub fn parts_by_category(category: PartCategory) -> Vec<&'static PartDefinition> {
    CATALOG.iter().filter(|p| p.category == category).collect()
}

use PartCategory::*;
use Rarity::*;

static CATALOG: &[PartDefinition] = &[
    // ==================== ENGINE ====================
    PartDefinition {
        name: "Mach I", category: Engine, series: 1, rarity: Common,
        levels: &[
            LevelStats::new(1, 2, 3, 1, 2, 0.97, 0),
            LevelStats::new(2, 3, 4, 2, 3, 0.94, 0),
            LevelStats::new(3, 4, 5, 3, 4, 0.91, 0),
            LevelStats::new(4, 5, 6, 3, 5, 0.88, 0),
            LevelStats::new(5, 6, 7, 4, 6, 0.85, 0),
            LevelStats::new(6, 7, 8, 5, 7, 0.83, 0),
            LevelStats::new(7, 7, 8, 6, 7, 0.80, 0),
            LevelStats::new(8, 8, 9, 7, 8, 0.77, 0),
            LevelStats::new(9, 9, 10, 7, 9, 0.74, 0),
            LevelStats::new(10, 10, 11, 8, 10, 0.71, 0),
            LevelStats::new(11, 11, 12, 9, 11, 0.69, 0),
        ],
    },
    PartDefinition {
        name: "Spark-E", category: Engine, series: 2, rarity: Common,
        levels: &[
            LevelStats::new(1, 6, 2, 1, 3, 0.90, 0),
            LevelStats::new(2, 7, 3, 2, 4, 0.85, 0),
            LevelStats::new(3, 8, 3, 2, 4, 0.81, 0),
            LevelStats::new(4, 10, 4, 3, 5, 0.77, 0),
            LevelStats::new(5, 11, 4, 3, 6, 0.73, 0),
            LevelStats::new(6, 12, 5, 4, 7, 0.69, 0),
            LevelStats::new(7, 13, 6, 5, 7, 0.64, 0),
            LevelStats::new(8, 14, 6, 5, 8, 0.60, 0),
            LevelStats::new(9, 16, 7, 6, 9, 0.56, 0),
            LevelStats::new(10, 17, 7, 6, 9, 0.52, 0),
            LevelStats::new(11, 18, 8, 7, 10, 0.48, 0),
        ],
    },
    PartDefinition {
        name: "Mach II", category: Engine, series: 4, rarity: Rare,
        levels: &[
            LevelStats::new(1, 8, 16, 6, 16, 0.79, 0),
            LevelStats::new(2, 9, 18, 7, 18, 0.77, 0),
            LevelStats::new(3, 10, 20, 8, 19, 0.75, 0),
            LevelStats::new(4, 10, 21, 8, 21, 0.72, 0),
            LevelStats::new(5, 11, 23, 9, 23, 0.70, 0),
            LevelStats::new(6, 12, 25, 10, 24, 0.68, 0),
            LevelStats::new(7, 13, 27, 11, 26, 0.66, 0),
            LevelStats::new(8, 13, 28, 11, 27, 0.64, 0),
            LevelStats::new(9, 14, 30, 12, 29, 0.62, 0),
        ],
    },
    PartDefinition {
        name: "The Reactor", category: Engine, series: 6, rarity: Common,
        levels: &[
            LevelStats::new(1, 5, 4, 12, 6, 0.86, 0),
            LevelStats::new(2, 6, 5, 14, 7, 0.84, 0),
            LevelStats::new(3, 6, 5, 16, 7, 0.81, 0),
            LevelStats::new(4, 7, 6, 18, 8, 0.79, 0),
            LevelStats::new(5, 7, 6, 20, 8, 0.76, 0),
            LevelStats::new(6, 8, 7, 22, 9, 0.74, 0),
            LevelStats::new(7, 9, 7, 23, 10, 0.71, 0),
            LevelStats::new(8, 9, 8, 25, 10, 0.69, 0),
            LevelStats::new(9, 10, 8, 27, 11, 0.66, 0),
            LevelStats::new(10, 10, 9, 29, 11, 0.64, 0),
            LevelStats::new(11, 11, 9, 31, 12, 0.62, 0),
        ],
    },
    PartDefinition {
        name: "Mach III", category: Engine, series: 7, rarity: Epic,
        levels: &[
            LevelStats::new(1, 14, 36, 13, 15, 0.55, 0),
            LevelStats::new(2, 15, 39, 14, 16, 0.51, 0),
            LevelStats::new(3, 16, 41, 15, 17, 0.48, 0),
            LevelStats::new(4, 17, 44, 16, 18, 0.44, 0),
            LevelStats::new(5, 17, 46, 16, 18, 0.41, 0),
            LevelStats::new(6, 18, 49, 17, 19, 0.37, 0),
            LevelStats::new(7, 19, 51, 18, 20, 0.34, 0),
            LevelStats::new(8, 20, 54, 19, 21, 0.30, 0),
        ],
    },
    PartDefinition {
        name: "Behemoth", category: Engine, series: 10, rarity: Rare,
        levels: &[
            LevelStats::new(1, 24, 8, 10, 9, 0.72, 0),
            LevelStats::new(2, 26, 9, 11, 10, 0.69, 0),
            LevelStats::new(3, 29, 10, 12, 11, 0.65, 0),
            LevelStats::new(4, 31, 10, 12, 11, 0.62, 0),
            LevelStats::new(5, 34, 11, 13, 12, 0.58, 0),
            LevelStats::new(6, 36, 12, 14, 13, 0.55, 0),
            LevelStats::new(7, 38, 13, 15, 14, 0.51, 0),
            LevelStats::new(8, 41, 13, 15, 14, 0.48, 0),
            LevelStats::new(9, 43, 14, 16, 15, 0.44, 0),
        ],
    },
    PartDefinition {
        name: "Chaos Core", category: Engine, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 48, 19, 19, 21, 0.54, 0),
            LevelStats::new(2, 50, 20, 20, 23, 0.52, 0),
            LevelStats::new(3, 52, 21, 21, 24, 0.50, 0),
            LevelStats::new(4, 54, 22, 22, 25, 0.48, 0),
            LevelStats::new(5, 56, 23, 23, 26, 0.46, 0),
            LevelStats::new(6, 58, 25, 25, 27, 0.44, 0),
            LevelStats::new(7, 60, 26, 26, 28, 0.42, 0),
            LevelStats::new(8, 62, 28, 28, 29, 0.40, 0),
        ],
    },
    PartDefinition {
        name: "Turbo Jet", category: Engine, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 16, 15, 42, 17, 0.48, 0),
            LevelStats::new(2, 17, 16, 45, 18, 0.45, 0),
            LevelStats::new(3, 18, 17, 47, 19, 0.43, 0),
            LevelStats::new(4, 19, 18, 50, 20, 0.40, 0),
            LevelStats::new(5, 20, 18, 52, 22, 0.38, 0),
            LevelStats::new(6, 21, 19, 55, 23, 0.35, 0),
            LevelStats::new(7, 22, 20, 57, 24, 0.33, 0),
            LevelStats::new(8, 23, 21, 60, 25, 0.30, 0),
        ],
    },

    // ==================== FRONT WING ====================
    PartDefinition {
        name: "The Dash", category: FrontWing, series: 1, rarity: Common,
        levels: &[
            LevelStats::new(1, 2, 2, 3, 1, 0.97, 0),
            LevelStats::new(2, 3, 3, 4, 2, 0.94, 0),
            LevelStats::new(3, 4, 4, 5, 3, 0.92, 0),
            LevelStats::new(4, 5, 4, 6, 4, 0.89, 0),
            LevelStats::new(5, 6, 5, 7, 5, 0.87, 0),
            LevelStats::new(6, 7, 6, 8, 6, 0.84, 0),
            LevelStats::new(7, 7, 7, 8, 7, 0.82, 0),
            LevelStats::new(8, 8, 8, 9, 8, 0.79, 0),
            LevelStats::new(9, 9, 8, 10, 9, 0.77, 0),
            LevelStats::new(10, 10, 9, 11, 10, 0.74, 0),
            LevelStats::new(11, 11, 10, 12, 11, 0.72, 0),
        ],
    },
    PartDefinition {
        name: "Glide", category: FrontWing, series: 2, rarity: Common,
        levels: &[
            LevelStats::new(1, 5, 2, 2, 4, 0.97, 0),
            LevelStats::new(2, 6, 3, 3, 5, 0.94, 0),
            LevelStats::new(3, 8, 3, 3, 6, 0.92, 0),
            LevelStats::new(4, 9, 4, 4, 8, 0.89, 0),
            LevelStats::new(5, 10, 4, 4, 9, 0.87, 0),
            LevelStats::new(6, 12, 5, 5, 10, 0.84, 0),
            LevelStats::new(7, 13, 6, 6, 11, 0.82, 0),
            LevelStats::new(8, 14, 6, 6, 12, 0.79, 0),
            LevelStats::new(9, 15, 7, 7, 14, 0.77, 0),
            LevelStats::new(10, 17, 7, 7, 15, 0.74, 0),
            LevelStats::new(11, 18, 8, 8, 16, 0.72, 0),
        ],
    },
    PartDefinition {
        name: "Synergy", category: FrontWing, series: 4, rarity: Common,
        levels: &[
            LevelStats::new(1, 4, 3, 7, 2, 0.79, 0),
            LevelStats::new(2, 5, 4, 8, 3, 0.74, 0),
            LevelStats::new(3, 5, 4, 10, 3, 0.69, 0),
            LevelStats::new(4, 6, 5, 11, 4, 0.64, 0),
            LevelStats::new(5, 6, 5, 13, 4, 0.59, 0),
            LevelStats::new(6, 7, 6, 14, 5, 0.55, 0),
            LevelStats::new(7, 8, 7, 15, 5, 0.50, 0),
            LevelStats::new(8, 8, 7, 17, 6, 0.45, 0),
            LevelStats::new(9, 9, 8, 18, 6, 0.40, 0),
            LevelStats::new(10, 9, 8, 20, 7, 0.35, 0),
            LevelStats::new(11, 10, 9, 21, 7, 0.30, 0),
        ],
    },
    PartDefinition {
        name: "Vortex", category: FrontWing, series: 6, rarity: Epic,
        levels: &[
            LevelStats::new(1, 13, 35, 14, 15, 0.58, 0),
            LevelStats::new(2, 14, 38, 15, 16, 0.55, 0),
            LevelStats::new(3, 15, 40, 16, 17, 0.52, 0),
            LevelStats::new(4, 16, 43, 17, 18, 0.49, 0),
            LevelStats::new(5, 16, 46, 18, 19, 0.46, 0),
            LevelStats::new(6, 17, 49, 19, 20, 0.43, 0),
            LevelStats::new(7, 18, 51, 20, 21, 0.40, 0),
            LevelStats::new(8, 19, 54, 21, 22, 0.37, 0),
        ],
    },
    PartDefinition {
        name: "The Sabre", category: FrontWing, series: 8, rarity: Rare,
        levels: &[
            LevelStats::new(1, 15, 13, 5, 6, 0.79, 0),
            LevelStats::new(2, 17, 15, 6, 7, 0.77, 0),
            LevelStats::new(3, 19, 16, 7, 7, 0.75, 0),
            LevelStats::new(4, 20, 18, 7, 8, 0.72, 0),
            LevelStats::new(5, 22, 20, 8, 9, 0.70, 0),
            LevelStats::new(6, 24, 21, 9, 9, 0.68, 0),
            LevelStats::new(7, 26, 23, 10, 10, 0.66, 0),
            LevelStats::new(8, 27, 24, 10, 10, 0.64, 0),
            LevelStats::new(9, 29, 26, 11, 11, 0.62, 0),
        ],
    },
    PartDefinition {
        name: "Curler", category: FrontWing, series: 9, rarity: Rare,
        levels: &[
            LevelStats::new(1, 9, 8, 24, 10, 0.72, 0),
            LevelStats::new(2, 10, 9, 26, 11, 0.69, 0),
            LevelStats::new(3, 11, 10, 29, 12, 0.67, 0),
            LevelStats::new(4, 11, 10, 31, 13, 0.64, 0),
            LevelStats::new(5, 12, 11, 34, 14, 0.62, 0),
            LevelStats::new(6, 13, 12, 36, 15, 0.59, 0),
            LevelStats::new(7, 14, 13, 38, 16, 0.56, 0),
            LevelStats::new(8, 14, 13, 41, 17, 0.54, 0),
            LevelStats::new(9, 15, 14, 43, 18, 0.51, 0),
        ],
    },
    PartDefinition {
        name: "Flex XL", category: FrontWing, series: 11, rarity: Epic,
        levels: &[
            LevelStats::new(1, 17, 42, 15, 16, 0.48, 0),
            LevelStats::new(2, 18, 45, 16, 17, 0.45, 0),
            LevelStats::new(3, 19, 47, 17, 18, 0.43, 0),
            LevelStats::new(4, 20, 50, 18, 19, 0.40, 0),
            LevelStats::new(5, 22, 52, 18, 20, 0.38, 0),
            LevelStats::new(6, 23, 55, 19, 21, 0.35, 0),
            LevelStats::new(7, 24, 57, 20, 22, 0.33, 0),
            LevelStats::new(8, 25, 60, 21, 23, 0.30, 0),
        ],
    },
    PartDefinition {
        name: "Edgecutter", category: FrontWing, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 46, 20, 19, 17, 0.51, 0),
            LevelStats::new(2, 48, 21, 20, 18, 0.49, 0),
            LevelStats::new(3, 50, 22, 21, 19, 0.47, 0),
            LevelStats::new(4, 52, 23, 22, 20, 0.45, 0),
            LevelStats::new(5, 55, 24, 23, 22, 0.43, 0),
            LevelStats::new(6, 57, 26, 25, 23, 0.41, 0),
            LevelStats::new(7, 59, 28, 26, 25, 0.39, 0),
            LevelStats::new(8, 61, 30, 28, 27, 0.37, 0),
        ],
    },

    // ==================== REAR WING ====================
    PartDefinition {
        name: "Motion", category: RearWing, series: 1, rarity: Common,
        levels: &[
            LevelStats::new(1, 2, 3, 2, 1, 0.79, 0),
            LevelStats::new(2, 3, 4, 3, 2, 0.76, 0),
            LevelStats::new(3, 4, 5, 4, 3, 0.73, 0),
            LevelStats::new(4, 4, 6, 5, 3, 0.70, 0),
            LevelStats::new(5, 5, 7, 6, 4, 0.66, 0),
            LevelStats::new(6, 6, 8, 7, 5, 0.63, 0),
            LevelStats::new(7, 7, 8, 7, 6, 0.60, 0),
            LevelStats::new(8, 8, 9, 8, 7, 0.57, 0),
            LevelStats::new(9, 8, 10, 9, 7, 0.54, 0),
            LevelStats::new(10, 9, 11, 10, 8, 0.51, 0),
            LevelStats::new(11, 10, 12, 11, 9, 0.48, 0),
        ],
    },
    PartDefinition {
        name: "Gale Force", category: RearWing, series: 3, rarity: Common,
        levels: &[
            LevelStats::new(1, 4, 8, 2, 2, 0.65, 0),
            LevelStats::new(2, 5, 9, 3, 3, 0.61, 0),
            LevelStats::new(3, 5, 11, 3, 3, 0.57, 0),
            LevelStats::new(4, 6, 12, 4, 4, 0.53, 0),
            LevelStats::new(5, 7, 14, 4, 4, 0.50, 0),
            LevelStats::new(6, 8, 15, 5, 5, 0.46, 0),
            LevelStats::new(7, 8, 16, 5, 5, 0.42, 0),
            LevelStats::new(8, 9, 18, 6, 6, 0.38, 0),
            LevelStats::new(9, 10, 19, 6, 6, 0.34, 0),
            LevelStats::new(10, 10, 21, 7, 7, 0.30, 0),
            LevelStats::new(11, 11, 22, 7, 7, 0.27, 0),
        ],
    },
    PartDefinition {
        name: "X-Hale", category: RearWing, series: 4, rarity: Epic,
        levels: &[
            LevelStats::new(1, 17, 16, 7, 5, 0.72, 27),
            LevelStats::new(2, 19, 18, 8, 6, 0.69, 30),
            LevelStats::new(3, 21, 19, 9, 7, 0.65, 32),
            LevelStats::new(4, 23, 21, 10, 8, 0.62, 35),
            LevelStats::new(5, 24, 23, 10, 8, 0.58, 37),
            LevelStats::new(6, 26, 25, 11, 9, 0.55, 40),
            LevelStats::new(7, 28, 26, 12, 10, 0.51, 42),
            LevelStats::new(8, 30, 28, 13, 11, 0.48, 45),
        ],
    },
    PartDefinition {
        name: "The Spire", category: RearWing, series: 6, rarity: Rare,
        levels: &[
            LevelStats::new(1, 6, 5, 15, 14, 0.79, 0),
            LevelStats::new(2, 7, 5, 17, 15, 0.77, 0),
            LevelStats::new(3, 7, 6, 19, 17, 0.75, 0),
            LevelStats::new(4, 8, 6, 20, 18, 0.73, 0),
            LevelStats::new(5, 8, 7, 22, 20, 0.71, 0),
            LevelStats::new(6, 9, 7, 24, 21, 0.69, 0),
            LevelStats::new(7, 9, 7, 26, 22, 0.66, 0),
            LevelStats::new(8, 10, 8, 27, 24, 0.64, 0),
            LevelStats::new(9, 10, 8, 29, 25, 0.62, 0),
        ],
    },
    PartDefinition {
        name: "Power Lift", category: RearWing, series: 7, rarity: Rare,
        levels: &[
            LevelStats::new(1, 30, 12, 10, 12, 0.69, 0),
            LevelStats::new(2, 32, 13, 11, 13, 0.65, 0),
            LevelStats::new(3, 35, 14, 12, 14, 0.62, 0),
            LevelStats::new(4, 37, 15, 13, 15, 0.59, 0),
            LevelStats::new(5, 39, 16, 14, 16, 0.56, 0),
            LevelStats::new(6, 41, 16, 14, 16, 0.53, 0),
            LevelStats::new(7, 44, 17, 15, 17, 0.50, 0),
            LevelStats::new(8, 46, 18, 16, 18, 0.47, 0),
            LevelStats::new(9, 48, 19, 17, 19, 0.44, 0),
        ],
    },
    PartDefinition {
        name: "Aero Blade", category: RearWing, series: 9, rarity: Rare,
        levels: &[
            LevelStats::new(1, 10, 12, 33, 14, 0.97, 42),
            LevelStats::new(2, 11, 13, 36, 15, 0.94, 45),
            LevelStats::new(3, 12, 14, 38, 16, 0.91, 47),
            LevelStats::new(4, 13, 15, 41, 16, 0.89, 50),
            LevelStats::new(5, 14, 16, 43, 17, 0.86, 52),
            LevelStats::new(6, 14, 16, 46, 18, 0.83, 54),
            LevelStats::new(7, 15, 17, 48, 19, 0.81, 56),
            LevelStats::new(8, 16, 18, 51, 19, 0.78, 58),
            LevelStats::new(9, 17, 19, 53, 20, 0.76, 60),
        ],
    },
    PartDefinition {
        name: "The Valkyrie", category: RearWing, series: 11, rarity: Epic,
        levels: &[
            LevelStats::new(1, 42, 16, 15, 17, 0.48, 17),
            LevelStats::new(2, 45, 17, 16, 18, 0.45, 19),
            LevelStats::new(3, 47, 18, 17, 19, 0.42, 21),
            LevelStats::new(4, 50, 19, 18, 20, 0.39, 23),
            LevelStats::new(5, 52, 20, 18, 21, 0.36, 24),
            LevelStats::new(6, 55, 21, 19, 22, 0.33, 26),
            LevelStats::new(7, 57, 22, 20, 23, 0.30, 28),
            LevelStats::new(8, 60, 23, 21, 24, 0.27, 30),
        ],
    },
    PartDefinition {
        name: "Phantom Arc", category: RearWing, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 11, 46, 12, 13, 0.94, 46),
            LevelStats::new(2, 12, 48, 13, 14, 0.94, 48),
            LevelStats::new(3, 13, 50, 14, 15, 0.93, 50),
            LevelStats::new(4, 14, 52, 15, 16, 0.93, 52),
            LevelStats::new(5, 14, 54, 15, 16, 0.92, 54),
            LevelStats::new(6, 15, 56, 16, 17, 0.92, 56),
            LevelStats::new(7, 16, 58, 17, 18, 0.91, 58),
            LevelStats::new(8, 17, 60, 18, 19, 0.90, 60),
        ],
    },

    // ==================== SUSPENSION ====================
    PartDefinition {
        name: "Equinox", category: Suspension, series: 1, rarity: Epic,
        levels: &[
            LevelStats::new(1, 4, 2, 9, 3, 0.93, 0),
            LevelStats::new(2, 5, 3, 12, 4, 0.90, 0),
            LevelStats::new(3, 6, 4, 14, 5, 0.86, 0),
            LevelStats::new(4, 7, 5, 17, 6, 0.83, 0),
            LevelStats::new(5, 9, 6, 19, 7, 0.79, 0),
            LevelStats::new(6, 10, 7, 22, 8, 0.76, 0),
            LevelStats::new(7, 11, 8, 24, 9, 0.72, 0),
            LevelStats::new(8, 12, 9, 27, 10, 0.69, 0),
        ],
    },
    PartDefinition {
        name: "Swish", category: Suspension, series: 2, rarity: Common,
        levels: &[
            LevelStats::new(1, 3, 2, 4, 3, 0.90, 0),
            LevelStats::new(2, 4, 3, 5, 4, 0.86, 0),
            LevelStats::new(3, 5, 4, 6, 5, 0.83, 0),
            LevelStats::new(4, 5, 4, 7, 5, 0.79, 0),
            LevelStats::new(5, 6, 5, 8, 6, 0.76, 0),
            LevelStats::new(6, 7, 6, 9, 7, 0.72, 0),
            LevelStats::new(7, 8, 7, 10, 8, 0.69, 0),
            LevelStats::new(8, 9, 8, 11, 9, 0.65, 0),
            LevelStats::new(9, 9, 8, 12, 9, 0.62, 0),
            LevelStats::new(10, 10, 9, 13, 10, 0.58, 0),
            LevelStats::new(11, 11, 10, 14, 11, 0.55, 0),
        ],
    },
    PartDefinition {
        name: "Curver 2.5", category: Suspension, series: 3, rarity: Common,
        levels: &[
            LevelStats::new(1, 7, 4, 2, 7, 0.93, 0),
            LevelStats::new(2, 8, 5, 3, 8, 0.91, 0),
            LevelStats::new(3, 10, 5, 3, 10, 0.89, 0),
            LevelStats::new(4, 11, 6, 4, 11, 0.87, 0),
            LevelStats::new(5, 13, 6, 4, 13, 0.85, 0),
            LevelStats::new(6, 14, 7, 5, 14, 0.83, 0),
            LevelStats::new(7, 15, 7, 5, 15, 0.80, 0),
            LevelStats::new(8, 17, 8, 6, 17, 0.78, 0),
            LevelStats::new(9, 18, 8, 6, 18, 0.76, 0),
            LevelStats::new(10, 20, 9, 7, 20, 0.74, 0),
            LevelStats::new(11, 21, 9, 7, 21, 0.72, 0),
        ],
    },
    PartDefinition {
        name: "The Arc", category: Suspension, series: 5, rarity: Common,
        levels: &[
            LevelStats::new(1, 4, 12, 5, 6, 0.69, 0),
            LevelStats::new(2, 5, 14, 6, 7, 0.64, 0),
            LevelStats::new(3, 5, 15, 6, 7, 0.59, 0),
            LevelStats::new(4, 6, 17, 7, 8, 0.55, 0),
            LevelStats::new(5, 6, 18, 7, 9, 0.50, 0),
            LevelStats::new(6, 7, 20, 8, 10, 0.46, 0),
            LevelStats::new(7, 7, 21, 8, 10, 0.41, 0),
            LevelStats::new(8, 8, 23, 9, 11, 0.37, 0),
            LevelStats::new(9, 8, 24, 9, 12, 0.32, 0),
            LevelStats::new(10, 9, 26, 10, 12, 0.28, 0),
            LevelStats::new(11, 9, 27, 10, 13, 0.23, 0),
        ],
    },
    PartDefinition {
        name: "Quantum", category: Suspension, series: 7, rarity: Rare,
        levels: &[
            LevelStats::new(1, 24, 7, 11, 10, 0.76, 0),
            LevelStats::new(2, 26, 8, 12, 11, 0.73, 0),
            LevelStats::new(3, 29, 9, 13, 12, 0.70, 0),
            LevelStats::new(4, 31, 10, 14, 13, 0.68, 0),
            LevelStats::new(5, 33, 11, 15, 14, 0.65, 0),
            LevelStats::new(6, 35, 11, 15, 14, 0.62, 0),
            LevelStats::new(7, 38, 12, 16, 15, 0.60, 0),
            LevelStats::new(8, 40, 13, 17, 16, 0.57, 0),
            LevelStats::new(9, 42, 14, 18, 17, 0.55, 0),
        ],
    },
    PartDefinition {
        name: "Gyro", category: Suspension, series: 9, rarity: Rare,
        levels: &[
            LevelStats::new(1, 10, 30, 12, 11, 0.65, 0),
            LevelStats::new(2, 11, 32, 13, 12, 0.62, 0),
            LevelStats::new(3, 12, 35, 14, 13, 0.59, 0),
            LevelStats::new(4, 12, 37, 15, 14, 0.56, 0),
            LevelStats::new(5, 13, 40, 16, 15, 0.53, 0),
            LevelStats::new(6, 14, 42, 16, 15, 0.50, 0),
            LevelStats::new(7, 15, 44, 17, 16, 0.47, 0),
            LevelStats::new(8, 15, 47, 18, 17, 0.44, 0),
            LevelStats::new(9, 16, 49, 19, 18, 0.41, 0),
        ],
    },
    PartDefinition {
        name: "Joltcoil", category: Suspension, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 21, 17, 48, 21, 0.50, 0),
            LevelStats::new(2, 23, 18, 50, 23, 0.48, 0),
            LevelStats::new(3, 24, 19, 52, 24, 0.45, 0),
            LevelStats::new(4, 25, 20, 54, 25, 0.43, 0),
            LevelStats::new(5, 26, 22, 56, 26, 0.40, 0),
            LevelStats::new(6, 27, 23, 58, 27, 0.38, 0),
            LevelStats::new(7, 28, 24, 60, 28, 0.35, 0),
            LevelStats::new(8, 29, 25, 62, 29, 0.33, 0),
        ],
    },
    PartDefinition {
        name: "Nexus", category: Suspension, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 16, 17, 42, 15, 0.48, 0),
            LevelStats::new(2, 17, 18, 45, 16, 0.45, 0),
            LevelStats::new(3, 18, 19, 47, 17, 0.43, 0),
            LevelStats::new(4, 19, 20, 50, 18, 0.40, 0),
            LevelStats::new(5, 20, 21, 52, 19, 0.38, 0),
            LevelStats::new(6, 21, 22, 55, 20, 0.35, 0),
            LevelStats::new(7, 22, 23, 57, 21, 0.33, 0),
            LevelStats::new(8, 23, 24, 60, 22, 0.30, 0),
        ],
    },
    PartDefinition {
        name: "Fluxspring", category: Suspension, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 49, 17, 18, 20, 0.44, 0),
            LevelStats::new(2, 51, 18, 19, 21, 0.42, 0),
            LevelStats::new(3, 53, 19, 20, 22, 0.40, 0),
            LevelStats::new(4, 55, 20, 21, 23, 0.38, 0),
            LevelStats::new(5, 57, 21, 22, 24, 0.36, 0),
            LevelStats::new(6, 59, 22, 23, 26, 0.34, 0),
            LevelStats::new(7, 61, 23, 24, 28, 0.32, 0),
            LevelStats::new(8, 63, 25, 26, 30, 0.30, 0),
        ],
    },
    PartDefinition {
        name: "Teeter Totter", category: Suspension, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 21, 46, 19, 17, 0.48, 0),
            LevelStats::new(2, 23, 48, 20, 18, 0.46, 0),
            LevelStats::new(3, 24, 50, 21, 19, 0.44, 0),
            LevelStats::new(4, 25, 52, 22, 20, 0.42, 0),
            LevelStats::new(5, 26, 55, 23, 22, 0.40, 0),
            LevelStats::new(6, 27, 57, 25, 23, 0.38, 0),
            LevelStats::new(7, 28, 59, 26, 25, 0.36, 0),
            LevelStats::new(8, 29, 61, 28, 27, 0.34, 0),
        ],
    },

    // ==================== BRAKES ====================
    PartDefinition {
        name: "Pivot", category: Brakes, series: 1, rarity: Common,
        levels: &[
            LevelStats::new(1, 2, 2, 1, 3, 0.97, 0),
            LevelStats::new(2, 3, 3, 2, 4, 0.94, 0),
            LevelStats::new(3, 4, 4, 3, 5, 0.91, 0),
            LevelStats::new(4, 5, 5, 3, 6, 0.88, 0),
            LevelStats::new(5, 6, 6, 4, 7, 0.85, 0),
            LevelStats::new(6, 7, 7, 5, 8, 0.83, 0),
            LevelStats::new(7, 7, 7, 6, 8, 0.80, 0),
            LevelStats::new(8, 8, 8, 7, 9, 0.77, 0),
            LevelStats::new(9, 9, 9, 7, 10, 0.74, 0),
            LevelStats::new(10, 10, 10, 8, 11, 0.71, 0),
            LevelStats::new(11, 11, 11, 9, 12, 0.69, 0),
        ],
    },
    PartDefinition {
        name: "The Stabiliser", category: Brakes, series: 2, rarity: Common,
        levels: &[
            LevelStats::new(1, 3, 5, 2, 1, 0.86, 0),
            LevelStats::new(2, 4, 7, 3, 2, 0.81, 0),
            LevelStats::new(3, 4, 8, 3, 2, 0.77, 0),
            LevelStats::new(4, 5, 10, 4, 3, 0.72, 0),
            LevelStats::new(5, 5, 11, 4, 3, 0.68, 0),
            LevelStats::new(6, 6, 13, 5, 4, 0.63, 0),
            LevelStats::new(7, 6, 14, 5, 4, 0.59, 0),
            LevelStats::new(8, 7, 16, 6, 5, 0.54, 0),
            LevelStats::new(9, 7, 17, 6, 5, 0.50, 0),
            LevelStats::new(10, 8, 19, 7, 6, 0.45, 0),
            LevelStats::new(11, 8, 20, 7, 6, 0.41, 0),
        ],
    },
    PartDefinition {
        name: "Supernova", category: Brakes, series: 3, rarity: Epic,
        levels: &[
            LevelStats::new(1, 15, 5, 6, 7, 0.83, 0),
            LevelStats::new(2, 18, 6, 7, 8, 0.80, 0),
            LevelStats::new(3, 20, 7, 8, 9, 0.77, 0),
            LevelStats::new(4, 23, 8, 9, 10, 0.74, 0),
            LevelStats::new(5, 25, 8, 10, 11, 0.71, 0),
            LevelStats::new(6, 28, 9, 11, 12, 0.68, 0),
            LevelStats::new(7, 30, 10, 12, 13, 0.65, 0),
            LevelStats::new(8, 33, 11, 13, 14, 0.62, 0),
        ],
    },
    PartDefinition {
        name: "The Descent", category: Brakes, series: 5, rarity: Rare,
        levels: &[
            LevelStats::new(1, 5, 15, 6, 14, 0.83, 0),
            LevelStats::new(2, 6, 17, 7, 16, 0.80, 0),
            LevelStats::new(3, 6, 19, 8, 17, 0.78, 0),
            LevelStats::new(4, 7, 20, 8, 19, 0.76, 0),
            LevelStats::new(5, 8, 22, 9, 21, 0.74, 0),
            LevelStats::new(6, 8, 24, 10, 22, 0.72, 0),
            LevelStats::new(7, 9, 26, 11, 24, 0.69, 0),
            LevelStats::new(8, 9, 27, 11, 25, 0.67, 0),
            LevelStats::new(9, 10, 29, 12, 27, 0.65, 0),
        ],
    },
    PartDefinition {
        name: "Rumble", category: Brakes, series: 8, rarity: Rare,
        levels: &[
            LevelStats::new(1, 8, 10, 28, 11, 0.69, 0),
            LevelStats::new(2, 9, 11, 30, 12, 0.65, 0),
            LevelStats::new(3, 10, 12, 32, 13, 0.62, 0),
            LevelStats::new(4, 11, 13, 34, 14, 0.59, 0),
            LevelStats::new(5, 12, 14, 36, 15, 0.56, 0),
            LevelStats::new(6, 12, 15, 38, 16, 0.53, 0),
            LevelStats::new(7, 13, 16, 40, 17, 0.50, 0),
            LevelStats::new(8, 14, 17, 42, 18, 0.47, 0),
            LevelStats::new(9, 15, 18, 44, 19, 0.44, 0),
        ],
    },
    PartDefinition {
        name: "Flow 1K", category: Brakes, series: 10, rarity: Rare,
        levels: &[
            LevelStats::new(1, 33, 12, 13, 11, 0.62, 0),
            LevelStats::new(2, 35, 13, 14, 12, 0.59, 0),
            LevelStats::new(3, 38, 14, 15, 13, 0.57, 0),
            LevelStats::new(4, 40, 15, 16, 14, 0.55, 0),
            LevelStats::new(5, 42, 16, 17, 15, 0.53, 0),
            LevelStats::new(6, 44, 17, 18, 16, 0.51, 0),
            LevelStats::new(7, 47, 18, 19, 17, 0.48, 0),
            LevelStats::new(8, 49, 19, 20, 18, 0.46, 0),
            LevelStats::new(9, 51, 20, 21, 19, 0.44, 0),
        ],
    },
    PartDefinition {
        name: "Boombox", category: Brakes, series: 11, rarity: Epic,
        levels: &[
            LevelStats::new(1, 16, 42, 17, 14, 0.44, 0),
            LevelStats::new(2, 17, 45, 18, 15, 0.41, 0),
            LevelStats::new(3, 18, 47, 19, 16, 0.37, 0),
            LevelStats::new(4, 19, 50, 20, 17, 0.34, 0),
            LevelStats::new(5, 19, 52, 20, 18, 0.30, 0),
            LevelStats::new(6, 20, 55, 21, 19, 0.27, 0),
            LevelStats::new(7, 21, 57, 22, 20, 0.23, 0),
            LevelStats::new(8, 22, 60, 23, 21, 0.20, 0),
        ],
    },
    PartDefinition {
        name: "Grindlock", category: Brakes, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 19, 17, 49, 17, 0.52, 0),
            LevelStats::new(2, 20, 18, 51, 18, 0.50, 0),
            LevelStats::new(3, 21, 19, 53, 19, 0.48, 0),
            LevelStats::new(4, 22, 20, 55, 20, 0.45, 0),
            LevelStats::new(5, 23, 22, 57, 22, 0.42, 0),
            LevelStats::new(6, 25, 23, 59, 23, 0.39, 0),
            LevelStats::new(7, 26, 25, 61, 25, 0.36, 0),
            LevelStats::new(8, 28, 27, 63, 27, 0.35, 0),
        ],
    },

    // ==================== GEARBOX ====================
    PartDefinition {
        name: "Hustle", category: Gearbox, series: 1, rarity: Common,
        levels: &[
            LevelStats::new(1, 2, 2, 3, 1, 0.97, 0),
            LevelStats::new(2, 3, 3, 4, 2, 0.93, 0),
            LevelStats::new(3, 4, 4, 5, 3, 0.90, 0),
            LevelStats::new(4, 4, 5, 6, 3, 0.87, 0),
            LevelStats::new(5, 5, 6, 7, 4, 0.84, 0),
            LevelStats::new(6, 6, 7, 8, 5, 0.81, 0),
            LevelStats::new(7, 7, 7, 8, 6, 0.78, 0),
            LevelStats::new(8, 8, 8, 9, 7, 0.74, 0),
            LevelStats::new(9, 8, 9, 10, 7, 0.71, 0),
            LevelStats::new(10, 9, 10, 11, 8, 0.68, 0),
            LevelStats::new(11, 10, 11, 12, 9, 0.65, 0),
        ],
    },
    PartDefinition {
        name: "Slickshift", category: Gearbox, series: 2, rarity: Rare,
        levels: &[
            LevelStats::new(1, 3, 2, 7, 7, 0.90, 0),
            LevelStats::new(2, 4, 3, 9, 9, 0.87, 0),
            LevelStats::new(3, 4, 3, 11, 11, 0.84, 0),
            LevelStats::new(4, 5, 4, 12, 12, 0.82, 0),
            LevelStats::new(5, 6, 5, 14, 14, 0.79, 0),
            LevelStats::new(6, 6, 5, 16, 16, 0.76, 0),
            LevelStats::new(7, 7, 6, 18, 18, 0.74, 0),
            LevelStats::new(8, 7, 6, 19, 19, 0.71, 0),
            LevelStats::new(9, 8, 7, 21, 21, 0.69, 0),
        ],
    },
    PartDefinition {
        name: "Beat", category: Gearbox, series: 3, rarity: Common,
        levels: &[
            LevelStats::new(1, 2, 8, 4, 2, 0.79, 0),
            LevelStats::new(2, 3, 9, 5, 3, 0.74, 0),
            LevelStats::new(3, 3, 11, 5, 3, 0.69, 0),
            LevelStats::new(4, 4, 12, 6, 4, 0.64, 0),
            LevelStats::new(5, 4, 14, 7, 4, 0.59, 0),
            LevelStats::new(6, 5, 15, 8, 5, 0.55, 0),
            LevelStats::new(7, 5, 16, 8, 5, 0.50, 0),
            LevelStats::new(8, 6, 18, 9, 6, 0.45, 0),
            LevelStats::new(9, 6, 19, 10, 6, 0.40, 0),
            LevelStats::new(10, 7, 21, 10, 7, 0.35, 0),
            LevelStats::new(11, 7, 22, 11, 7, 0.30, 0),
        ],
    },
    PartDefinition {
        name: "Fury", category: Gearbox, series: 5, rarity: Epic,
        levels: &[
            LevelStats::new(1, 17, 5, 7, 16, 0.76, 0),
            LevelStats::new(2, 19, 6, 8, 18, 0.72, 0),
            LevelStats::new(3, 21, 7, 9, 19, 0.69, 0),
            LevelStats::new(4, 23, 8, 10, 21, 0.65, 0),
            LevelStats::new(5, 24, 8, 10, 23, 0.62, 0),
            LevelStats::new(6, 26, 9, 11, 25, 0.58, 0),
            LevelStats::new(7, 28, 10, 12, 26, 0.55, 0),
            LevelStats::new(8, 30, 11, 13, 28, 0.51, 0),
        ],
    },
    PartDefinition {
        name: "The Dynamo", category: Gearbox, series: 8, rarity: Rare,
        levels: &[
            LevelStats::new(1, 11, 8, 27, 11, 0.69, 0),
            LevelStats::new(2, 12, 9, 29, 12, 0.65, 0),
            LevelStats::new(3, 13, 10, 31, 13, 0.62, 0),
            LevelStats::new(4, 14, 11, 33, 14, 0.58, 0),
            LevelStats::new(5, 15, 12, 36, 15, 0.55, 0),
            LevelStats::new(6, 15, 12, 38, 15, 0.51, 0),
            LevelStats::new(7, 16, 13, 40, 16, 0.48, 0),
            LevelStats::new(8, 17, 14, 42, 17, 0.44, 0),
            LevelStats::new(9, 18, 15, 44, 18, 0.41, 0),
        ],
    },
    PartDefinition {
        name: "Metronome", category: Gearbox, series: 10, rarity: Rare,
        levels: &[
            LevelStats::new(1, 12, 33, 10, 14, 0.65, 0),
            LevelStats::new(2, 13, 36, 11, 15, 0.62, 0),
            LevelStats::new(3, 14, 38, 12, 16, 0.58, 0),
            LevelStats::new(4, 15, 41, 13, 16, 0.55, 0),
            LevelStats::new(5, 16, 43, 14, 17, 0.51, 0),
            LevelStats::new(6, 16, 46, 14, 18, 0.48, 0),
            LevelStats::new(7, 17, 48, 15, 19, 0.44, 0),
            LevelStats::new(8, 18, 51, 16, 19, 0.41, 0),
            LevelStats::new(9, 19, 53, 17, 20, 0.37, 0),
        ],
    },
    PartDefinition {
        name: "The Beast", category: Gearbox, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 42, 15, 17, 16, 0.48, 0),
            LevelStats::new(2, 44, 16, 18, 17, 0.45, 0),
            LevelStats::new(3, 47, 17, 19, 18, 0.42, 0),
            LevelStats::new(4, 49, 17, 20, 19, 0.40, 0),
            LevelStats::new(5, 51, 18, 21, 20, 0.37, 0),
            LevelStats::new(6, 53, 19, 21, 20, 0.34, 0),
            LevelStats::new(7, 56, 20, 22, 21, 0.32, 0),
            LevelStats::new(8, 60, 21, 24, 23, 0.27, 0),
        ],
    },
    PartDefinition {
        name: "Jittershift", category: Gearbox, series: 12, rarity: Epic,
        levels: &[
            LevelStats::new(1, 17, 20, 46, 19, 0.52, 0),
            LevelStats::new(2, 18, 21, 48, 20, 0.50, 0),
            LevelStats::new(3, 19, 22, 50, 21, 0.48, 0),
            LevelStats::new(4, 20, 23, 52, 22, 0.46, 0),
            LevelStats::new(5, 22, 24, 55, 23, 0.44, 0),
            LevelStats::new(6, 23, 26, 57, 25, 0.42, 0),
            LevelStats::new(7, 25, 28, 59, 26, 0.40, 0),
            LevelStats::new(8, 27, 30, 61, 28, 0.38, 0),
        ],
    },
];
