mod config;

pub use config::Configuration;

use crate::utils;
use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

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

        // This validates if the LLM provider is supported.
        config.language_model();

        Self {
            config: config.clone(),
            database: pool,
        }
    }
}
