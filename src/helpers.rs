use crate::package::Package;
use reqwest;
use select::document::Document;
use std::process::Command;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";

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
    let cache_path: String = format!("{}/{}", home::home_dir().unwrap().display(), ".cache/aur");
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
            let description = name.clone();

            Package::new(name, version, description)
        })
        .collect();

    Ok(installed_packages)
}

/**
* checks if a dependency is installed on the system
* @param dependency_name: the name of the dependency to check
*/
pub fn check_dependency(dependency_name: &str) {
    let dependency_check = Command::new("pacman")
        .arg("-Q")
        .output()
        .expect("Failed to get installed packages");

    // convert to string
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

    let names: Vec<String> = Document::from(res.as_str())
        .find(select::predicate::Name("tr"))
        .flat_map(|n| n.find(select::predicate::Name("td")))
        .flat_map(|n| n.find(select::predicate::Name("a")))
        .take(10)
        .map(|n| n.text().trim().to_string())
        .collect();

    let descriptions: Vec<String> = Document::from(res.as_str())
        .find(select::predicate::Name("tr"))
        .flat_map(|n| n.find(select::predicate::Name("td")))
        .filter(|n| n.attr("class").unwrap_or("") == "wrap")
        .take(10)
        .map(|n| n.text().trim().to_string())
        .collect();

    let packages: Vec<Package> = names
        .iter()
        .zip(descriptions.iter())
        .map(|(name, description)| {
            Package::new(name.to_string(), description.to_string(), "1".to_string())
        })
        .collect();

    packages
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
        ".cache/aur",
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
