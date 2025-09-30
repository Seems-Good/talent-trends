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

impl ClassSpecs {
    pub fn load() -> Self {
        const CONFIG: &str = include_str!("../classes.toml");
        toml::from_str(CONFIG).expect("Failed to parse classes.toml")
    }
    
    pub fn class_names(&self) -> Vec<String> {
        self.classes.keys()
            .map(|k| k.replace('_', " "))
            .collect()
    }
    
    pub fn get_specs(&self, class_name: &str) -> Option<&Vec<String>> {
        let key = class_name.replace(' ', "_");
        self.classes.get(&key).map(|c| &c.specs)
    }
}
