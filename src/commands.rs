use crate::errors;
use crate::helpers;
use crate::helpers::check_n_clone;
use crate::helpers::Package;
use crate::helpers::AUR_URL;
use select::document::Document;
use std::io::{self, Write};

// Purpose: Handle the commands passed to the program.
pub fn handle_install(values: Vec<String>) {
    if values.len() == 0 {
        errors::handle_error("no packages specified");
    }

    println!("Installing: {:?}", values)
}

pub async fn handle_search(query: String) {
    let url = format!("{}/packages/?K={}", AUR_URL, query);
    let res = helpers::fetch(&url).await.unwrap();

    let names: Vec<String> = Document::from(res.as_str())
        .find(select::predicate::Name("tr"))
        .flat_map(|n| n.find(select::predicate::Name("td")))
        .flat_map(|n| n.find(select::predicate::Name("a")))
        .take(10)
        .map(|n| n.text().trim().to_string())
        .collect();

    let descriptions: Vec<String> = Document::from(res.as_str())
        .find(select::predicate::Name("tr"))
        .flat_map(|n| n.find(select::predicate::Name("td")))
        .filter(|n| n.attr("class").unwrap_or("") == "wrap")
        .take(10)
        .map(|n| n.text().trim().to_string())
        .collect();

    let packages: Vec<Package> = names
        .iter()
        .zip(descriptions.iter())
        .map(|(name, description)| Package::new(name.to_string(), description.to_string()))
        .collect();

    if packages.len() == 0 {
        println!("No packages found");
        return;
    }

    // print packages
    for (i, package) in packages.iter().enumerate() {
        println!(
            "\n{}: {}\n  {}",
            i + 1,
            package.get_name(),
            package.get_description()
        );
    }

    print!("Install package(s) (1-10) or (q)uit: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    if input == "q" {
        return;
    }

    let parsed_input: Result<usize, _> = input.parse();

    match parsed_input {
        Ok(i) => {
            if i > 0 && i <= packages.len() {
                match check_n_clone(packages[i - 1].get_name()) {
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

pub fn handle_update() {
    println!("Updating...");
}
