/// Part stat priorities used by the optimizer.
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
        if self.speed {
            out.push("Speed");
        }
        if self.cornering {
            out.push("Cornering");
        }
        if self.power_unit {
            out.push("Power Unit");
        }
        if self.qualifying {
            out.push("Qualifying");
        }
        out
    }
}

// ── Upgrade calculator ────────────────────────────────────────────────────────

/// Cards required to upgrade from level N to N+1 (index 0 = L1→L2).
/// Same for all rarities and series.
pub const CARD_COSTS: &[i32] = &[4, 10, 20, 50, 100, 200, 400, 1_000, 2_000, 4_000];

/// Coin cost to upgrade from level N to N+1, per series (outer index = series - 1,
/// inner index 0 = L1→L2).
const COIN_COSTS_S2025: &[&[u64]] = &[
    // Series 1
    &[
        2_000, 8_000, 35_000, 90_000, 275_000, 800_000, 1_600_000, 2_400_000, 3_200_000, 4_000_000,
    ],
    // Series 2
    &[
        6_000, 30_000, 95_000, 300_000, 950_000, 1_900_000, 3_750_000, 5_600_000, 7_450_000,
        9_300_000,
    ],
    // Series 3
    &[
        22_000, 45_000, 135_000, 450_000, 1_350_000, 2_800_000, 5_250_000, 7_700_000, 10_150_000,
        12_600_000,
    ],
    // Series 4
    &[
        1_950_000, 3_850_000, 5_800_000, 7_700_000, 15_750_000, 28_000_000, 51_750_000,
    ],
    // Series 5
    &[
        4_500_000,
        9_000_000,
        20_000_000,
        42_250_000,
        139_250_000,
        167_000_000,
        195_000_000,
        223_000_000,
    ],
    // Series 6
    &[
        790_000, 1_600_000, 2_400_000, 3_200_000, 6_800_000, 19_500_000, 36_250_000, 53_000_000,
        69_750_000, 86_500_000,
    ],
    // Series 7
    &[
        1_950_000, 3_850_000, 5_800_000, 7_700_000, 15_750_000, 28_000_000, 51_750_000, 75_500_000,
    ],
    // Series 8
    &[
        4_500_000,
        9_000_000,
        20_000_000,
        42_250_000,
        139_250_000,
        167_000_000,
        195_000_000,
        223_000_000,
    ],
    // Series 9
    &[
        9_500_000,
        19_000_000,
        45_000_000,
        105_000_000,
        199_000_000,
        239_000_000,
        278_000_000,
        317_000_000,
    ],
    // Series 10
    &[
        22_000_000,
        43_000_000,
        65_000_000,
        150_000_000,
        284_000_000,
        341_000_000,
        398_000_000,
    ],
    // Series 11
    &[
        54_000_000,
        107_000_000,
        161_000_000,
        215_000_000,
        406_000_000,
        487_000_000,
        568_000_000,
    ],
    // Series 12
    &[
        116_000_000,
        232_000_000,
        348_000_000,
        464_000_000,
        580_000_000,
        696_000_000,
        812_000_000,
    ],
];

// Season 2026 — still being gathered; values confirmed so far differ on S1–S4.
const COIN_COSTS_S2026: &[&[u64]] = &[
    // Series 1
    &[
        2_000, 8_000, 60_000, 190_000, 640_000, 800_000, 1_600_000, 2_400_000, 3_200_000, 4_000_000,
    ],
    // Series 2
    &[
        8_600, 30_000, 95_000, 300_000, 950_000, 1_900_000, 3_750_000, 5_600_000, 7_450_000,
        9_300_000,
    ],
    // Series 3
    &[
        35_000, 45_000, 135_000, 450_000, 1_350_000, 2_800_000, 5_250_000, 7_700_000, 10_150_000,
        12_600_000,
    ],
    // Series 4
    &[
        110_000, 3_850_000, 5_800_000, 7_700_000, 15_750_000, 28_000_000, 51_750_000,
    ],
    // Series 5
    &[
        4_500_000,
        9_000_000,
        20_000_000,
        42_250_000,
        139_250_000,
        167_000_000,
        195_000_000,
        223_000_000,
    ],
    // Series 6
    &[
        790_000, 1_600_000, 2_400_000, 3_200_000, 6_800_000, 19_500_000, 36_250_000, 53_000_000,
        69_750_000, 86_500_000,
    ],
    // Series 7
    &[
        1_950_000, 3_850_000, 5_800_000, 7_700_000, 15_750_000, 28_000_000, 51_750_000, 75_500_000,
    ],
    // Series 8
    &[
        4_500_000,
        9_000_000,
        20_000_000,
        42_250_000,
        139_250_000,
        167_000_000,
        195_000_000,
        223_000_000,
    ],
    // Series 9
    &[
        9_500_000,
        19_000_000,
        45_000_000,
        105_000_000,
        199_000_000,
        239_000_000,
        278_000_000,
        317_000_000,
    ],
    // Series 10
    &[
        22_000_000,
        43_000_000,
        65_000_000,
        150_000_000,
        284_000_000,
        341_000_000,
        398_000_000,
    ],
    // Series 11
    &[
        54_000_000,
        107_000_000,
        161_000_000,
        215_000_000,
        406_000_000,
        487_000_000,
        568_000_000,
    ],
    // Series 12
    &[
        116_000_000,
        232_000_000,
        348_000_000,
        464_000_000,
        580_000_000,
        696_000_000,
        812_000_000,
    ],
];

