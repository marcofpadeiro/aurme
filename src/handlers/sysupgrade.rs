use async_trait::async_trait;

use super::handler::CommandHandler;

pub struct SysUpgradeHandler;

#[async_trait]
impl CommandHandler for SysUpgradeHandler {
    async fn handle(&self, _matches: &clap::ArgMatches, _config: &crate::config::Config) {
        // crate::commands::handle_sysupgrade().await;
        todo!("sysupgrade handler");
    }
}
