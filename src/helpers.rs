use reqwest;
use std::process::Command;

// variables and structs for ease of use
pub const AUR_URL: &str = "https://aur.archlinux.org";

pub struct Package {
    name: String,
    description: String,
}

impl Package {
    pub fn new(name: String, description: String) -> Package {
        Package { name, description }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
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
