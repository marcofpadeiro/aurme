// Purpose: Handle command line arguments
use crate::commands;
use crate::errors;

pub struct Config {
    arg: String,
    values: Vec<String>,
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

        return Ok(Config { arg, values });
    }

    pub async fn handle_args(&self) {
        match (self.arg.as_str(), self.values.as_slice()) {
            ("-S", values) => commands::handle_install(values.to_vec()).await,
            ("-Ss", [query]) => commands::handle_search(query.clone()).await,
            ("-Syu" | "-Suy", values) => commands::handle_update(values.to_vec()).await,
            ("-Sc", values) => commands::handle_cache_delete(values.to_vec()).await,
            _ => errors::handle_error(self.arg.as_str()),
        }
    }
}
