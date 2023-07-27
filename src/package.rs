use crate::helpers::check_dependency;
use crate::helpers::makepkg;
use crate::helpers::CACHE_PATH;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Popularity")]
    popularity: f32,
    #[serde(rename = "URLPath")]
    url_path: String,
}

impl Package {
    pub fn new(name: String, description: Option<String>, version: Option<String>, popularity: Option<f32>, url: Option<String>) -> Package {
        Package {
            name,
            description: match description {
                Some(d) => d,
                None => String::from("No description provided"),
            },
            version: match version {
                Some(v) => v,
                None => String::from("1"),
            },
            popularity: match popularity {
                Some(n) => n,
                None => 0.0,
            },
            url_path: match url {
                Some(u) => u,
                None => String::from(""),
            }
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

    pub fn get_popularity(&self) -> f32 {
        self.popularity
    }

    pub fn get_url_path(&self) -> &str {
        &self.url_path
    }

    /**
     * Check if package is in cache
     */
    pub fn check_if_package_in_cache(&self) -> bool {
        let package_path: String = format!(
            "{}/{}/{}",
            home::home_dir().unwrap().display(),
            CACHE_PATH,
            &self.name
        );

        std::path::Path::new(package_path.as_str()).exists()
    }

    /**
     * if package is in cache, pull changes and updates
     */
    pub fn pull_cached_package(&self) -> Result<(), Box<dyn std::error::Error>> {
        let package_path: String = format!(
            "{}/{}/{}",
            home::home_dir().unwrap().display(),
            CACHE_PATH,
            &self.name
        );

        check_dependency("git");

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
            match makepkg(&self.name) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
    }

    /**
     * check for package updates
     */
    pub async fn check_for_package_updates(self) -> Result<(Package, String), String> {
        let url = format!(
            "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
            &self.get_name()
        );

        let response = reqwest::get(&url).await.expect("asd").text().await;

        let json: Value = serde_json::from_str(&response.expect("Failed to get response")).unwrap();

        let new_version = json
            .get("results")
            .and_then(|results| {
                results.as_array().and_then(|results_array| {
                    results_array
                        .iter()
                        .find(|result| result["Name"] == self.get_name())
                })
            })
            .and_then(|result| result.get("Version"))
            .and_then(|version| version.as_str())
            .ok_or_else(|| "Invalid JSON response or version not found".to_string())?;

        if self.get_version() != new_version.to_string() {
            return Ok((self, new_version.to_string()));
        }
        Err("No update available".to_string())
    }
}
