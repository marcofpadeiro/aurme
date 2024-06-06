use crate::handlers::*;
use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn build_sync_command() -> Command {
    Command::new("sync")
        .short_flag('S')
        .long_flag("sync")
        .about("Synchronize packages.")
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .help("search remote repositories for matching strings")
                .conflicts_with_all(&["info", "sysupgrade", "refresh"])
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(
            Arg::new("info")
                .long("info")
                .conflicts_with_all(&["search", "sysupgrade", "refresh"])
                .short('i')
                .action(ArgAction::SetTrue)
                .help("view package information"),
        )
        .arg(
            Arg::new("refresh")
                .long("refresh")
                .short('y')
                .conflicts_with_all(&["info", "search"])
                .action(ArgAction::SetTrue)
                .help("download fresh package databases"),
        )
        .arg(
            Arg::new("sysupgrade")
                .long("sysupgrade")
                .short('u')
                .conflicts_with_all(&["info", "search"])
                .action(ArgAction::SetTrue)
                .help("upgrade all out-of-date packages"),
        )
        .arg(
            Arg::new("package")
                .help("packages")
                .required_unless_present_any(&["search", "refresh", "sysupgrade"])
                .action(ArgAction::Set)
                .num_args(1..),
        )
}

pub fn get_sync_handler(sync_matches: &ArgMatches) -> Box<dyn handler::CommandHandler> {
    match () {
        _ if sync_matches.contains_id("search") => Box::new(search::SearchHandler),
        _ if sync_matches.get_flag("info") => Box::new(info::InfoHandler),
        _ if sync_matches.get_flag("refresh") => Box::new(refresh::RefreshHandler),
        _ if sync_matches.get_flag("sysupgrade") => Box::new(sysupgrade::SysUpgradeHandler),
        _ => Box::new(install::InstallHandler),
    }
}
