#![warn(unused_qualifications)]
#![warn(missing_debug_implementations)]

mod services;

use clap::{ArgMatches, Command};
use services::{Configuration, Linnear};

// List of commands.
// We want to avoid using string literals in the code.
const START_COMMAND: &str = "start";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let command = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interface to setup and manage the service")
        .arg_required_else_help(true)
        .subcommand(start_command())
        .get_matches();

    match command.subcommand() {
        Some((START_COMMAND, args)) => start_handler(args).await,
        _ => unreachable!(),
    }
}

fn start_command() -> Command {
    Command::new(START_COMMAND)
        .alias("run")
        .about("Starts the service")
}

async fn start_handler(_args: &ArgMatches) {
    let config = Configuration::from_env();
    let linnear = Linnear::new(&config).await;
}
