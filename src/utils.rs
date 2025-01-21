use semver::Version;
use sqlx::{Connection, PgConnection, Postgres};
use std::env;
use url::Url;

/// Retrieves an environment variable or panics if it is not set.
pub fn get_env(key: &str) -> String {
    let error = format!("Please set the {key} environment variable");
    env::var(key).expect(&error)
}

/// Retrieves the current schema version from the database.
/// - url: Postgres database URL
pub async fn get_version(url: &Url) -> Version {
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
