use async_trait::async_trait;

#[async_trait]
pub trait CommandHandler {
    async fn handle(&self, matches: &clap::ArgMatches, config: &crate::config::Config);
}
