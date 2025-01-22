mod gpt;

pub use gpt::InferenceGPT;

use crate::utils;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

// When we add more language model providers, we want them to implement these
// traits. The model provider trait helps the service to initialize the
// provider while the language model trait helps the service to call the
// inference method of the provider.

pub trait ModelProvider {
    // Initializes the provider with the necessary data from the environment.
    fn new() -> Self;

    // Lists the supported models for the provider.
    fn models() -> Vec<String>;

    // Returns the model name from the environment or the default model.
    fn model() -> String {
        // Get the model name from the environment variable.
        // If it is not set, use the first model from the list.
        let model = match env::var("LLM_MODEL_NAME").ok() {
            Some(model) => model,
            None => {
                let default_model = Self::models()[0].to_string();
                tracing::warn!("No LLM model name specified...");
                tracing::info!("Using the default model: {default_model}");
                return default_model;
            },
        };

        // If the model is provided but not supported, panic.
        if !Self::models().contains(&model) {
            tracing::error!("The specified LLM model is not supported");
            panic!("Please provide a supported model from a provider.")
        }

        model
    }
}

#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// Infers the next message based on the provided messages.
    async fn infer(&self, messages: &[Message]) -> Result<Message>;
}
