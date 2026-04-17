#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverRarity {
    Common,
    Rare,
    Epic,
    Legendary,
    ProspectStandard,
    ProspectTurbocharged,
    PodiumStars,
    PodiumStarsLegends,
}

impl DriverRarity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Common => "Common",
            Self::Rare => "Rare",
            Self::Epic => "Epic",
            Self::Legendary => "Legendary",
            Self::ProspectStandard => "Prospect Standard",
            Self::ProspectTurbocharged => "Prospect Turbocharged",
            Self::PodiumStars => "Podium Stars",
            Self::PodiumStarsLegends => "Podium Stars Legends",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Common => "rarity-common",
            Self::Rare => "rarity-rare",
            Self::Epic => "rarity-epic",
            Self::Legendary => "rarity-legendary",
            Self::ProspectStandard => "rarity-prospect-std",
            Self::ProspectTurbocharged => "rarity-prospect-turbo",
            Self::PodiumStars => "rarity-podium",
            Self::PodiumStarsLegends => "rarity-podium-legends",
        }
    }

    pub fn db_key(&self) -> &'static str {
        self.label()
    }

    pub fn from_db(s: &str) -> Option<DriverRarity> {
        match s {
            "Common" => Some(Self::Common),
            "Rare" => Some(Self::Rare),
            "Epic" => Some(Self::Epic),
            "Legendary" => Some(Self::Legendary),
            "Prospect Standard" => Some(Self::ProspectStandard),
            "Prospect Turbocharged" => Some(Self::ProspectTurbocharged),
            "Podium Stars" => Some(Self::PodiumStars),
            "Podium Stars Legends" => Some(Self::PodiumStarsLegends),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverCategory {
    Normal,
    Legendary,
    SpecialEdition,
}

impl DriverCategory {
    pub fn all() -> &'static [DriverCategory] {
        &[Self::Normal, Self::Legendary, Self::SpecialEdition]
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Legendary => "Legendary",
            Self::SpecialEdition => "Special Edition",
        }
    }
}

impl DriverRarity {
    pub fn category(&self) -> DriverCategory {
        match self {
            Self::Common | Self::Rare | Self::Epic => DriverCategory::Normal,
            Self::Legendary => DriverCategory::Legendary,
            _ => DriverCategory::SpecialEdition,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- DriverRarity::from_db ---

    #[test]
    fn from_db_parses_all_variants() {
        assert_eq!(DriverRarity::from_db("Common"), Some(DriverRarity::Common));
        assert_eq!(DriverRarity::from_db("Rare"), Some(DriverRarity::Rare));
        assert_eq!(DriverRarity::from_db("Epic"), Some(DriverRarity::Epic));
        assert_eq!(
            DriverRarity::from_db("Legendary"),
            Some(DriverRarity::Legendary)
        );
        assert_eq!(
            DriverRarity::from_db("Prospect Standard"),
            Some(DriverRarity::ProspectStandard)
        );
        assert_eq!(
            DriverRarity::from_db("Prospect Turbocharged"),
            Some(DriverRarity::ProspectTurbocharged)
        );
        assert_eq!(
            DriverRarity::from_db("Podium Stars"),
            Some(DriverRarity::PodiumStars)
        );
        assert_eq!(
            DriverRarity::from_db("Podium Stars Legends"),
            Some(DriverRarity::PodiumStarsLegends)
        );
    }

    #[test]
    fn from_db_returns_none_for_unknown() {
        assert!(DriverRarity::from_db("").is_none());
        assert!(DriverRarity::from_db("common").is_none()); // case-sensitive
        assert!(DriverRarity::from_db("Unknown").is_none());
    }

    // --- DriverRarity::category ---

    #[test]
    fn category_normal_for_common_rare_epic() {
        assert_eq!(DriverRarity::Common.category(), DriverCategory::Normal);
        assert_eq!(DriverRarity::Rare.category(), DriverCategory::Normal);
        assert_eq!(DriverRarity::Epic.category(), DriverCategory::Normal);
    }

    #[test]
    fn category_legendary_for_legendary() {
        assert_eq!(
            DriverRarity::Legendary.category(),
            DriverCategory::Legendary
        );
    }

    #[test]
    fn category_special_edition_for_rest() {
        assert_eq!(
            DriverRarity::ProspectStandard.category(),
            DriverCategory::SpecialEdition
        );
        assert_eq!(
            DriverRarity::ProspectTurbocharged.category(),
            DriverCategory::SpecialEdition
        );
        assert_eq!(
            DriverRarity::PodiumStars.category(),
            DriverCategory::SpecialEdition
        );
        assert_eq!(
            DriverRarity::PodiumStarsLegends.category(),
            DriverCategory::SpecialEdition
        );
    }
}

// The driver catalog is now stored in the database and seeded from drivers.json.
// DriverRarity and DriverCategory remain here because templates use them for
// CSS classes, labels, and category grouping.
