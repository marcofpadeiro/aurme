use std::{path::PathBuf, process::Stdio};

use aurme::expand_path;
use serde::{Deserialize, Serialize};

use crate::theme;

pub const CACHE_PATH: &str = "~/.cache/aurme";
pub const PACKAGES_CACHE_PATH: &str = "~/.cache/aurme/packages";
pub const CONFIG_PATH: &str = "~/.config/aurme/config.json";


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum VerboseOtion {
    Quiet,
    Default,
    Verbose,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub keep_cache: bool,
    pub no_confirm: bool,
    pub verbose: VerboseOtion,
}

impl Config {
    pub fn default() -> Config {
        Config {
            keep_cache: true,
            no_confirm: false,
            verbose: VerboseOtion::Default,
        }
    }

    fn create(config: &Config, config_path: &PathBuf) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&config)?;
        if let Err(_) = std::fs::metadata(config_path) {
            std::fs::create_dir_all(
                config_path
                    .parent()
                    .expect("Path doesn't have a parent directory"),
            )?;
        }

        std::fs::write(config_path, json)
    }

    pub fn read(path: &str) -> Result<Config, std::io::Error> {
        let config_path: PathBuf = expand_path(path);

        if !config_path.exists() {
            let default = Config::default();
            Config::create(&default, &config_path)?;
            return Ok(default);
        }

        let json = std::fs::read_to_string(config_path)?;
        match serde_json::from_str::<Config>(&json) {
            Ok(config) => return Ok(config),
            Err(_) => eprintln!(
                "{}",
                theme::colorize(
                    theme::Type::Warning,
                    "Couldn't read config file. Running off the default config"
                )
            ),
        };

        Ok(Config::default())
    }

    pub fn get_verbose_config(&self) -> (Stdio, Stdio) {
        match self.verbose {
            VerboseOtion::Verbose => (
                std::process::Stdio::inherit(),
                std::process::Stdio::inherit(),
            ),
            VerboseOtion::Quiet => (std::process::Stdio::piped(), std::process::Stdio::piped()),
            VerboseOtion::Default => (std::process::Stdio::piped(), std::process::Stdio::inherit()),
        }
    }
}
