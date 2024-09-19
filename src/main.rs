use aurme::run;
use clap::Command;
use commands::{build_lookup_command, build_sync_command};
mod commands;

#[tokio::main]
async fn main() {
    let matches = Command::new("aurme")
        .about("AUR wrapper utility")
        .version("0.0.1.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(build_sync_command())
        .subcommand(build_lookup_command())
        .get_matches();

    run(matches).await;
}
