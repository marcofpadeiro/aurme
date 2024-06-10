use std::path::PathBuf;

use async_trait::async_trait;
use aurme::expand_path;

use crate::theme::{colorize, Type};

use super::handler::CommandHandler;

pub struct ClearHandler;

#[async_trait]
impl CommandHandler for ClearHandler {
    async fn handle(&self, matches: &clap::ArgMatches, _config: &crate::config::Config) {
        let packages: Vec<_> = matches
            .get_many::<String>("package")
            .map(|vals| vals.map(|s| s.as_str()).collect())
            .unwrap_or_else(Vec::new);

        let cache_path: PathBuf = expand_path(crate::config::PACKAGES_CACHE_PATH);
        if !cache_path.exists() {
            std::fs::create_dir_all(cache_path).unwrap();
            println!(
                "{} cleared cache",
                colorize(Type::Success, "Successfully cleared cache"),
            );
            return;
        }

        if packages.len() == 0 {
            std::fs::remove_dir_all(&cache_path).unwrap();
            println!("{} cleared cache", colorize(Type::Success, "Successfully"),);
            return;
        }

        let mut packages_deleted: Vec<&str> = Vec::new();
        let mut packages_not_found: Vec<&str> = Vec::new();
        packages.iter().for_each(|package| {
            let package_path = cache_path.join(package);

            if !package_path.exists() {
                packages_not_found.push(package);
                return;
            }
            std::fs::remove_file(format!("{}.tar.gz", package_path.display())).unwrap();
            std::fs::remove_dir_all(package_path).unwrap();
            packages_deleted.push(package);
        });

        if packages_not_found.len() != 0 {
            println!(
                "{} no entries found for:",
                colorize(Type::Warning, "Warning"),
            );
            packages_not_found.iter().for_each(|package| {
                println!("  {}", package);
            });
        }
        if packages_deleted.len() == 0 {
            return;
        }

        println!(
            "{} cleared cache of packages:",
            colorize(Type::Success, "Successfully")
        );
        packages_deleted.iter().for_each(|package| {
            println!("  {}", package);
        });
    }
}
