use crate::config::Config;
use crate::package::Package;
use crate::theme::colorize;
use crate::theme::Type;
use flate2::read::GzDecoder;
use reqwest;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use tar::Archive;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";
pub const DB_NAME: &str = "packages-meta-ext-v1.json";

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
* helper function to check if package exists @param package_name: the name of the package
* @return true if the package exists, false otherwise
*/
pub fn check_packages_existance(
    package_names: &Vec<&str>,
    packages_db: &Vec<Package>,
) -> Result<(Vec<String>, Vec<Package>), Box<dyn std::error::Error>> {
    let existent_packages: Vec<Package> = packages_db
        .iter()
        .filter(|package| package_names.contains(&package.get_name()))
        .map(|package| package.clone())
        .collect();

    // filter out the packages that don't exist
    let non_existent = package_names
        .iter()
        .filter(|package_name| {
            !existent_packages
                .iter()
                .any(|package| package.get_name() == **package_name)
        })
        .map(|package_name| package_name.to_string())
        .collect();

    Ok((non_existent, existent_packages))
}

/**
* Clone a package from the AUR
* @param package_name: the name of the package
*/
pub fn clone_package(
    package: &Package,
    user_config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache_path: String = format!(
        "{}/{}",
        home::home_dir().unwrap().display(),
        user_config.get_cache_path()
    );
    let package_path: String = format!("{}/{}", cache_path, package.get_name());

    if !std::path::Path::new(cache_path.as_str()).exists() {
        std::fs::create_dir_all(cache_path.as_str()).expect("Failed to create cache directory");
    }

    //// if dir with package name already exists, delete it
    if std::path::Path::new(package_path.as_str()).exists() {
        std::fs::remove_dir_all(package_path.as_str()).expect("Failed to remove old package");
    }

    check_dependency("curl");
    // specify the directory to clone the package to
    Command::new("curl")
        .arg("-L")
        .arg("-O")
        .arg("--output-dir")
        .arg(cache_path.clone())
        .arg(package.get_url_path())
        .output()
        .unwrap();

    let file = File::open(package_path.clone() + ".tar.gz")?;
    let mut archive = Archive::new(GzDecoder::new(file));
    archive.unpack(cache_path.clone())?;

    println!(
        "{} cloned package {}",
        colorize(Type::Success, "Successfully"),
        package.get_name()
    );
    match makepkg(&package.get_name(), &user_config) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn sync_db(db_path: &str) {
    // download the database
    let url = format!("{}/{}.gz", AUR_URL, DB_NAME);
    let res = reqwest::get(&url).await.unwrap();
    if let Err(_) = std::fs::metadata(db_path) {
        std::fs::create_dir_all(db_path).unwrap();
    }
    let db_path = format!("{}/{}.gz", db_path, DB_NAME);
    let mut file = File::create(db_path.clone()).unwrap();
    let content = res.bytes().await.unwrap();
    file.write_all(&content).unwrap();

    // extract the database
    let file = File::open(&db_path).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str).unwrap();
    let json_path = db_path.trim_end_matches(".gz");
    let mut json_file = File::create(&json_path).unwrap();
    json_file.write_all(json_str.as_bytes()).unwrap();
    std::fs::remove_file(db_path).unwrap();
}

pub async fn get_db(user_config: &Config) -> Vec<Package> {
    let db_location = format!(
        "{}/{}",
        home::home_dir().unwrap().display(),
        user_config.get_db_path(),
    );
    let db_path = format!("{}/{}", db_location, DB_NAME);
    let db_path = std::path::Path::new(&db_path);
    if let Err(_) = std::fs::metadata(db_path) {
        sync_db(&db_location).await;
    }

    serde_json::from_str(&std::fs::read_to_string(db_path).unwrap()).unwrap()
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

pub async fn check_for_updates(packages: Vec<Package>) -> Vec<(Package, String)> {
    let mut url = format!("{}/rpc/?v=5&type=info", AUR_URL);
    packages.iter().for_each(|package| {
        url = format!("{}&arg[]={}", url, package.get_name());
    });

    let res = fetch(&url).await.unwrap();
    let json: Value = serde_json::from_str(&res).unwrap();
    let rpc_packages: Vec<Package> = json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|result| serde_json::from_value::<Package>(result.clone()).unwrap())
        .collect();

    packages
        .iter()
        .zip(rpc_packages.iter())
        .filter(|(package, rpc_package)| package.get_version() != rpc_package.get_version())
        .map(|(package, rpc_package)| (rpc_package.clone(), package.get_version().to_owned()))
        .collect()
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
pub fn makepkg(package_name: &str, user_config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("  {} {}...", colorize(Type::Info, "Building"), package_name);
    let package_path: String = format!(
        "{}/{}/{}",
        home::home_dir().unwrap().display(),
        user_config.get_cache_path(),
        package_name
    );

    check_dependency("fakeroot");
    check_dependency("make");

    let mut no_confirm = String::from("--noconfirm");
    if !user_config.get_no_confirm() {
        no_confirm = String::from("");
    }

    let (stdout, stderr) = user_config.get_verbose_config();

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
    if !user_config.get_keep_cache() {
        std::fs::remove_file(package_path.clone() + ".tar.gz").unwrap();
        std::fs::remove_dir_all(package_path).unwrap();
    }

    if exit_status.status.code().unwrap() != 0 {
        if user_config.get_verbose() != "quiet" {
            return Err("Check Above Output".into());
        }
        return Err(String::from_utf8_lossy(&exit_status.stderr).into());
    }
    Ok(())
}
