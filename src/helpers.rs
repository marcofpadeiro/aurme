use crate::package::Package;
use reqwest;
use select::document::Document;
use std::process::Command;
use std::sync::Arc;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";
pub const CACHE_PATH: &str = ".cache/aur";

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
pub async fn check_package_existance(
    package_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let url = format!("{}/packages/{}", AUR_URL, package_name);
    let res = fetch(&url).await.unwrap();

    Ok(!res.contains("id=\"error-page\""))
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
        std::fs::create_dir(cache_path.as_str()).expect("Failed to create cache directory");
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
        println!("Successfully cloned package: {}", package_name);
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

            Package::new(name, None, Some(version))
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
    let url = format!("{}/packages/?K={}", AUR_URL, package_name);
    let res = fetch(&url).await.unwrap();

    // find all a tags with packages href
    Document::from(res.as_str())
        .find(select::predicate::Name("tr"))
        .flat_map(|n| n.find(select::predicate::Name("td")))
        .flat_map(|n| n.find(select::predicate::Name("a")))
        .filter(|n| n.attr("href").unwrap_or("").contains("/packages"))
        .take(10)
        .map(|n| n.text().trim().to_string())
        .collect::<Vec<String>>()
        .iter()
        .zip(
            // zip with the package description
            Document::from(res.as_str())
                .find(select::predicate::Name("tr"))
                .flat_map(|n| n.find(select::predicate::Name("td")))
                .filter(|n| n.attr("class").unwrap_or("") == "wrap")
                .take(10)
                .map(|n| n.text().trim().to_string())
                .collect::<Vec<String>>()
                .iter(),
        )
        .map(|(name, description)| {
            Package::new(name.to_string(), Some(description.to_string()), None)
        })
        .collect()
}

/**
* Runs makepkg command to build a package
* @param package_name: the name of the package to build
*/
pub fn makepkg(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building {}...", package_name);
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

/**
* Remove a package from the cache
* @param package_name: the name of the package to remove
*/
pub fn remove_package_from_cache(package_name: &str) {
    let package_path: String = format!(
        "{}/{}/{}",
        home::home_dir().unwrap().display(),
        ".cache/aur",
        package_name
    );

    std::fs::remove_dir_all(package_path).unwrap();
}
