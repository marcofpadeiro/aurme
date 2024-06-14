mod clean;
mod config;
mod database;
mod install;
mod package;
mod query;
mod theme;

use crate::clean::handle_clean;
use crate::config::Config;
use crate::config::CONFIG_PATH;
use crate::install::handle_install;
use crate::install::handle_sysupgrade;
use clap::ArgMatches;
use database::download_database;
use query::handle_info;
use query::handle_search;
use std::error::Error;

pub const NON_ALPHA: &str = "non_alpha";

pub async fn run(matches: ArgMatches) {
    let result = match matches.subcommand() {
        Some(("sync", sync_matches)) => {
            let config = match Config::read(CONFIG_PATH) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!(
                        "{}: {}",
                        theme::colorize(theme::Type::Error, "Error getting config file"),
                        e
                    );
                    return;
                }
            };
            handle_sync(sync_matches, &config).await
        }
        Some(("lookup", lookup_matches)) => query::handle_lookup(lookup_matches).await,
        _ => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("{}: {}", theme::colorize(theme::Type::Error, "Error: "), e);
    }
}

pub fn name_to_key(package_name: &str) -> String {
    let first_char = package_name.chars().next().unwrap();
    match first_char.is_alphabetic() {
        true => first_char.to_string().to_lowercase(),
        false => NON_ALPHA.to_string(),
    }
}

async fn handle_sync(sync_matches: &ArgMatches, config: &Config) -> Result<(), Box<dyn Error>> {
    if sync_matches.contains_id("search") {
        let search_term = sync_matches
            .get_one::<String>("search")
            .expect("impossible to get here");
        return handle_search(search_term, &config).await;
    }

    let packages: Vec<_> = sync_matches
        .get_many::<String>("package")
        .map(|vals| vals.map(|s| s.as_str()).collect())
        .unwrap_or_else(Vec::new);

    if sync_matches.get_flag("refresh") && sync_matches.get_flag("sysupgrade") {
        download_database().await?;
        handle_sysupgrade(&packages, &config).await?;
    };

    if sync_matches.get_flag("refresh") {
        download_database().await?;
    };

    if sync_matches.get_flag("sysupgrade") {
        handle_sysupgrade(&packages, &config).await?;
    };

    if sync_matches.get_flag("info") {
        handle_info(&packages, &config)?;
    };

    if sync_matches.get_flag("clear") {
        handle_clean(&packages, &config)?;
    };

    handle_install(&packages, &config).await
}
