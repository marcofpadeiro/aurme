use reqwest;

pub const AUR_URL: &str = "https://aur.archlinux.org";

// helper function to get the html of a page
pub async fn fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = reqwest::get(url).await?.text().await?;
    Ok(res)
}

pub struct Package {
    name: String,
    description: String,
    download: String,
}

impl Package {
    pub fn new(name: String, description: String) -> Package {
        Package {
            name: name.clone(),
            description,
            download: format!("{}/{}.git", AUR_URL, name),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_download(&self) -> &str {
        &self.download
    }
}
