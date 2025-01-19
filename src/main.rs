#![warn(unused_qualifications)]
#![warn(missing_debug_implementations)]

mod services;

use clap::Command;
use dotenv::dotenv;
use semver::Version;
use services::{Configuration, Linnear};
use sqlx::{PgPool, Postgres};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() {
    dotenv().ok();

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

async fn migrate_handler() {
    tracing_subscriber::fmt::init();
    tracing::info!("Migrating the database schema...");

    let config = Configuration::from_env();
    let pool = PgPool::connect(config.database_url.as_str())
        .await
        .expect("Failed to connect to the Postgres database");

    let target_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let current_version = get_version(&pool).await;

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

    for migration in migrations.iter() {
        let filepath = migrations_dir.join(format!("{migration}.sql"));
        let script = fs::read_to_string(filepath)
            .expect("Failed to read the migration script");

        tracing::info!("Applying the migration script:\n\n{script}");
        sqlx::raw_sql(&script)
            .execute(&pool)
            .await
            .expect("Failed to execute the migration script");

        sqlx::query("UPDATE version SET version = $1")
            .bind(migration.to_string())
            .execute(&pool)
            .await
            .expect("Failed to update the schema version");

        tracing::info!("Migrated the database schema to version {migration}");
    }
}

/// Retrieves the current schema version from the database.
/// - pool: Postgres connection pool.
async fn get_version(pool: &PgPool) -> Version {
    sqlx::query_scalar::<Postgres, String>("SELECT version from version")
        .fetch_one(pool)
        .await
        .unwrap_or(String::from("0.0.0"))
        .parse::<Version>()
        .expect("Failed to parse semantic versioning")
}
