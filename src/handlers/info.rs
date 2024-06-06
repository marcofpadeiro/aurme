use async_trait::async_trait;

use super::handler::CommandHandler;

pub struct InfoHandler;

#[async_trait]
impl CommandHandler for InfoHandler {
    async fn handle(&self, _matches: &clap::ArgMatches, _config: &crate::config::Config) {
        todo!("refresh handler");
    }
}
