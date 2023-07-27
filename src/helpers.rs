use crate::package::Package;
use crate::theme::colorize;
use crate::theme::Type;
use reqwest;
use serde_json::Value;
use std::process::Command;
use std::result;
use std::sync::Arc;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";
pub const CACHE_PATH: &str = ".cache/aurme";


/**
* helper function to fetch the html of a page
* @param url: the url of the page
* @return the html of the page
*/
pub async fn fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = reqwest::get(url).await?.text().await?;
    Ok(res)
}

/**
* helper function to check if package exists
* @param package_name: the name of the package
* @return true if the package exists, false otherwise
*/
pub async fn check_packages_existance(
    package_names: &Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut url = format!("{}/rpc/?v=5&type=info", AUR_URL);
    url = format!("{}&arg[]={}", url, package_names.join("&arg[]="));
    
    let res = fetch(&url).await.unwrap();
    let json: Value = serde_json::from_str(&res).unwrap();

    let non_existent_packages: Vec<String> = match json["resultcount"].as_u64() {
        Some(result_count) if result_count < package_names.len() as u64 => {
            let existent_packages: Vec<String> = json["results"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|result| result["Name"].as_str().unwrap_or("").to_owned())
                .collect();

            package_names
                .iter()
                .filter(|package| !existent_packages.contains(package))
                .cloned()
                .collect()
        }
        _ => Vec::new(),
    };

    Ok(non_existent_packages)
}

/**
* helper function to get the git link of a package
* @param package_name: the name of the package
* @return the git link of the package
*/
pub fn get_git_url(package_name: &str) -> String {
    format!("{}/{}.git", AUR_URL, package_name.to_lowercase())
}

/**
* Clone a package from the AUR
* @param package_name: the name of the package
*/
pub fn clone_package(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_path: String = format!("{}/{}", home::home_dir().unwrap().display(), CACHE_PATH);
    let package_path: String = format!("{}/{}", cache_path, package_name);

    check_dependency("git");

    if !std::path::Path::new(cache_path.as_str()).exists() {
        std::fs::create_dir_all(cache_path.as_str()).expect("Failed to create cache directory");
    }

    //// if dir with package name already exists, delete it
    if std::path::Path::new(package_path.as_str()).exists() {
        std::fs::remove_dir_all(package_path.as_str()).expect("Failed to remove old package");
    }

    // specify the directory to clone the package to
    let exit_status = Command::new("git")
        .arg("clone")
        .arg(get_git_url(package_name))
        .arg(package_path)
        .output()
        .unwrap();

    if exit_status.status.code().unwrap() != 0 {
        Err(String::from_utf8_lossy(&exit_status.stderr).into())
    } else {
        println!(
            "{} cloned package {}",
            colorize(Type::Success, "Successfully"),
            package_name
        );
        match makepkg(&package_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
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

            Package::new(name, None, Some(version), None, None)
        })
        .collect();

    Ok(installed_packages)
}

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
        if installed_packages_str.contains(&package) {
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
            ));
        } else {
            packages_missing.push(package);
        }
    }

    if packages_missing.is_empty() {
        Ok(packages_installed)
    } else {
        Err(packages_missing)
    }
}

/**
* Checks for updates using threads
* @param packages: a vector of packages to check for updates
* @return tuple of package and the new version
*/
pub async fn check_for_updates_threads(
    packages: Vec<Package>,
) -> Result<Vec<(Package, String)>, Box<dyn std::error::Error>> {
    let packages = Arc::new(packages);

    let mut tasks = Vec::new();

    for package in packages.iter() {
        let package = package.clone();
        let task = tokio::spawn(async move {
            let result = package.check_for_package_updates().await;
            match result {
                Ok(result) => Ok(result),
                Err(e) => Err(e),
            }
        });

        tasks.push(task);
    }

    let mut results: Vec<(Package, String)> = Vec::new();

    for task in tasks {
        let result = task.await?;
        match result {
            Ok(result) => results.push(result),
            Err(_) => continue,
        }
    }

    Ok(results)
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

/**
* Scrapes AUR page for top 10 packages
* @param package_name: the package name to search for
* @return a vector of the top 10 packages
*/
pub async fn get_top_packages(package_name: &str) -> Vec<Package> {
    let url = format!("{}/rpc/?v=5&type=search&arg={}", AUR_URL, package_name);
    let res = fetch(&url).await.unwrap();

    let json: Value = serde_json::from_str(&res).unwrap();

    let mut top_packages: Vec<Package> = Vec::new();
    let results = json["results"].as_array().unwrap();
    if results.is_empty() {
        return top_packages;
    }

    for result in results.iter() {
        let new = serde_json::from_value::<Package>(result.clone());
        match new {
            Ok(new) => top_packages.push(new),
            Err(_) => continue,
        }
    }

    top_packages.sort_by(|a, b| b.get_popularity().partial_cmp(&a.get_popularity()).unwrap());
    top_packages.truncate(10);
    top_packages
}

/**
* Runs makepkg command to build a package
* @param package_name: the name of the package to build
*/
pub fn makepkg(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("  {} {}...", colorize(Type::Info, "Building"), package_name);
    let package_path: String = format!(
        "{}/{}/{}",
        home::home_dir().unwrap().display(),
        CACHE_PATH,
        package_name
    );

    check_dependency("fakeroot");
    check_dependency("make");

    let exit_status = Command::new("makepkg")
        .arg("-si")
        .arg("--noconfirm")
        .current_dir(package_path)
        .output()
        .unwrap();

    if exit_status.status.code().unwrap() != 0 {
        Err(String::from_utf8_lossy(&exit_status.stderr).into())
    } else {
        Ok(())
    }
}