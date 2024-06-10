use crate::{
    config::PACKAGES_CACHE_PATH, database::read_database, helpers::{download_package, makepkg}
};
use async_trait::async_trait;
use aurme::expand_path;

use crate::{
    helpers::check_packages_existance,
    package::Package,
    theme::{colorize, Type},
};

use super::handler::CommandHandler;

pub struct InstallHandler;

#[async_trait]
impl CommandHandler for InstallHandler {
    async fn handle(&self, matches: &clap::ArgMatches, config: &crate::config::Config) {
        let packages: Vec<_> = matches
            .get_many::<String>("package")
            .map(|vals| vals.map(|s| s.as_str()).collect())
            .unwrap_or_else(Vec::new);

        let packages_db = read_database().await.unwrap();

        let existent_packages: Vec<Package>;
        let non_existent_packages: Vec<String> =
            match check_packages_existance(&packages, &packages_db) {
                Ok((non_existent_packages, packages)) => {
                    existent_packages = packages;
                    non_existent_packages
                }
                Err(err) => {
                    println!("{} {}", colorize(Type::Error, "Error:"), err);
                    return;
                }
            };

        if non_existent_packages.len() > 0 {
            println!(
                "{} The following packages do not exist in the AUR:",
                colorize(Type::Error, "\nError:")
            );
            non_existent_packages.iter().for_each(|package| {
                println!("  {}", package);
            });
            return;
        }

        let cache_path = expand_path(PACKAGES_CACHE_PATH);
        let cache_path = std::path::Path::new(&cache_path);

        for package in existent_packages.iter() {
            match download_package(&package).await {
                Ok(_) => {
                    makepkg(&package.name, &config).unwrap();
                    println!("{}", colorize(Type::Info, "Package installed"));
                }
                Err(e) => {
                    println!("{} {}", colorize(Type::Error, "Error:"), e);
                    std::fs::remove_dir_all(cache_path.join(&package.name)).unwrap();
                }
            };
        }
    }
}
