use std::io::{stdin, stdout, Write};

use async_trait::async_trait;

use crate::{
    database::read_database,
    helpers::{download_package, get_installed_packages, makepkg, name_to_key},
    package::Package,
    theme::{colorize, Type},
};

use super::handler::CommandHandler;

pub struct SysUpgradeHandler;

#[async_trait]
impl CommandHandler for SysUpgradeHandler {
    async fn handle(&self, _matches: &clap::ArgMatches, config: &crate::config::Config) {
        let packages_db = read_database().await.unwrap();
        let mut outdated: Vec<(Package, Package)> = Vec::new();
        get_installed_packages()
            .unwrap()
            .iter()
            .for_each(|package| {
                if let Some(packages) = packages_db.get(&name_to_key(&package.name)) {
                    let db_package = packages.iter().find(|p| p.name == package.name);
                    if let Some(db_package) = db_package {
                        if db_package.version != package.version {
                            outdated.push((package.clone(), db_package.clone()));
                        }
                    }
                }
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

        for (_, package) in outdated.iter() {
            match download_package(&package).await {
                Ok(_) => {
                    eprintln!(
                        "{} updated {}",
                        colorize(Type::Success, "Successfully"),
                        package.name
                    );
                    makepkg(package.name.as_str(), &config).unwrap();
                }
                Err(e) => println!("{} {}", colorize(Type::Error, "Error:"), e),
            };
        }
    }
}
