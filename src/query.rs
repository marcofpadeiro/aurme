use clap::ArgMatches;

use crate::cli::{get_value_from_range, print_top_packages};
use crate::install::install_packages;
use crate::name_to_key;
use crate::package::Package;
use crate::theme::{colorize, Type};
use std::collections::HashMap;
use std::error::Error;
use std::process::exit;

use crate::{config::Config, database::read_database};

pub async fn handle_search(search_term: &String, config: &Config) -> Result<(), Box<dyn Error>> {
    let packages_db = match read_database() {
        Ok(x) => x,
        Err(_) => {
            eprintln!(
                "{} to read database. Try to {} to refresh the local database",
                colorize(Type::Error, "Failed"),
                colorize(Type::Info, "aurme -Sy"),
            );
            exit(1);
        }
    };

    let top_packages = get_top_packages(&search_term, &packages_db);

    let len = top_packages.len();
    if len == 0 {
        println!("No packages found");
        return Ok(());
    }

    print_top_packages(&top_packages);

    if let Some(i) = get_value_from_range("Install package(s)", 1, len)? {
        return install_packages(&vec![top_packages[i - 1]], config).await;
    }

    Ok(())
}

#[allow(dead_code, unused)]
pub fn handle_info(packages: &[&str], config: &Config) -> Result<(), Box<dyn Error>> {
    todo!()
}

pub async fn handle_lookup(lookup_matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let search_term = lookup_matches
        .get_one::<String>("word")
        .expect("impossible to reach");

    let packages = read_database()?;

    print_packages_matching(search_term, packages);

    Ok(())
}

fn get_top_packages<'a>(
    package_name: &str,
    packages_db: &'a HashMap<String, Vec<Package>>,
) -> Vec<&'a Package> {
    let mut top_packages: Vec<&Package> = packages_db
        .iter()
        .flat_map(|(_, packages)| packages.iter())
        .filter(|package| {
            package
                .name
                .to_lowercase()
                .contains(&package_name.to_lowercase())
                || package
                    .get_description()
                    .to_lowercase()
                    .contains(&package_name.to_lowercase())
        })
        .map(|package| package)
        .collect();

    if top_packages.is_empty() {
        return top_packages;
    }

    top_packages.sort_by(|a, b| b.popularity.partial_cmp(&a.popularity).unwrap());
    top_packages.truncate(10);
    top_packages
}

fn print_packages_matching(search_term: &String, packages: HashMap<String, Vec<Package>>) {
    if let Some(packages) = packages.get(&name_to_key(search_term)) {
        packages.iter().for_each(|package| {
            if package.name.starts_with(search_term) {
                println!("{}", package.name);
            }
        });
    }
}

pub fn query_exact_package<'a>(
    package: &str,
    database: &'a HashMap<String, Vec<Package>>,
) -> Option<&'a Package> {
    if let Some(packages) = database.get(&name_to_key(package)) {
        return packages.iter().find(|p| p.name == *package);
    }
    None
}

pub fn get_outdated_packages<'a, 'b>(
    installed_packages: &'a Vec<Package>,
    database: &'b HashMap<String, Vec<Package>>,
) -> Vec<(&'a Package, &'b Package)> {
    let mut outdated: Vec<(&Package, &Package)> = Vec::new();

    installed_packages.iter().for_each(|package| {
        let db_package = match query_exact_package(&package.name, database) {
            Some(x) => x,
            None => {
                if !package.name.ends_with("debug") {
                    eprintln!(
                        "{}",
                        colorize(
                            Type::Warning,
                            &format!(
                                "Package {} no longer exists in AUR. Skipping...",
                                package.name
                            )
                        )
                    );
                }
                return;
            }
        };
        if db_package.version != package.version {
            outdated.push((package, db_package));
        }
    });

    outdated
}