/// Returns the coin cost table for the given season string.
/// Defaults to 2025 costs for any unrecognised season.
pub fn coin_costs_for_season(season: &str) -> &'static [&'static [u64]] {
    match season {
        "2026" => COIN_COSTS_S2026,
        _ => COIN_COSTS_S2025,
    }
}

/// Maximum upgrade level for a given rarity.
pub fn max_level_for_rarity(rarity: &str) -> i32 {
    match rarity {
        "Epic" => 8,
        "Rare" => 9,
        _ => 11, // Common
    }
}

/// Result of an upgrade calculation.
pub struct UpgradeInfo {
    /// Highest level reachable with the cards currently owned.
    pub reachable_level: i32,
    /// Total coins needed to reach that level from the current level.
    pub coins_needed: u64,
    /// Cards still needed to reach the next level beyond what is owned.
    pub cards_to_next: i32,
}

/// Calculate how far a part can be upgraded given cards owned.
pub fn calculate_upgrade(
    current_level: i32,
    cards_owned: i32,
    series: i32,
    rarity: &str,
    season: &str,
) -> UpgradeInfo {
    let max_lvl = max_level_for_rarity(rarity);
    let coin_table = coin_costs_for_season(season)
        .get((series - 1) as usize)
        .copied()
        .unwrap_or(&[]);

    let mut cards_remaining = cards_owned;
    let mut coins_needed = 0u64;
    let mut reachable_level = current_level;
    let mut cards_to_next = 0;

    for from_level in current_level..max_lvl {
        let idx = (from_level - 1) as usize;
        let card_cost = CARD_COSTS.get(idx).copied().unwrap_or(0);
        let coin_cost = coin_table.get(idx).copied().unwrap_or(0);

        if cards_remaining >= card_cost {
            cards_remaining -= card_cost;
            coins_needed += coin_cost;
            reachable_level = from_level + 1;
        } else {
            cards_to_next = card_cost - cards_remaining;
            break;
        }
    }

    UpgradeInfo {
        reachable_level,
        coins_needed,
        cards_to_next,
    }
}

/// How many cards are needed to reach a target level from the current level.
/// Returns (reachable_level, cards_to_next) — no coin data required.
pub fn calculate_upgrade_cards_only(
    current_level: i32,
    cards_owned: i32,
    max_level: i32,
) -> (i32, i32) {
    let mut cards_remaining = cards_owned;
    let mut reachable = current_level;

    for from_level in current_level..max_level {
        let idx = (from_level - 1) as usize;
        let card_cost = CARD_COSTS.get(idx).copied().unwrap_or(0);
        if cards_remaining >= card_cost {
            cards_remaining -= card_cost;
            reachable = from_level + 1;
        } else {
            return (reachable, card_cost - cards_remaining);
        }
    }
    (reachable, 0)
}

/// Format a large coin number as a compact string (e.g. 1_250_000 → "1.3M").
pub fn format_coins(coins: u64) -> String {
    if coins >= 1_000_000_000 {
        format!("{:.1}B", coins as f64 / 1_000_000_000.0)
    } else if coins >= 1_000_000 {
        format!("{:.1}M", coins as f64 / 1_000_000.0)
    } else if coins >= 1_000 {
        format!("{:.0}K", coins as f64 / 1_000.0)
    } else {
        format!("{coins}")
    }
}

