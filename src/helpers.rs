use reqwest;
use serde_json::Value;
use std::process::Command;
use std::sync::Arc;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";

#[derive(Clone)]
pub struct Package {
    name: String,
    version: String,
    description: String,
}

impl Package {
    pub fn new(name: String, description: String, version: String) -> Package {
        Package {
            name,
            description,
            version,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }
}

/**
* helper function to fetch the html of a page
* @param url: the url of the page
*/
pub async fn fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = reqwest::get(url).await?.text().await?;
    Ok(res)
}

/**
* helper function to check if package exists
* @param package_name: the name of the package
*/
pub async fn check_package(package_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let url = format!("{}/packages/{}", AUR_URL, package_name);
    let res = fetch(&url).await.unwrap();

    Ok(!res.contains("id=\"error-page\""))
}

/**
* helper function to get the git link of a package
* @param package_name: the name of the package
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

            Package {
                name,
                description,
                version,
            }
        })
        .collect();

    Ok(installed_packages)
}

pub async fn check_for_package_updates(name: &str, version: &str) -> Result<bool, String> {
    let url = format!(
        "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
        name
    );

    let response = reqwest::get(&url).await.expect("asd").text().await;

    let json: Value = serde_json::from_str(&response.expect("Failed to get response")).unwrap();

    let new_version = json
        .get("results")
        .and_then(|results| {
            results.as_array().and_then(|results_array| {
                results_array.iter().find(|result| result["Name"] == name)
            })
        })
        .and_then(|result| result.get("Version"))
        .and_then(|version| version.as_str())
        .ok_or_else(|| "Invalid JSON response or version not found".to_string())?;

    Ok(version != new_version)
}

pub async fn check_for_updates_threads(
    packages: Vec<Package>,
) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let packages = Arc::new(packages); // Wrap the vector in an Arc

    let mut tasks = Vec::new();

    for number in 0..packages.len() {
        let packages_clone = Arc::clone(&packages); // Create a clone of Arc for each task
        let task = tokio::spawn(async move {
            let package_name = packages_clone[number].get_name().clone();
            let package_version = packages_clone[number].get_version().clone();
            let result = check_for_package_updates(package_name, package_version).await;
            (number, result.expect("Failed to get result"))
        });
        tasks.push(task);
    }

    let mut results: Vec<Package> = Vec::new();

    for task in tasks {
        let (number, result) = task.await?;
        if result == true {
            results.push(packages[number].clone());
        }
    }

    Ok(results)
}

pub fn check_if_package_in_cache(package_name: &str) -> bool {
    let cache_path: String = format!("{}/{}", home::home_dir().unwrap().display(), ".cache/aur");
    let package_path: String = format!("{}/{}", cache_path, package_name);

    std::path::Path::new(package_path.as_str()).exists()
}

pub fn pull_cached_package(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let package_path: String = format!(
        "{}/{}/{}",
        home::home_dir().unwrap().display(),
        ".cache/aur",
        package_name
    );

    check_dependency("git");

    // cd into package, pull changes
    let exit_status = Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg("master")
        .current_dir(package_path)
        .output()
        .unwrap();

    if exit_status.status.code().unwrap() != 0 {
        Err(String::from_utf8_lossy(&exit_status.stderr).into())
    } else {
        Ok(())
    }
}

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
