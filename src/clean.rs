use crate::config::expand_path;
use crate::theme::{colorize, Type};
use crate::Config;
use std::error::Error;
use std::path::PathBuf;

pub fn handle_clean(packages: &[&str], config: &Config) -> Result<(), Box<dyn Error>> {
    let cache_path: PathBuf = expand_path(crate::config::PACKAGES_CACHE_PATH);
    if !cache_path.exists() {
        std::fs::create_dir_all(cache_path).unwrap();
        println!(
            "{} cleared cache",
            colorize(Type::Success, "Successfully cleared cache"),
        );
        return Ok(());
    }

    if packages.len() == 0 {
        std::fs::remove_dir_all(&cache_path).unwrap();
        println!("{} cleared cache", colorize(Type::Success, "Successfully"),);
        return Ok(());
    }

    let mut packages_deleted: Vec<&str> = Vec::new();
    let mut packages_not_found: Vec<&str> = Vec::new();
    packages.iter().for_each(|package| {
        let package_path = cache_path.join(package);

        if !package_path.exists() {
            packages_not_found.push(package);
            return;
        }
        std::fs::remove_file(format!("{}.tar.gz", package_path.display())).unwrap();
        std::fs::remove_dir_all(package_path).unwrap();
        packages_deleted.push(package);
    });

    if packages_not_found.len() != 0 {
        println!(
            "{} no entries found for:",
            colorize(Type::Warning, "Warning"),
        );
        packages_not_found.iter().for_each(|package| {
            println!("  {}", package);
        });
    }
    if packages_deleted.len() == 0 {
        return Ok(());
    }

    println!(
        "{} cleared cache of packages:",
        colorize(Type::Success, "Successfully")
    );
    packages_deleted.iter().for_each(|package| {
        println!("  {}", package);
    });

    Ok(())
}
