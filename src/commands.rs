use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn build_lookup_command() -> Command {
    Command::new("lookup")
        .short_flag('L')
        .long_flag("lookup")
        .about("Lookup packages on database (used for autocomplete)")
        .arg(
            Arg::new("word")
                .help("word")
                .action(ArgAction::Set)
                .required(true)
                .index(1),
        )
}

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
            Arg::new("clear")
                .long("clear")
                .short('c')
                .conflicts_with_all(&["search", "sysupgrade", "refresh"])
                .action(ArgAction::SetTrue)
                .help("clear package cache"),
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
                .required_unless_present_any(&["search", "clear", "refresh", "sysupgrade"])
                .action(ArgAction::Set)
                .num_args(1..),
        )
}

