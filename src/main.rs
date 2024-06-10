use std::process::exit;

use clap::{ArgMatches, Command};
use command_line::{build_sync_command, get_sync_handler};
use config::{Config, CONFIG_PATH};
use handlers::handler;

mod command_line;
mod config;
mod database;
mod handlers;
mod helpers;
mod package;
mod theme;

#[tokio::main]
async fn main() {
    let matches = Command::new("aurme")
        .about("AUR wrapper utility")
        .version("0.0.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(build_sync_command())
        .get_matches();

    let config = match Config::read(CONFIG_PATH) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "{}: {}",
                theme::colorize(theme::Type::Error, "Error getting config file"),
                e
            );
            exit(1);
        }
    };

    let (command_handler, res_matches): (Box<dyn handler::CommandHandler>, &ArgMatches) =
        match matches.subcommand() {
            Some(("sync", sync_matches)) => (get_sync_handler(sync_matches), sync_matches),
            _ => unreachable!(),
        };

    command_handler.handle(res_matches, &config).await;
}
