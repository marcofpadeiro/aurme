use aurme::expand_path;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{Read, Write},
};

use crate::{config::Config, helpers::AUR_URL, package::Package};

pub const DB_NAME: &str = "packages-meta-ext-v1.json";

pub async fn download_database(config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let db_path = expand_path(config.db_path.as_str());
    let json_path = db_path.join(DB_NAME);

    if let Err(_) = std::fs::metadata(&db_path) {
        std::fs::create_dir_all(&db_path)?;
    }

    let url = format!("{}/{}.gz", AUR_URL, DB_NAME);
    let res = reqwest::get(&url).await?;
    let content = res.bytes().await?;

    let mut decoder = GzDecoder::new(&content[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str)?;

    let mut json_file = File::create(&json_path)?;
    json_file.write_all(json_str.as_bytes())?;

    Ok(json_str)
}

pub async fn read_database(config: &Config) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let db_path = expand_path(config.db_path.as_str()).join(DB_NAME);

    let json: String = if !db_path.exists() {
        download_database(config).await?
    } else {
        std::fs::read_to_string(db_path)?
    };

    Ok(serde_json::from_str(&json)?)
}
