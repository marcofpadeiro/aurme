use std::io::{stdin, stdout, Write};

use async_trait::async_trait;

use crate::{
    database::read_database,
    helpers::{clone_package, get_installed_packages},
    package::Package,
    theme::{colorize, Type},
};

use super::handler::CommandHandler;

pub struct SysUpgradeHandler;

#[async_trait]
impl CommandHandler for SysUpgradeHandler {
    async fn handle(&self, _matches: &clap::ArgMatches, config: &crate::config::Config) {
        let packages_db = read_database(&config).await.unwrap();
        let mut installed_packages = get_installed_packages().unwrap();

        let mut outdated: Vec<(Package, Package)> = Vec::new();
        packages_db.iter().for_each(|package| {
            installed_packages.retain(|installed_package| {
                if installed_package.name == package.name {
                    if installed_package.version != package.version {
                        outdated.push((installed_package.clone(), package.clone()));
                    }
                    return false;
                }
                true
            });
        });

        if outdated.len() == 0 {
            println!("{}", colorize(Type::Header, "System is up to date"));
            return;
        }

        println!(
            "{}",
            colorize(
                Type::Header,
                format!("Packages ({}) ", outdated.len()).as_str()
            )
        );

        outdated.iter().for_each(|(local, db)| {
            println!(
                "   {} ({} -> {})",
                local.name,
                colorize(Type::Error, &local.version),
                colorize(Type::Success, &db.version),
            );
        });

        print!("\nProceed with update? [Y/n]:");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        if input != "" && input != "y" && input != "Y" {
            println!("{}", colorize(Type::Warning, "Aborting..."));
            return;
        }

        outdated
            .iter()
            .for_each(|(_, package)| match clone_package(&package, &config) {
                Ok(_) => eprintln!(
                    "{} updated {}",
                    colorize(Type::Success, "Successfully"),
                    package.name
                ),
                Err(e) => println!("{} {}", colorize(Type::Error, "Error:"), e),
            });
    }
}
