use crate::handlers::*;
use clap::{Arg, ArgAction, ArgMatches, Command};

use async_trait::async_trait;
use handler::CommandHandler;

pub struct CompositeHandler {
    handlers: Vec<Box<dyn handler::CommandHandler + Send + Sync>>,
}

impl CompositeHandler {
    pub fn new() -> Self {
        CompositeHandler {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Box<dyn handler::CommandHandler + Send + Sync>) {
        self.handlers.push(handler);
    }
}

#[async_trait]
impl handler::CommandHandler for CompositeHandler {
    async fn handle(&self, matches: &clap::ArgMatches, config: &crate::config::Config) {
        for handler in &self.handlers {
            handler.handle(matches, config).await;
        }
    }
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

pub fn get_sync_handler(sync_matches: &ArgMatches) -> Box<dyn CommandHandler + Send + Sync> {
    if sync_matches.contains_id("search") {
        return Box::new(search::SearchHandler);
    }

    if sync_matches.get_flag("info") {
        return Box::new(info::InfoHandler);
    }

    let mut composite_handler = CompositeHandler::new();

    if sync_matches.get_flag("refresh") {
        composite_handler.add_handler(Box::new(refresh::RefreshHandler));
    }

    if sync_matches.get_flag("sysupgrade") {
        composite_handler.add_handler(Box::new(sysupgrade::SysUpgradeHandler));
    }

    if !composite_handler.handlers.is_empty() {
        Box::new(composite_handler)
    } else {
        Box::new(install::InstallHandler)
    }
}
