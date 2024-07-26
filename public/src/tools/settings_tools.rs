use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Settings {
    api_key: String,
    secret_key: String,
}

impl Settings {
    pub fn get_api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn get_secret_key(&self) -> String {
        self.secret_key.clone()
    }
}

pub fn load_settings(path: &str) -> Settings {
    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let settings: Settings = serde_yml::from_str(&contents).expect("Unable to parse YAML");

    settings
}
