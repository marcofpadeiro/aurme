use crate::config::Config;
use crate::config::VerboseOtion;
use crate::config::PACKAGES_CACHE_PATH;
use crate::database::NON_ALPHA;
use crate::package::Package;
use crate::theme::colorize;
use crate::theme::Type;
use aurme::expand_path;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tar::Archive;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";

/**
* helper function to check if package exists @param package_name: the name of the package
* @return true if the package exists, false otherwise
*/
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

/**
* Clone a package from the AUR
* @param package_name: the name of the package
*/
pub async fn download_package(package: &Package) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut file = File::create(&file_path)?;
    file.write_all(&response)?;

    let file = File::open(&file_path)?;
    let mut archive = Archive::new(GzDecoder::new(file));

    archive.unpack(cache_path)?;

    println!(
        "{} downloaded package {}",
        colorize(Type::Success, "Successfully"),
        package.name
    );
    Ok(())
}

/**
* Checks system for installed packages
* @return: a vector of installed packages
**/
pub fn get_installed_packages() -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let installed_packages_output = Command::new("pacman")
        .arg("-Qm")
        .output()
        .expect("Failed to get installed packages");

    // Extract the installed packages as a string
    let installed_packages_str = std::str::from_utf8(&installed_packages_output.stdout)?;

    // Split the string into individual package names
    let package_lines: Vec<&str> = installed_packages_str.trim().split('\n').collect();

    // Split each line into name and version
    let installed_packages: Vec<Package> = package_lines
        .into_iter()
        .map(|package_line| {
            let mut package_parts = package_line.split_whitespace();
            let name = package_parts.next().unwrap_or("").to_owned();
            let version = package_parts.next().unwrap_or("").to_owned();

            Package::new(name, None, Some(version), None, None, None)
        })
        .collect();

    Ok(installed_packages)
}

#[allow(dead_code)]
pub fn check_if_packages_installed(packages: Vec<String>) -> Result<Vec<Package>, Vec<String>> {
    let installed_packages_output = Command::new("pacman")
        .arg("-Qm")
        .output()
        .expect("Failed to get installed packages");

    // Extract the installed packages as a string
    let installed_packages_str = String::from_utf8(installed_packages_output.stdout)
        .expect("Failed to get installed packages");

    let mut packages_installed: Vec<Package> = Vec::new();
    let mut packages_missing: Vec<String> = Vec::new();

    for package in packages {
        if !installed_packages_str.contains(&package) {
            packages_missing.push(package);
            continue;
        }
        // Extract the package name and version from the installed packages
        let parts: Vec<&str> = installed_packages_str
            .lines()
            .find(|line| line.starts_with(&package))
            .map(|line| line.split_whitespace().collect())
            .unwrap_or_else(Vec::new);

        let package_name = parts[0].to_owned();
        let package_version = parts[1].to_owned();
        packages_installed.push(Package::new(
            package_name.clone(),
            Some(package_name),
            Some(package_version),
            None,
            None,
            None,
        ));
    }

    if packages_missing.is_empty() {
        return Ok(packages_installed);
    }
    Err(packages_missing)
}

pub fn name_to_key(package_name: &str) -> String {
    let first_char = package_name.chars().next().unwrap();
    match first_char.is_alphabetic() {
        true => first_char.to_string().to_lowercase(),
        false => NON_ALPHA.to_string(),
    }
}

/**
* Checks if a dependency is installed in the system
* @param dependency_name: the name of the dependency
*/
pub fn check_dependency(dependency_name: &str) {
    let dependency_check = Command::new("pacman")
        .arg("-Q")
        .output()
        .expect("Failed to get installed packages");

    let output = String::from_utf8_lossy(&dependency_check.stdout);

    if !output.contains(dependency_name) {
        std::eprintln!(
            "{} is not installed, please install it first",
            dependency_name
        );
        std::process::exit(1);
    }
}

/** Scrapes AUR page for top 10 packages
* @param package_name: the package name to search for
* @return a vector of the top 10 packages
*/
pub fn get_top_packages(
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

/**
* Runs makepkg command to build a package
* @param package_name: the name of the package to build
*/
pub fn makepkg(package_name: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("  {} {}...", colorize(Type::Info, "Building"), package_name);
    let package_path = expand_path(PACKAGES_CACHE_PATH).join(package_name);

    check_dependency("fakeroot");
    check_dependency("make");

    let no_confirm = match &config.no_confirm {
        true => "--noconfirm",
        false => "",
    };

    let (stdout, stderr) = config.get_verbose_config();

    let exit_status = Command::new("makepkg")
        .arg("-si")
        .arg(no_confirm)
        .stdout(stdout)
        .stderr(stderr)
        .current_dir(package_path.clone())
        .spawn()?
        .wait_with_output()
        .expect("Error running makepkg process");

    // clear cache depending on user config
    if !config.keep_cache {
        std::fs::remove_file(format!("{}.tar.gz", package_path.display())).unwrap();
        std::fs::remove_dir_all(package_path).unwrap();
    }

    if exit_status.status.code().unwrap() != 0 {
        return Err(match &config.verbose {
            VerboseOtion::Quiet => String::from_utf8_lossy(&exit_status.stderr).into(),
            _ => "Check Above Output".into(),
        });
    }
    Ok(())
}
