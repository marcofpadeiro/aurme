use crate::errors;
use crate::helpers;
use crate::helpers::Package;
use crate::helpers::AUR_URL;
use select::document::Document;

// Purpose: Handle the commands passed to the program.
pub fn handle_install(values: Vec<String>) {
    if values.len() == 0 {
        errors::handle_error("no packages specified");
    }

    println!("Installing: {:?}", values);
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

    // print packages
    // TODO: ask if user wants to install any of the packages 1-10 or (q)uit
    for (i, package) in packages.iter().enumerate() {
        println!(
            "\n{}: {}\n  {}",
            i + 1,
            package.get_name(),
            package.get_description()
        );
    }
}

pub fn handle_update() {
    println!("Updating...");
}
