#![warn(unused_qualifications)]
#![warn(missing_debug_implementations)]

mod services;

use clap::Command;
use services::{Configuration, Linnear};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let start_command = Command::new("start")
        .alias("run")
        .about("Starts the conversational service");

    let migrate_command = Command::new("migrate")
        .about("Migrate the database schema to the latest version");

    let command = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interface to setup and manage the service")
        .arg_required_else_help(true)
        .subcommand(start_command)
        .subcommand(migrate_command)
        .get_matches();

    match command.subcommand() {
        Some(("start", _)) => start_handler().await,
        Some(("migrate", _)) => migrate_handler().await,
        _ => unreachable!(),
    }
}

async fn start_handler() {
    let config = Configuration::from_env();
    let linnear = Linnear::new(&config).await;
}

async fn migrate_handler() {}
