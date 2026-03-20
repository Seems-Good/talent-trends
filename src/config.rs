use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct ClassSpecs {
    #[serde(flatten)]
    pub classes: BTreeMap<String, ClassData>,
}

#[derive(Debug, Deserialize)]
pub struct ClassData {
    pub specs: Vec<String>,
    pub color: Vec<String>,
    #[serde(rename = "pretty-color")]
    pub pretty_color: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub code: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Clone)]
pub struct Mode {
    pub name: &'static str,
    // Warcraft Logs difficulty: 3 = Normal, 4 = Heroic, 5 = Mythic
    pub difficulty: i32,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: &'static str,  // display label
    pub code: &'static str,  // WCL API value
}

// Season / bosses configuration

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub current_season: CurrentSeason,
    pub seasons: BTreeMap<String, Season>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentSeason {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Season {
    pub name: String,
    pub encounters: Vec<SeasonEncounter>,
    pub modes: Option<SeasonModes>,
    /// Optional WCL partition number. Set when a mid-season patch splits
    /// rankings (e.g. a prepatch). Omit for new seasons with no partition yet.
    pub partition: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SeasonEncounter {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SeasonModes {
    pub default: i32,
    pub allowed: Vec<i32>,
}

impl ClassSpecs {
    pub fn load() -> Self {
        const CONFIG: &str = include_str!("../classes.toml");
        toml::from_str(CONFIG).expect("Failed to parse classes.toml")
    }

    pub fn class_names(&self) -> Vec<String> {
        self.classes.keys().map(|k| k.replace('_', " ")).collect()
    }

    pub fn get_specs(&self, class_name: &str) -> Option<Vec<String>> {
        let key = class_name.replace(' ', "_");
        self.classes.get(&key).map(|c| c.specs.clone())
    }

    pub fn get_regions() -> Vec<Region> {
        vec![
            Region { code: "all", name: "All Regions" },
            Region { code: "US",  name: "US & Oceanic" },
            Region { code: "EU",  name: "Europe" },
            Region { code: "KR",  name: "Korea" },
            Region { code: "TW",  name: "Taiwan" },
            Region { code: "CN",  name: "China" },
        ]
    }

    pub fn get_modes() -> Vec<Mode> {
        vec![
            Mode { name: "Normal", difficulty: 3 },
            Mode { name: "Heroic", difficulty: 4 },
            Mode { name: "Mythic", difficulty: 5 },
        ]
    }

    pub fn get_metrics() -> Vec<Metric> {
        vec![
            Metric { name: "Damage",       code: "dps" },
            Metric { name: "Healing",      code: "hps" },
            Metric { name: "Tank Healing", code: "tankhps" },
        ]
    }
}

impl Settings {
    pub fn load() -> Self {
        const SETTINGS: &str = include_str!("../settings.toml");
        toml::from_str(SETTINGS).expect("Failed to parse settings.toml")
    }

    pub fn current_encounters(&self) -> Vec<SeasonEncounter> {
        let id = &self.current_season.id;
        self.seasons.get(id).map(|s| s.encounters.clone()).unwrap_or_default()
    }

    pub fn default_difficulty(&self) -> i32 {
        let id = &self.current_season.id;
        self.seasons
            .get(id)
            .and_then(|s| s.modes.as_ref())
            .map(|m| m.default)
            .unwrap_or(5)
    }

    pub fn allowed_difficulties(&self) -> Vec<i32> {
        let id = &self.current_season.id;
        self.seasons
            .get(id)
            .and_then(|s| s.modes.as_ref())
            .map(|m| m.allowed.clone())
            .unwrap_or_else(|| vec![3, 4, 5])
    }

    pub fn current_partition(&self) -> Option<i32> {
        let id = &self.current_season.id;
        self.seasons.get(id).and_then(|s| s.partition)
    }
}
