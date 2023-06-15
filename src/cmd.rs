// Purpose: Handle command line arguments
use crate::commands;
use crate::errors;

pub struct Config {
    pub arg: String,
    pub values: Vec<String>,
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
}

pub async fn handle_args(config: Config) {
    match config.arg.as_str() {
        "-S" => {
            if config.values.len() == 0 {
                errors::handle_error("no packages specified");
            } else {
                commands::handle_install(config.values).await;
            }
        }
        "-Ss" => {
            if config.values.len() == 0 {
                errors::handle_error("no packages specified");
            } else {
                commands::handle_search(config.values[0].clone()).await;
            }
        }
        "-Syu" => commands::handle_update(),
        _ => errors::handle_error(config.arg.as_str()),
    }
}
