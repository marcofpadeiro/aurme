use crate::handlers::*;
use clap::Command;
use command_line::build_sync_command;

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

    let command_handler: Box<dyn handler::CommandHandler> = match matches.subcommand() {
        Some(("sync", sync_matches)) => match () {
            _ if sync_matches.contains_id("search") => Box::new(search::SearchHandler),
            _ if sync_matches.get_flag("info") => Box::new(info::InfoHandler),
            _ if sync_matches.get_flag("refresh") => Box::new(refresh::RefreshHandler),
            _ if sync_matches.get_flag("sysupgrade") => Box::new(sysupgrade::SysUpgradeHandler),
            _ => Box::new(install::InstallHandler),
        },
        _ => unreachable!(),
    };

    command_handler.handle(&matches, &config).await;
}
