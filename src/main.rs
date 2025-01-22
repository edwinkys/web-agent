#![warn(unused_qualifications)]
#![warn(missing_debug_implementations)]

mod inferences;
mod services;
mod subagents;
mod utils;

use clap::{arg, value_parser, ArgMatches, Command};
use dotenv::dotenv;
use futures::{SinkExt, StreamExt};
use semver::Version;
use services::{Configuration, Linnear, SessionState};
use sqlx::{Connection, PgConnection};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let port_arg = arg!(-p --port <port> "Port for the service to listen on")
        .default_value("2505")
        .value_parser(value_parser!(u16));

    let start_command = Command::new("start")
        .arg(port_arg)
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
        Some(("start", args)) => start_handler(args).await,
        Some(("migrate", _)) => migrate_handler().await,
        _ => unreachable!(),
    }
}

async fn start_handler(args: &ArgMatches) {
    // Parse the command-line arguments from Clap.
    let port = *args.get_one::<u16>("port").unwrap();

    let config = Configuration::from_env();

    // Make sure the database schema is up-to-date.
    let package_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let schema_version = utils::get_version(&config.database_url).await;
    if schema_version != package_version {
        panic!("Please run the migrate command to update the schema.");
    }

    let linnear = Linnear::new(&config).await;
    let service = Arc::new(linnear);

    // Create a new WebSocket server listener.
    let addr = format!("[::]:{port}");
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("The server is ready for connections on port {port}");

    while let Ok((stream, _)) = listener.accept().await {
        let service = service.clone();

        tokio::spawn(async move {
            let websocket = accept_async(stream).await.unwrap();
            let (mut writer, mut reader) = websocket.split();
            let (sender, mut receiver) = mpsc::unbounded_channel();

            // Create session state for the connection.
            // Depending on our memory agent, some of the state will be stored
            // in the database for future reference.
            let mut session_state = SessionState::default();

            tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    if writer.send(message).await.is_err() {
                        break;
                    }
                }
            });

            tokio::spawn(async move {
                while let Some(Ok(message)) = reader.next().await {
                    let response = service
                        .process_message(&mut session_state, &message)
                        .await;

                    if sender.send(response).is_err() {
                        break;
                    }
                }
            });
        });
    }
}

async fn migrate_handler() {
    tracing::info!("Migrating the database schema...");

    let config = Configuration::from_env();
    let target_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let current_version = utils::get_version(&config.database_url).await;

    if current_version >= target_version {
        tracing::info!("The database schema is up-to-date");
        return;
    }

    let migrations_dir = Path::new("static").join("migrations");

    // List all the migration files that need to be applied.
    let mut migrations = fs::read_dir(&migrations_dir)
        .expect("Failed to read the migrations directory")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read a directory entry");
            if entry.path().is_dir() {
                return None;
            }

            // Check if the file is a SQL file.
            let _filename = entry.file_name();
            let filename = _filename.to_str().unwrap();
            if !filename.ends_with(".sql") {
                return None;
            }

            let _version = filename.split_at(filename.len() - 4).0;
            let version = Version::parse(_version).ok()?;
            match version <= current_version || version > target_version {
                true => None,
                false => Some(version),
            }
        })
        .collect::<Vec<Version>>();

    migrations.sort_unstable();

    let mut conn = PgConnection::connect(config.database_url.as_str())
        .await
        .expect("Failed to connect to the Postgres database");

    for migration in migrations.iter() {
        let filepath = migrations_dir.join(format!("{migration}.sql"));
        let script = fs::read_to_string(filepath)
            .expect("Failed to read the migration script");

        tracing::info!("Applying the migration script:\n\n{script}");
        sqlx::raw_sql(&script)
            .execute(&mut conn)
            .await
            .expect("Failed to execute the migration script");

        sqlx::query("UPDATE version SET version = $1")
            .bind(migration.to_string())
            .execute(&mut conn)
            .await
            .expect("Failed to update the schema version");

        tracing::info!("Migrated the database schema to version {migration}");
    }
}
