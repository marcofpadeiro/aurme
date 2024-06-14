use flate2::read::GzDecoder;
use tar::Archive;

use crate::build::build_packages;
use crate::clean::remove_cache;
use crate::cli::get_yes_no;
use crate::cli::print_outdated_packages;
use crate::config::expand_path;
use crate::config::PACKAGES_CACHE_PATH;
use crate::database::get_installed_packages;
use crate::query::get_outdated_packages;
use std::fs::File;
use std::{error::Error, io::Write};

use crate::{
    config::Config,
    database::read_database,
    package::Package,
    query::query_exact_package,
    theme::{colorize, Type},
};

pub const AUR_URL: &str = "https://aur.archlinux.org";

pub async fn handle_install(packages: &Vec<&str>, config: &Config) -> Result<(), Box<dyn Error>> {
    let packages_db = read_database()?;

    let mut existent_packages: Vec<&Package> = Vec::new();
    let mut non_existent_packages: Vec<&str> = Vec::new();

    packages
        .iter()
        .for_each(|package| match query_exact_package(package, &packages_db) {
            Some(x) => existent_packages.push(x),
            None => non_existent_packages.push(package),
        });

    if non_existent_packages.len() > 0 {
        println!(
            "{} The following packages do not exist in the AUR:",
            colorize(Type::Error, "\nError:")
        );
        non_existent_packages.iter().for_each(|package| {
            println!("  {}", package);
        });
        return Ok(());
    }

    install_packages(&existent_packages, config).await
}

pub async fn handle_sysupgrade(packages: &[&str], config: &Config) -> Result<(), Box<dyn Error>> {
    let packages_db = read_database()?;
    let installed_packages = get_installed_packages()?;

    // TODO: updated only specific packages

    let outdated: Vec<(&Package, &Package)> =
        get_outdated_packages(&installed_packages, &packages_db);

    if outdated.len() == 0 {
        println!("{}", colorize(Type::Header, "System is up to date"));
        return Ok(());
    }

    print_outdated_packages(&outdated);

    if !get_yes_no("Proceed with update?") {
        println!("{}", colorize(Type::Warning, "Aborting..."));
        return Ok(());
    }

    let packages = outdated.iter().map(|(_, db)| *db).collect();

    install_packages(&packages, config).await
}

/// The argument `packages` is assumed to have already been validated for their existance on AUR
pub async fn install_packages(
    packages: &Vec<&Package>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    download_packages(packages).await?;
    build_packages(packages, config)
}

async fn download_packages(packages: &Vec<&Package>) -> Result<(), Box<dyn std::error::Error>> {
    let mut successfully_downloaded: Vec<&Package> = Vec::new();
    for package in packages {
        match download(&package).await {
            Ok(_) => {
                successfully_downloaded.push(package);
                eprintln!(
                    "{} downloaded {}",
                    colorize(Type::Success, "Successfully"),
                    package.name
                );
            }
            Err(e) => {
                eprintln!(
                    "{} to download {}",
                    colorize(Type::Error, "Failed"),
                    package.name
                );
                remove_cache(successfully_downloaded)?;
                return Err(e);
            }
        }
    }
    Ok(())
}

async fn download(package: &Package) -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = expand_path(PACKAGES_CACHE_PATH);
    let package_folder = cache_path.join(&package.name);

    if !cache_path.exists() {
        std::fs::create_dir_all(&cache_path).expect("Failed to create cache directory");
    }

    if package_folder.exists() {
        std::fs::remove_dir_all(package_folder).expect("Failed to remove old package");
    }

    let response = reqwest::get(package.get_url_path()).await?.bytes().await?;
    let file_path = cache_path.join(format!("{}.tar.gz", package.name));

    let decoder = GzDecoder::new(&response[..]);

    let mut file = File::create(&file_path)?;
    file.write_all(&response)?;

    let mut archive = Archive::new(decoder);

    archive.unpack(cache_path)?;

    Ok(())
}
