// Purpose: Handle command line arguments
use crate::commands;
use crate::errors;
use crate::settings;

pub struct Config {
    arg: String,
    values: Vec<String>,
    settings: settings::Settings,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() == 0 {
            return Err("usage");
        }

        if !args[0].starts_with("-") {
            return Err("usage");
        }

        let arg = args[0].clone();
        let values: Vec<String> = args[1..].to_vec();
        let settings = settings::read();

        return Ok(Config { arg, values, settings });
    }

    pub async fn handle_args(&self) {
        let settings = self.settings.to_owned();
        match (self.arg.as_str(), self.values.as_slice()) {
            ("-S", values) => commands::handle_install(values.to_vec(), settings).await,
            ("-Ss", [query]) => commands::handle_search(query.clone(), settings).await,
            ("-Syu" | "-Suy", values) => commands::handle_update(values.to_vec(), settings).await,
            ("-Sc", values) => commands::handle_cache_delete(values.to_vec(), settings).await,
            _ => errors::handle_error(self.arg.as_str()),
        }
    }
}
