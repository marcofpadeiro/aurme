use crate::config::expand_path;
use crate::install::AUR_URL;
use crate::name_to_key;
use flate2::read::GzDecoder;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    process::Command,
};

use crate::{
    config::CACHE_PATH,
    package::Package,
    theme::{colorize, Type},
};

pub const DB_PATH: &str = "~/.cache/aurme/packages-meta.json";

pub async fn download_database() -> Result<HashMap<String, Vec<Package>>, Box<dyn std::error::Error>>
{
    let cache_path = expand_path(CACHE_PATH);
    let db_path = expand_path(DB_PATH);

    println!(
        "{}",
        colorize(Type::Info, "Synchronising package database...")
    );

    if let Err(_) = std::fs::metadata(&cache_path) {
        std::fs::create_dir_all(&cache_path)?;
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

pub fn read_database() -> Result<HashMap<String, Vec<Package>>, Box<dyn std::error::Error>> {
    let db_path = expand_path(DB_PATH);

    let json: String = std::fs::read_to_string(db_path)?;

    Ok(serde_json::from_str(&json)?)
}

// temp
pub fn get_installed_packages() -> Result<Vec<Package>, Box<dyn std::error::Error>> {
    let installed_packages_output = Command::new("pacman")
        .arg("-Qm")
        .output()
        .expect("Failed to get installed packages");

    let installed_packages_str = std::str::from_utf8(&installed_packages_output.stdout)?;

    let package_lines: Vec<&str> = installed_packages_str.trim().split('\n').collect();

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
