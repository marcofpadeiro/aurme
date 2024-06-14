use crate::config::{expand_path, PACKAGES_CACHE_PATH};
use crate::package::Package;
use crate::theme::{colorize, Type};
use std::error::Error;
use std::path::PathBuf;

pub fn handle_clean(packages: &[&str]) -> Result<(), Box<dyn Error>> {
    let cache_path: PathBuf = expand_path(PACKAGES_CACHE_PATH);

    if !cache_path.exists() {
        return Ok(());
    }

    if packages.len() == 0 {
        remove_all_cache(&cache_path)?;
        return Ok(());
    }

    let mut packages_deleted: Vec<&str> = Vec::new();
    for package in packages.iter() {
        if check_if_cache_exists(&cache_path, package) {
            remove(package)?;
            packages_deleted.push(package);
        }
    }

    if packages_deleted.len() == 0 {
        return Ok(());
    }

    println!(
        "{} cleared cache of packages: {:?}",
        colorize(Type::Success, "Successfully"),
        packages_deleted
    );

    Ok(())
}

pub fn remove_cache(packages: Vec<&Package>) -> Result<(), Box<dyn Error>> {
    for package in packages.iter() {
        remove(&package.name)?;
    }
    Ok(())
}

fn remove_all_cache(cache_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    std::fs::remove_dir_all(&cache_path)?;
    println!(
        "{} cleared cache of all packages",
        colorize(Type::Success, "Successfully"),
    );
    Ok(())
}

fn remove(package: &str) -> Result<(), Box<dyn Error>> {
    let cache_path: PathBuf = expand_path(PACKAGES_CACHE_PATH);
    let package_path = cache_path.join(package);
    std::fs::remove_file(format!("{}.tar.gz", package_path.display()))?;
    std::fs::remove_dir_all(package_path)?;
    Ok(())
}

fn check_if_cache_exists(cache_path: &PathBuf, package: &str) -> bool {
    let tar_name = format!("{}.tar.gz", package);
    let tar_path = cache_path.join(tar_name);
    let package_path = cache_path.join(package);

    package_path.exists() || tar_path.exists()
}
