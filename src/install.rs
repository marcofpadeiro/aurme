use crate::config::expand_path;
use crate::config::PACKAGES_CACHE_PATH;
use std::{
    error::Error,
    io::{stdin, stdout, Write},
};

use crate::{
    config::Config,
    database::{get_installed_packages, read_database},
    name_to_key,
    package::Package,
    query::check_packages_existance,
    theme::{colorize, Type},
};

pub const AUR_URL: &str = "https://aur.archlinux.org";

pub async fn handle_install(packages: &Vec<&str>, config: &Config) -> Result<(), Box<dyn Error>> {
    let packages_db = read_database()?;

    let existent_packages: Vec<Package>;
    let non_existent_packages: Vec<String> = match check_packages_existance(&packages, &packages_db)
    {
        Ok((non_existent_packages, packages)) => {
            existent_packages = packages;
            non_existent_packages
        }
        Err(err) => {
            println!("{} {}", colorize(Type::Error, "Error:"), err);
            return Ok(());
        }
    };

    if non_existent_packages.len() > 0 {
        println!(
            "{} The following packages do not exist in the AUR:",
            colorize(Type::Error, "\nError:")
        );
        non_existent_packages.iter().for_each(|package| {
            println!("  {}", package);
        });
        return Ok(());
    }

    let cache_path = expand_path(PACKAGES_CACHE_PATH);
    let cache_path = std::path::Path::new(&cache_path);

    // temp
    Ok(())
    // for package in existent_packages.iter() {
    //     match download_package(&package).await {
    //         Ok(_) => {
    //             makepkg(&package.name, &config).unwrap();
    //             println!("{}", colorize(Type::Info, "Package installed"));
    //         }
    //         Err(e) => {
    //             println!("{} {}", colorize(Type::Error, "Error:"), e);
    //             std::fs::remove_dir_all(cache_path.join(&package.name)).unwrap();
    //         }
    //     };
    // }
}

pub async fn handle_sysupgrade(packages: &[&str], config: &Config) -> Result<(), Box<dyn Error>> {
    let packages_db = read_database()?;
    let mut outdated: Vec<(Package, Package)> = Vec::new();
    get_installed_packages()
        .unwrap()
        .iter()
        .for_each(|package| {
            if let Some(packages) = packages_db.get(&name_to_key(&package.name)) {
                let db_package = packages.iter().find(|p| p.name == package.name);
                if let Some(db_package) = db_package {
                    if db_package.version != package.version {
                        outdated.push((package.clone(), db_package.clone()));
                    }
                }
            }
        });

    if outdated.len() == 0 {
        println!("{}", colorize(Type::Header, "System is up to date"));
        return Ok(());
    }

    println!(
        "{}",
        colorize(
            Type::Header,
            format!("Packages ({}) ", outdated.len()).as_str()
        )
    );

    outdated.iter().for_each(|(local, db)| {
        println!(
            "   {} ({} -> {})",
            local.name,
            colorize(Type::Error, &local.version),
            colorize(Type::Success, &db.version),
        );
    });

    print!("\nProceed with update? [Y/n]:");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    if input != "" && input != "y" && input != "Y" {
        println!("{}", colorize(Type::Warning, "Aborting..."));
        return Ok(());
    }

    //temp
    Ok(())

    // for (_, package) in outdated.iter() {
    //     match download_package(&package).await {
    //         Ok(_) => {
    //             eprintln!(
    //                 "{} updated {}",
    //                 colorize(Type::Success, "Successfully"),
    //                 package.name
    //             );
    //             makepkg(package.name.as_str(), &config).unwrap();
    //         }
    //         Err(e) => println!("{} {}", colorize(Type::Error, "Error:"), e),
    //     };
    // }
}
