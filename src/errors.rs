// Purpose: Handle errors
use std::process;

pub fn handle_error(err: &str) {
    println!("Error: {}", err);
    match err {
        "usage" => handle_usage(),
        "no packages specified" => handle_no_packages_specified(),
        _ => handle_usage(),
    }
    process::exit(1);
}

fn handle_usage() {
    println!("Usage: aur <flags> <values>
    Flags:
        -S <package>    Install package
        -Ss <package>   Search for package
        -Syu            Update aur packages");
}

fn handle_no_packages_specified() {
    println!("Usage: aur -S[s] <values>");
}
