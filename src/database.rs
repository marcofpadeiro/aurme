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

pub const DB_PATH: &str = "~/.cache/aurme/packages-meta.json";
pub const NON_ALPHA: &str = "non_alpha";

pub async fn download_database() -> Result<HashMap<String, Vec<Package>>, Box<dyn std::error::Error>>
{
    println!(
        "{}",
        colorize(Type::Info, "Synchronising package database...")
    );
    let db_path = expand_path(DB_PATH);

    if let Err(_) = std::fs::metadata(&CACHE_PATH) {
        std::fs::create_dir_all(&CACHE_PATH)?;
    }

    let url = format!("{}/packages-meta-ext-v1.json.gz", AUR_URL);
    let res = reqwest::get(&url).await?;
    let content = res.bytes().await?;

    let mut decoder = GzDecoder::new(&content[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str)?;

    let alphabet_map = parse_into_alphabet_map(&json_str);
    let alphabet_json = serde_json::to_string(&alphabet_map)?;

    let mut json_file = File::create(db_path)?;
    json_file.write_all(alphabet_json.as_bytes())?;
    println!("{}", colorize(Type::Success, "Database is updated!"));

    Ok(alphabet_map)
}

fn parse_into_alphabet_map(json_str: &String) -> HashMap<String, Vec<Package>> {
    let packages: Vec<Package> = serde_json::from_str(&json_str)
        .expect("Invalid json database, please fix or remove invalid file");
    let mut alphabet_map: HashMap<String, Vec<Package>> = HashMap::new();

    packages.iter().for_each(|package| {
        alphabet_map
            .entry(name_to_key(&package.name))
            .or_insert(Vec::new())
            .push(package.clone());
    });

    alphabet_map
}

pub async fn read_database() -> Result<HashMap<String, Vec<Package>>, Box<dyn std::error::Error>> {
    let db_path = expand_path(DB_PATH);

    if !db_path.exists() {
        return download_database().await;
    }

    let json: String = std::fs::read_to_string(db_path)?;

    Ok(serde_json::from_str(&json)?)
}
