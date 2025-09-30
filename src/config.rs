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
}

#[derive(Debug, Clone)]
pub struct Encounter {
    pub id: i32,
    pub name: &'static str,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub code: &'static str,
    pub name: &'static str,
}

impl ClassSpecs {
    pub fn load() -> Self {
        const CONFIG: &str = include_str!("../classes.toml");
        toml::from_str(CONFIG).expect("Failed to parse classes.toml")
    }
    
    // pub fn class_names(&self) -> Vec<String> {
    //     self.classes.keys()
    //         .map(|k| k.replace('_', " "))
    //         .collect()
    // }
    //
    // pub fn get_specs(&self, class_name: &str) -> Option<&Vec<String>> {
    //     let key = class_name.replace(' ', "_");
    //     self.classes.get(&key).map(|c| &c.specs)
    // }
}

// (Season 3 of The War Within)
pub fn get_encounters() -> Vec<Encounter> {
    vec![
         Encounter { id: 3129, name: "Plexus Sentinel" },
         Encounter { id: 3131, name: "Loom'ithar" },
         Encounter { id: 3130, name: "Soulbinder Naazindhri" },
         Encounter { id: 3132, name: "Forgeweaver Araz" },
         Encounter { id: 3122, name: "The Soul Hunters" },
         Encounter { id: 3133, name: "Fractillus" },
         Encounter { id: 3134, name: "Nexus-King Salahadaar" },
         Encounter { id: 3135, name: "Dimensius, the All-Devouring" },
     ]
}

pub fn get_regions() -> Vec<Region> {
    vec![
        Region { code: "all", name: "All Regions" },
        Region { code: "US", name: "US & Oceanic" },
        Region { code: "EU", name: "Europe" },
        Region { code: "KR", name: "Korea" },
        Region { code: "TW", name: "Taiwan" },
        Region { code: "CN", name: "China" },
    ]
}


// (Season 1 The War Within)
//     // vec![
//     //     Encounter { id: 2902, name: "Ulgrax the Devourer" },
//     //     Encounter { id: 2917, name: "The Bloodbound Horror" },
//     //     Encounter { id: 2898, name: "Sikran" },
//     //     Encounter { id: 2918, name: "Rasha'nan" },
//     //     Encounter { id: 2919, name: "Broodtwister Ovi'nax" },
//     //     Encounter { id: 2920, name: "Nexus-Princess Ky'veza" },
//     //     Encounter { id: 2921, name: "The Silken Court" },
//     //     Encounter { id: 2922, name: "Queen Ansurek" },
//     // ]
// }

