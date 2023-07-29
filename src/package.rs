use crate::helpers::AUR_URL;
use serde::{Deserialize, Serialize};

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
    pub fn new(
        name: String,
        description: Option<String>,
        version: Option<String>,
        popularity: Option<f32>,
        url: Option<String>,
    ) -> Package {
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
            },
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

    pub fn get_url_path(&self) -> String {
        AUR_URL.to_owned() + &self.url_path
    }
}
