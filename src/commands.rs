// File name: commands.rs
// Purpose: Handle the commands passed to the program by parameters.

use crate::errors;
use crate::helpers;
use crate::helpers::check_package_existance;
use crate::helpers::clone_package;
use crate::helpers::CACHE_PATH;
use std::io::{self, Write};

/**
* Handle the install of packages
* @param values: A vector of strings containing the packages to install
*/
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


    let cache_path: String = format!("{}/{}", home::home_dir().unwrap().display(), CACHE_PATH);
    let cache_path = std::path::Path::new(&cache_path);

    std::fs::remove_dir_all(cache_path.join(&package)).unwrap();


}

/**
* Handle the search of packages
* @param query: A string containing the package to search for
*/
pub async fn handle_search(query: String) {
    if query.len() == 0 {
        errors::handle_error("no packages specified");
    }

    let packages = helpers::get_top_packages(&query).await;

    if packages.len() == 0 {
        println!("No packages found");
        return;
    }

    // print packages
    packages.iter().rev().enumerate().for_each(|(i, package)| {
        println!(
            "\n{}: {}\n  {}",
            packages.len() - i,
            package.get_name(),
            package.get_description()
        );
    });

    print!("\nInstall package(s) (1-10) or (q)uit: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    if input == "q" || input == "quit" {
        return;
    }

    let parsed_input: Result<usize, _> = input.parse();

    match parsed_input {
        Ok(i) if i > 0 && i <= packages.len() => match clone_package(packages[i - 1].get_name()) {
            Ok(_) => println!("Package installed"),
            Err(e) => println!("Error: {}", e),
        },
        _ => println!("Invalid input or package out of range"),
    }
}

/**
* Handle the update of packages
* @param values: A vector of strings containing the packages to update
*/
pub async fn handle_update(values: Vec<String>) {
    println!("Checking for updates...");

    let packages_look_for_updates = if values.len() > 0 {
        match helpers::check_if_packages_installed(values) {
            Ok(packages) => packages,
            Err(packages_missing) => {
                println!("The following packages are not installed:");
                for package in packages_missing.iter() {
                    println!("  {}", package);
                }
                println!("Aborting...");
                std::process::exit(1);
            }
        }
    } else {
        helpers::get_installed_packages().expect("Error getting installed packages")
    };

    let packages_need_updates = helpers::check_for_updates_threads(packages_look_for_updates)
        .await
        .expect("Error checking for updates");

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

    packages_need_updates
        .iter()
        .for_each(|(package, _version)| {
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

/**
* Handle the deletion of packages from the cache
* @param values: A vector of strings containing the packages to delete from cache
*/
pub async fn handle_cache_delete(packages: Vec<String>) {
    let cache_path: String = format!("{}/{}", home::home_dir().unwrap().display(), CACHE_PATH);
    let cache_path = std::path::Path::new(&cache_path);

    if !cache_path.exists() {
        println!("Successfully cleared cache");
        std::fs::create_dir_all(cache_path).unwrap();
        return;
    }

    if packages.len() > 0 {
        let mut packages_deleted: Vec<String> = Vec::new();
        for package in packages {
            let package_path = cache_path.join(&package);

            if package_path.exists() {
                std::fs::remove_dir_all(package_path).unwrap();
                packages_deleted.push(package);
            }
        }
        println!(
            "Successfully cleared cache of packages: {:?}",
            packages_deleted
        );
    } else {
        std::fs::read_dir(cache_path).unwrap().for_each(|entry| {
            let path = entry.unwrap().path();

            std::fs::remove_dir_all(path).unwrap();
        });

        println!("Successfully cleared cache");
    }
}
