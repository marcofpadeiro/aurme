use reqwest;
use std::process::Command;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";

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

    if res.contains("id=\"error-page\"") {
        return Ok(false);
    } else {
        Ok(true)
    }
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
    let clone_path: String = format!("{}/{}", home::home_dir().unwrap().display(), ".cache/aur");
    let package_path: String = format!("{}/{}", clone_path, package_name);

    let git_check = Command::new("git").arg("--version").output()?;
    if !git_check.status.success() {
        std::eprintln!("Git is not installed, please install it first");
        std::process::exit(1);
    }

    if !std::path::Path::new(clone_path.as_str()).exists() {
        std::fs::create_dir(clone_path.as_str()).expect("Failed to create cache directory");
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
        // maybe call the install function here
        Ok(())
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

pub async fn check_for_updates(
    package: &Package,
) -> Result<(bool, String), Box<dyn std::error::Error>> {
    let url = format!("{}/packages/{}", AUR_URL, package.get_name());
    let res = fetch(&url).await.unwrap();

    let re = regex::Regex::new(r"<h2>Package Details: [^<]+ (.+)</h2>").unwrap();

    if let Some(captures) = re.captures(&res) {
        if let Some(version) = captures.get(1) {
            Ok((
                version.as_str() != package.get_version(),
                version.as_str().to_owned(),
            ))
        } else {
            Err("No version found".into())
        }
    } else {
        Err("Couldn't get most recent version".into())
    }
}
