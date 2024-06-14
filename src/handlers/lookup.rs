use async_trait::async_trait;

use crate::{database::read_database, helpers::name_to_key};

use super::handler::CommandHandler;

pub struct LookupHandler;

#[async_trait]
impl CommandHandler for LookupHandler {
    #[allow(unused)]
    async fn handle(&self, matches: &clap::ArgMatches, config: &crate::config::Config) {
        let search_term = matches.get_one::<String>("word").unwrap();
        let packages_db = read_database().await.unwrap();

        if let Some(packages) = packages_db.get(&name_to_key(search_term)) {
            packages.iter().for_each(|package| {
                if package.name.starts_with(&*search_term) {
                    println!("{}", package.name);
                }
            });
        }
    }
}
