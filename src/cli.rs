use std::{
    error::Error,
    io::{stdin, stdout, Write},
};

use crate::{
    package::Package,
    theme::{colorize, Type},
};

pub fn print_outdated_packages(outdated: &[(&Package, &Package)]) {
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
}

pub fn get_yes_no(question: &str) -> bool {
    print!("\n{} [Y/n]:", question);
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    let input = input.trim();

    input == "" || input == "y" || input == "Y"
}

pub fn print_top_packages(packages: &[&Package]) {
    packages.iter().rev().enumerate().for_each(|(i, package)| {
        println!(
            "\n{} {}\n  {}",
            colorize(Type::Info, format!("{} ┃", packages.len() - i).as_str()),
            colorize(Type::Header, package.name.as_str()),
            package.get_description()
        );
    });
}

pub fn get_value_from_range<'a>(
    message: &str,
    min: usize,
    max: usize,
) -> Result<Option<usize>, Box<dyn Error>> {
    print!("\n{} ({}-{}) or (q)uit: ", message, min, max);
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    input = input.trim().to_string();

    if input == "q" || input == "quit" || input == "" {
        return Ok(None);
    }

    let parsed_input = input.parse::<usize>().unwrap_or(0);

    if parsed_input < min || parsed_input > max {
        return Err(format!(
            "Input out of range must be between [{}; {}]",
            min,
            max
        )
        .into());
    }

    Ok(Some(parsed_input))
}
