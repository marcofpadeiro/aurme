use std::env;

mod cmd;
mod errors;
mod commands;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = cmd::Config::build(&args).unwrap_or_else(|err| {
        errors::handle_error(err);
        std::process::exit(1);
    });

    run(config);
}

fn run(config: cmd::Config) {
    cmd::handle_args(config);
}

