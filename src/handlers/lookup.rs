use async_trait::async_trait;
use aurme::expand_path;

use crate::database::{read_database, DB_PATH};

use super::handler::CommandHandler;

pub struct LookupHandler;

#[async_trait]
impl CommandHandler for LookupHandler {
    #[allow(unused)]
    async fn handle(&self, matches: &clap::ArgMatches, config: &crate::config::Config) {
        let db_path = expand_path(DB_PATH);
        let word = matches.get_one::<String>("word").unwrap().to_lowercase();
        let first_letter = word.chars().next().unwrap().to_string();

        let package_map = read_database().await.unwrap();

        if let Some(packages) = package_map.get(&first_letter) {
            for package in packages {
                if package.name.starts_with(&word) {
                    println!("{}", package.name);
                }
            }
        }
    }
}