/// Part rarity — kept here for use in drivers_data context.
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Upgrade calculator ---

    #[test]
    fn max_level_for_rarity_common() {
        assert_eq!(max_level_for_rarity("Common"), 11);
    }

    #[test]
    fn max_level_for_rarity_rare() {
        assert_eq!(max_level_for_rarity("Rare"), 9);
    }

    #[test]
    fn max_level_for_rarity_epic() {
        assert_eq!(max_level_for_rarity("Epic"), 8);
    }

    #[test]
    fn calculate_upgrade_already_at_max() {
        let info = calculate_upgrade(8, 999, 1, "Epic", "2025");
        assert_eq!(info.reachable_level, 8);
        assert_eq!(info.coins_needed, 0);
    }

    #[test]
    fn calculate_upgrade_no_cards() {
        let info = calculate_upgrade(1, 0, 1, "Common", "2025");
        assert_eq!(info.reachable_level, 1);
        assert_eq!(info.coins_needed, 0);
        assert_eq!(info.cards_to_next, 4); // need 4 cards for L1→L2
    }

    #[test]
    fn calculate_upgrade_exact_cards_for_one_level() {
        // 4 cards = exactly enough for L1→L2, series 1 costs 2_000 coins
        let info = calculate_upgrade(1, 4, 1, "Common", "2025");
        assert_eq!(info.reachable_level, 2);
        assert_eq!(info.coins_needed, 2_000);
    }

    #[test]
    fn calculate_upgrade_two_levels() {
        // 4 + 10 = 14 cards → L1→L2→L3, series 1: 2_000 + 8_000 = 10_000 coins
        let info = calculate_upgrade(1, 14, 1, "Common", "2025");
        assert_eq!(info.reachable_level, 3);
        assert_eq!(info.coins_needed, 10_000);
    }

    #[test]
    fn calculate_upgrade_not_enough_for_next() {
        // Only 3 cards at L1 — need 4 for L1→L2
        let info = calculate_upgrade(1, 3, 1, "Common", "2025");
        assert_eq!(info.reachable_level, 1);
        assert_eq!(info.coins_needed, 0);
        assert_eq!(info.cards_to_next, 1);
    }

    #[test]
    fn calculate_upgrade_series_2_costs() {
        // 4 cards at L1, series 2 costs 6_000 for L1→L2 in 2025
        let info = calculate_upgrade(1, 4, 2, "Common", "2025");
        assert_eq!(info.reachable_level, 2);
        assert_eq!(info.coins_needed, 6_000);
    }

    #[test]
    fn format_coins_small() {
        assert_eq!(format_coins(500), "500");
    }

    #[test]
    fn format_coins_thousands() {
        assert_eq!(format_coins(2_000), "2K");
    }

    #[test]
    fn format_coins_millions() {
        assert_eq!(format_coins(1_250_000), "1.2M");
        assert_eq!(format_coins(1_750_000), "1.8M");
    }

    #[test]
    fn format_coins_billions() {
        assert_eq!(format_coins(1_200_000_000), "1.2B");
    }

    // --- StatPriorities ---

    #[test]
    fn stat_priorities_any_selected_false_when_all_false() {
        assert!(!StatPriorities::default().any_selected());
    }

    #[test]
    fn stat_priorities_any_selected_true_when_one_set() {
        assert!(
            StatPriorities {
                speed: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            StatPriorities {
                cornering: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            StatPriorities {
                power_unit: true,
                ..Default::default()
            }
            .any_selected()
        );
        assert!(
            StatPriorities {
                qualifying: true,
                ..Default::default()
            }
            .any_selected()
        );
    }

    #[test]
    fn stat_priorities_labels_empty_when_none_selected() {
        assert!(StatPriorities::default().labels().is_empty());
    }

    #[test]
    fn stat_priorities_labels_correct_order() {
        let p = StatPriorities {
            speed: true,
            cornering: true,
            power_unit: true,
            qualifying: true,
        };
        assert_eq!(
            p.labels(),
            vec!["Speed", "Cornering", "Power Unit", "Qualifying"]
        );
    }

    #[test]
    fn stat_priorities_labels_single() {
        let p = StatPriorities {
            qualifying: true,
            ..Default::default()
        };
        assert_eq!(p.labels(), vec!["Qualifying"]);
    }
}
