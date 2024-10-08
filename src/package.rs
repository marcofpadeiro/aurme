use crate::install::AUR_URL;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Package {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Description", default)]
    pub description: Option<String>,
    #[serde(rename = "Popularity")]
    pub popularity: f32,
    #[serde(rename = "URLPath")]
    pub url_path: String,
    #[serde(rename = "Depends", default)]
    pub depends: Option<Vec<String>>,
}

impl Package {
    pub fn new(
        name: String,
        description: Option<String>,
        version: Option<String>,
        popularity: Option<f32>,
        url: Option<String>,
        depends: Option<Vec<String>>,
    ) -> Package {
        Package {
            name,
            description: match description {
                Some(d) => Some(d),
                None => Some(String::from("No description provided")),
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
            depends: match depends {
                Some(d) => Some(d),
                None => None,
            },
        }
    }

    pub fn get_url_path(&self) -> String {
        AUR_URL.to_owned() + &self.url_path
    }

    pub fn get_description(&self) -> &str {
        match &self.description {
            Some(d) => d,
            None => "No description provided",
        }
    }
}
