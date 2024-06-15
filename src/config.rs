use home::home_dir;
use std::path::{Path, PathBuf};
use std::process::Stdio;

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

        let json = std::fs::read_to_string(config_path.to_owned())?;
        if let Ok(config) = serde_json::from_str::<Config>(&json) {
            return Ok(config);
        }

        eprintln!(
            "{}",
            theme::colorize(
                theme::Type::Warning,
                "Couldn't read config file.
                \nYour config file will be overwritten with the default config.
                \nOld config file has been renamed to config.old.\n"
            )
        );

        std::fs::rename(&config_path, config_path.with_extension("old"))?;
        Config::create(&Config::default(), &config_path)?;

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

pub fn expand_path(path: &str) -> PathBuf {
    let mut expanded_path = path.to_string();

    if expanded_path.starts_with("~") {
        if let Some(home) = home_dir() {
            let home_str = home.to_str().unwrap_or("~");
            expanded_path = expanded_path.replacen("~", home_str, 1);
        }
    }

    if expanded_path.contains('$') {
        expanded_path = shellexpand::env(&expanded_path)
            .unwrap_or_else(|_| expanded_path.clone().into())
            .to_string();
    }

    Path::new(&expanded_path).to_path_buf()
}
