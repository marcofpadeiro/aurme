use clap::{Arg, ArgAction, Command};

pub fn build_sync_command() -> Command {
    Command::new("sync")
        .short_flag('S')
        .long_flag("sync")
        .about("Synchronize packages.")
        .arg(
            Arg::new("search")
                .long("search")
                .conflicts_with_all(&["info", "sysupgrade", "refresh"])
                .short('s')
                .num_args(1)
                .action(ArgAction::Set)
                .help("search remote repositories for matching strings"),
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
