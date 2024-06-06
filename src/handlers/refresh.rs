use async_trait::async_trait;

use super::handler::CommandHandler;

pub struct RefreshHandler;

#[async_trait]
impl CommandHandler for RefreshHandler {
    async fn handle(&self, _matches: &clap::ArgMatches, _config: &crate::config::Config) {
        todo!("refresh handler");
    }
}
