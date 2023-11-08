use std::path::Path;

use serde::{Deserialize, Serialize};

const DEFAULT_PATH: &str = ".config/aurme";
const SETTINGS_FILE: &str = "settings.json";
const CACHE_PATH: &str = ".cache/aurme";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    cache_path: String,
    keep_cache: bool,
    no_confirm: bool,
}

impl Settings {
    pub fn get_cache_path(&self) -> &str {
        &self.cache_path
    }

    pub fn get_keep_cache(&self) -> bool {
        self.keep_cache
    }
}

pub fn read() -> Settings {
    let path = format!("{}/{}/{}", home::home_dir().unwrap().display(), DEFAULT_PATH, SETTINGS_FILE);
    let config_path = std::path::Path::new(&path);

    if let Err(_) = std::fs::metadata(config_path) {
        create_default(config_path);
    }
    let json = std::fs::read_to_string(config_path).unwrap();
    serde_json::from_str(&json).unwrap()
}

fn create_default(path: &Path) {
    let settings_folder = format!("{}/{}", home::home_dir().unwrap().display(), DEFAULT_PATH);
    let settings_folder_path = std::path::Path::new(&settings_folder);

    let settings = Settings {
        cache_path: String::from(CACHE_PATH),
        keep_cache: true,
        no_confirm: true,
    };
    let json = serde_json::to_string_pretty(&settings).unwrap();
    if let Err(_) = std::fs::metadata(settings_folder_path) {
        std::fs::create_dir_all(settings_folder_path).unwrap();
    }
    std::fs::write(path, json).unwrap();
}


