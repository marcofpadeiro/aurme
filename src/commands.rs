use crate::errors;
use crate::helpers;
use crate::helpers::check_package_existance;
use crate::helpers::clone_package;
use crate::package;
use std::io::{self, Write};

// Purpose: Handle the commands passed to the program.
pub async fn handle_install(values: Vec<String>) {
    if values.len() == 0 {
        errors::handle_error("no packages specified");
    }

    let mut non_existent_packages: Vec<&str> = Vec::new();

    for package in values.iter() {
        if let Ok(exists) = check_package_existance(&package).await {
            if !exists {
                non_existent_packages.push(&package);
            }
        }
    }

    if non_existent_packages.len() > 0 {
        println!("The following packages do not exist in the AUR:");
        for package in non_existent_packages.iter() {
            println!("{}", package);
        }
        return;
    }

    values
        .iter()
        .for_each(|package| match clone_package(&package) {
            Ok(_) => println!("Package installed"),
            Err(e) => println!("Error: {}", e),
        });
}

pub async fn handle_search(query: String) {
    let packages = helpers::get_top_packages(&query).await;

    if packages.len() == 0 {
        println!("No packages found");
        return;
    }

    // print packages
    packages.iter().enumerate().for_each(|(i, package)| {
        println!(
            "\n{}: {}\n  {}",
            i + 1,
            package.get_name(),
            package.get_description()
        );
    });

    print!("Install package(s) (1-10) or (q)uit: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    if input == "q" || input == "quit" {
        return;
    }

    let parsed_input: Result<usize, _> = input.parse();

    match parsed_input {
        Ok(i) => {
            if i > 0 && i <= packages.len() {
                match clone_package(packages[i - 1].get_name()) {
                    Ok(_) => println!("Package installed"),
                    Err(e) => println!("Error: {}", e),
                }
            } else {
                println!("Invalid input");
            }
        }
        Err(_) => println!("Invalid input"),
    }
}

pub async fn handle_update() {
    println!("Checking for updates...");
    let installed_packages: Vec<package::Package> =
        helpers::get_installed_packages().expect("Error getting installed packages");

    let mut packages_need_updates: Vec<(&package::Package, String)> = Vec::new(); // Use a reference to the package

    for package in installed_packages.iter() {
        let (needs_update, version) = package
            .check_for_updates()
            .await
            .expect("Error checking for updates");

        if needs_update {
            packages_need_updates.push((package, version));
        }
    }

    if packages_need_updates.len() == 0 {
        println!("No updates available");
        return;
    }

    println!("Packages ({}) ", packages_need_updates.len());
    packages_need_updates.iter().for_each(|(package, version)| {
        println!(
            "   {} ({} -> {})",
            package.get_name(),
            package.get_version(),
            version
        );
    });

    print!("\nProceed with update? [Y/n]:");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    if input != "" && input != "y" && input != "Y" {
        println!("Aborting...");
        return;
    }

    packages_need_updates.iter().for_each(|(package, _)| {
        if package.check_if_package_in_cache() {
            match package.pull_cached_package() {
                Ok(_) => eprintln!("Successfully updated {}", package.get_name()),
                Err(e) => println!("Error: {}", e),
            }
        } else {
            match clone_package(package.get_name()) {
                Ok(_) => eprintln!("Successfully updated {}", package.get_name()),
                Err(e) => println!("Error: {}", e),
            }
        }
    });
}

pub async fn handle_cache_delete() {
    let cache_path: String = format!("{}/{}", home::home_dir().unwrap().display(), ".cache/aur");
    let cache_path = std::path::Path::new(&cache_path);

    // delete every folder in the cache_path
    std::fs::read_dir(cache_path).unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(path).unwrap();
        } else {
            std::fs::remove_file(path).unwrap();
        }
    });

    println!("Successfully cleared cache");
}
