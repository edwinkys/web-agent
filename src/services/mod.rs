use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use url::Url;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub database_url: Url,
    pub database_pool: u8,
}

impl Configuration {
    /// Loads the configuration from the environment variables.
    pub fn from_env() -> Self {
        let database = Self::get_env("DB_URL")
            .parse::<Url>()
            .expect("Please provide a valid DB_URL value");

        // Get a pool size from the environment variable or
        // use the default value of 4.
        let pool = match env::var("DB_POOL").ok() {
            Some(pool) => pool.parse().expect("Invalid DB_POOL value"),
            None => 4,
        };

        Self {
            database_url: database,
            database_pool: pool,
        }
    }

    /// Retrieves an environment variable or panics if it is not set.
    pub fn get_env(key: &str) -> String {
        let error = format!("Please set the {key} environment variable");
        env::var(key).expect(&error)
    }
}

#[cfg(test)]
impl Default for Configuration {
    fn default() -> Self {
        let database = "postgres://postgres:password@localhost:5432/postgres";

        Self {
            database_url: Url::parse(database).unwrap(),
            database_pool: 1,
        }
    }
}

pub struct Linnear {
    config: Configuration,
    database: PgPool,
}

impl Linnear {
    /// Creates a new instance of the service.
    pub async fn new(config: &Configuration) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(config.database_pool as u32)
            .connect(config.database_url.as_str())
            .await
            .expect("Failed to connect to the Postgres database");

        Self {
            config: config.clone(),
            database: pool,
        }
    }
}
