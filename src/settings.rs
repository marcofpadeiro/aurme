use std::{path::Path, process::Stdio};

use serde::{Deserialize, Serialize};

use crate::theme;

const DEFAULT_PATH: &str = ".config/aurme";
const SETTINGS_FILE: &str = "settings.json";
const CACHE_PATH: &str = ".cache/aurme";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    cache_path: String,
    keep_cache: bool,
    no_confirm: bool,
    verbose: String,
}

impl Settings {
    pub fn get_cache_path(&self) -> &str {
        &self.cache_path
    }

    pub fn get_keep_cache(&self) -> bool {
        self.keep_cache
    }

    pub fn get_no_confirm(&self) -> bool {
        self.no_confirm
    }

    pub fn get_verbose_settings(&self) -> (Stdio, Stdio) {
        match self.verbose.as_str() {
            "verbose" => {
                (std::process::Stdio::inherit(), std::process::Stdio::inherit())
            }
            "quiet" => {
                (std::process::Stdio::piped(), std::process::Stdio::piped())
            }
            _ => {
                (std::process::Stdio::piped(), std::process::Stdio::inherit())
            }
        }
    }
}

pub fn read() -> Settings {
    let path = format!("{}/{}/{}", home::home_dir().unwrap().display(), DEFAULT_PATH, SETTINGS_FILE);
    let config_path = std::path::Path::new(&path);

    if let Err(_) = std::fs::metadata(config_path) {
       return create_default(config_path);
    }

    let json = std::fs::read_to_string(config_path).unwrap();
    if let Ok(settings) = serde_json::from_str::<Settings>(&json) {
        return settings;
    }

    println!(
        "{}",
        theme::colorize(theme::Type::Warning,
    "Your settings file has been updated to the latest version.")
    );
    println!(
        "{}",
        theme::colorize(theme::Type::Warning,
    "Old settings file has been renamed to settings.json.old.")
    );
    std::fs::rename(config_path, format!("{}.old", path)).unwrap();

    create_default(config_path)
}

fn create_default(path: &Path) -> Settings {
    let settings_folder = format!("{}/{}", home::home_dir().unwrap().display(), DEFAULT_PATH);
    let settings_folder_path = std::path::Path::new(&settings_folder);

    let settings = Settings {
        cache_path: String::from(CACHE_PATH),
        keep_cache: true,
        no_confirm: true,
        verbose: String::from("normal"),
    };
    let json = serde_json::to_string_pretty(&settings).unwrap();
    if let Err(_) = std::fs::metadata(settings_folder_path) {
        std::fs::create_dir_all(settings_folder_path).unwrap();
    }
    std::fs::write(path, json).unwrap();
    settings
}


