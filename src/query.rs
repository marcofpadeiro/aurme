use clap::ArgMatches;

use crate::name_to_key;
use crate::package::Package;
use crate::theme::colorize;
use crate::theme::Type;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::Write;

use crate::{config::Config, database::read_database};

pub async fn handle_search(search_term: &String, config: &Config) -> Result<(), Box<dyn Error>> {
    let packages_db = read_database()?;

    let packages = get_top_packages(&search_term, &packages_db);

    let len = packages.len();
    if len == 0 {
        println!("No packages found");
        return Ok(());
    }

    packages.iter().rev().enumerate().for_each(|(i, package)| {
        println!(
            "\n{} {}\n  {}",
            colorize(Type::Info, format!("{} ┃", len - i).as_str()),
            colorize(Type::Header, package.name.as_str()),
            package.get_description()
        );
    });

    print!("\nInstall package(s) (1-10) or (q)uit: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    if input == "q" || input == "quit" {
        return Ok(());
    }

    let parsed_input: Result<usize, _> = input.parse();

    // temp
    Ok(())

    // match parsed_input {
    //     Ok(i) if i > 0 && i <= packages.len() => match download_package(&packages[i - 1]).await {
    //         Ok(_) => {
    //             makepkg(packages[i - 1].name.as_str(), &config).unwrap();
    //             println!("{}", colorize(Type::Success, "Package installed"));
    //         }
    //         Err(e) => println!("{} {}", colorize(Type::Error, "\nError:"), e),
    //     },
    //     _ => println!(
    //         "{}",
    //         colorize(Type::Warning, "Invalid input or package out of range")
    //     ),
    // }
}

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

fn get_top_packages(
    package_name: &str,
    packages_db: &HashMap<String, Vec<Package>>,
) -> Vec<Package> {
    let mut top_packages: Vec<Package> = packages_db
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
        .map(|package| package.clone())
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

pub fn check_packages_existance(
    package_names: &Vec<&str>,
    packages_db: &HashMap<String, Vec<Package>>,
) -> Result<(Vec<String>, Vec<Package>), Box<dyn std::error::Error>> {
    let mut existent_packages: Vec<Package> = Vec::new();
    package_names.iter().for_each(|package| {
        if let Some(packages) = packages_db.get(&name_to_key(package)) {
            let package = packages.iter().find(|p| p.name == *package);
            if let Some(package) = package {
                existent_packages.push(package.clone());
            }
        }
    });

    // filter out the packages that don't exist
    let non_existent = package_names
        .iter()
        .filter(|package_name| {
            !existent_packages
                .iter()
                .any(|package| package.name == **package_name)
        })
        .map(|package_name| package_name.to_string())
        .collect();

    Ok((non_existent, existent_packages))
}
