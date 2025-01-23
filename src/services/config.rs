use super::*;
use crate::inferences::*;
use std::env;
use std::str::FromStr;
use url::Url;

/// LLM provider name.
///
/// We use the LLM product name instead of the company name to allow for
/// easier configuration and backward compatibility with another LLM family
/// from the same company.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum LLMProvider {
    GPT,
}

impl FromStr for LLMProvider {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        match value.to_uppercase().as_str() {
            "GPT" => Ok(LLMProvider::GPT),
            _ => Err(anyhow!("Unsupported language model provider.")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub llm_provider: LLMProvider,
    pub database_url: Url,
    pub database_pool: u8,
}

impl Configuration {
    /// Loads the configuration from the environment variables.
    pub fn from_env() -> Self {
        let database = utils::get_env("DB_URL")
            .parse::<Url>()
            .expect("Please provide a valid DB_URL value");

        // Get a pool size from the environment variable or
        // use the default value of 4.
        let pool = match env::var("DB_POOL").ok() {
            None => 4,
            Some(pool) => pool
                .parse::<u8>()
                .expect("Please provide a valid DB_POOL value"),
        };

        let provider = match env::var("LLM_PROVIDER").ok() {
            None => LLMProvider::GPT,
            Some(provider) => provider
                .parse::<LLMProvider>()
                .expect("Please provide a valid LLM_PROVIDER value"),
        };

        tracing::info!("Using the LLM provider: {provider:?}");

        Self {
            llm_provider: provider,
            database_url: database,
            database_pool: pool,
        }
    }

    /// Returns a callable trait object of the language model provider.
    pub fn language_model(&self) -> Arc<dyn LanguageModel> {
        match self.llm_provider {
            LLMProvider::GPT => Arc::new(InferenceGPT::new()),
        }
    }
}

#[cfg(test)]
impl Default for Configuration {
    fn default() -> Self {
        let database = "postgres://postgres:password@localhost:5432/postgres";

        Self {
            llm_provider: LLMProvider::GPT,
            database_url: Url::parse(database).unwrap(),
            database_pool: 1,
        }
    }
}
