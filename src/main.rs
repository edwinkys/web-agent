#![warn(unused_qualifications)]
#![warn(missing_debug_implementations)]

mod inferences;
mod services;
mod utils;

use clap::{arg, ArgMatches, Command};
use dotenv::dotenv;
use semver::Version;
use services::{Configuration, Linnear};
use sqlx::{Connection, PgConnection, Postgres};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use url::Url;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let start_command = Command::new("start")
        .arg(arg!(-d --debug "Enable detailed traces for debugging"))
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
    let debug = *args.get_one::<bool>("debug").unwrap_or(&false);
    if debug {
        tracing_subscriber::fmt::init();
        tracing::info!("Running the service in debugging mode...");
    }

    let config = Configuration::from_env();

    // Make sure the database schema is up-to-date.
    let package_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let schema_version = get_version(&config.database_url).await;
    if schema_version != package_version {
        panic!("Please run the migrate command to update the schema.");
    }

    let linnear = Linnear::new(&config).await;
    let service = Arc::new(linnear);
}

async fn migrate_handler() {
    tracing_subscriber::fmt::init();
    tracing::info!("Migrating the database schema...");

    let config = Configuration::from_env();
    let target_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let current_version = get_version(&config.database_url).await;

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

/// Retrieves the current schema version from the database.
/// - url: Postgres database URL
async fn get_version(url: &Url) -> Version {
    let mut conn = PgConnection::connect(url.as_str())
        .await
        .expect("Failed to connect to the Postgres database");

    sqlx::query_scalar::<Postgres, String>("SELECT version from version")
        .fetch_one(&mut conn)
        .await
        .unwrap_or(String::from("0.0.0"))
        .parse::<Version>()
        .expect("Failed to parse semantic versioning")
}
