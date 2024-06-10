use aurme::expand_path;
use flate2::read::GzDecoder;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use crate::{
    config::CACHE_PATH,
    helpers::{name_to_key, AUR_URL},
    package::Package,
    theme::{colorize, Type},
};

pub const DB_NAME: &str = "packages-meta-ext-v1.json";
pub const NON_ALPHA: &str = "non_alpha";

pub async fn download_database() -> Result<String, Box<dyn std::error::Error>> {
    println!(
        "{}",
        colorize(Type::Info, "Synchronising package database...")
    );
    let db_path = expand_path(CACHE_PATH);
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

    let packages: Vec<Package> = serde_json::from_str(&json_str)?;
    let mut alphabet_map: HashMap<String, Vec<Package>> = HashMap::new();

    packages.iter().for_each(|package| {
        alphabet_map
            .entry(name_to_key(&package.name))
            .or_insert(Vec::new())
            .push(package.clone());
    });

    let alphabet_json = serde_json::to_string_pretty(&alphabet_map)?;
    let mut json_file = File::create(&json_path)?;
    json_file.write_all(alphabet_json.as_bytes())?;
    println!("  {}", colorize(Type::Success, "Database is up to date"));

    Ok(json_str)
}

pub async fn read_database(
) -> Result<HashMap<String, Vec<Package>>, Box<dyn std::error::Error>> {
    let db_path = expand_path(CACHE_PATH).join(DB_NAME);

    let json: String = if !db_path.exists() {
        download_database().await?
    } else {
        std::fs::read_to_string(db_path)?
    };

    Ok(serde_json::from_str(&json)?)
}
