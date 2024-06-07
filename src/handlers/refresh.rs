use async_trait::async_trait;

use crate::database::download_database;

use super::handler::CommandHandler;

pub struct RefreshHandler;

#[async_trait]
impl CommandHandler for RefreshHandler {
    async fn handle(&self, _matches: &clap::ArgMatches, config: &crate::config::Config) {
        download_database(&config).await.unwrap();
    }
}
