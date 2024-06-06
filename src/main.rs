use clap::{ArgMatches, Command};
use command_line::{build_sync_command, get_sync_handler};
use handlers::handler;

mod command_line;
mod config;
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

    let config = config::read();

    let (command_handler, res_matches): (Box<dyn handler::CommandHandler>, &ArgMatches) =
        match matches.subcommand() {
            Some(("sync", sync_matches)) => (get_sync_handler(sync_matches), sync_matches),
            _ => unreachable!(),
        };

    command_handler.handle(res_matches, &config).await;
}
