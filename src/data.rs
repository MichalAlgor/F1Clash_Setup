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
        if self.speed { out.push("Speed"); }
        if self.cornering { out.push("Cornering"); }
        if self.power_unit { out.push("Power Unit"); }
        if self.qualifying { out.push("Qualifying"); }
        out
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

    #[test]
    fn stat_priorities_any_selected_false_when_all_false() {
        assert!(!StatPriorities::default().any_selected());
    }

    #[test]
    fn stat_priorities_any_selected_true_when_one_set() {
        assert!(StatPriorities { speed: true, ..Default::default() }.any_selected());
        assert!(StatPriorities { cornering: true, ..Default::default() }.any_selected());
        assert!(StatPriorities { power_unit: true, ..Default::default() }.any_selected());
        assert!(StatPriorities { qualifying: true, ..Default::default() }.any_selected());
    }

    #[test]
    fn stat_priorities_labels_empty_when_none_selected() {
        assert!(StatPriorities::default().labels().is_empty());
    }

    #[test]
    fn stat_priorities_labels_correct_order() {
        let p = StatPriorities { speed: true, cornering: true, power_unit: true, qualifying: true };
        assert_eq!(p.labels(), vec!["Speed", "Cornering", "Power Unit", "Qualifying"]);
    }

    #[test]
    fn stat_priorities_labels_single() {
        let p = StatPriorities { qualifying: true, ..Default::default() };
        assert_eq!(p.labels(), vec!["Qualifying"]);
    }
}
