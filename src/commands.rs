use crate::errors;

// Purpose: Handle the commands passed to the program.
pub fn handle_install(values: Vec<String>) {
    if values.len() == 0 {
        errors::handle_error("no packages specified");
    }

    println!("Installing: {:?}", values);
}

pub fn handle_search(query: String) {
    println!("Searching: {:?}", query);
}

pub fn handle_update() {
    println!("Updating...");
}
