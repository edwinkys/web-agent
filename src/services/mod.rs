mod config;

pub use config::Configuration;

use crate::inferences::{Message, Role};
use crate::subagents::PlanningAgent;
use crate::utils;
use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message as WSMessage;

#[derive(Debug, Default)]
pub struct SessionState {
    pub history: Vec<Message>,
}

pub struct Linnear {
    config: Configuration,
    database: PgPool,
    planner: PlanningAgent,
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
        let model = config.language_model();

        Self {
            config: config.clone(),
            database: pool,
            planner: PlanningAgent::new(&model),
        }
    }

    /// Processes the incoming message from the WebSocket connection.
    /// - state: Session state for the current connection
    /// - message: WebSocket message from the client
    pub async fn process_message(
        &self,
        state: &mut SessionState,
        message: &WSMessage,
    ) -> WSMessage {
        let content = match message {
            WSMessage::Text(text) => text.to_string(),
            _ => return "ERROR: We only support text-type messages.".into(),
        };

        // Add the most recent message to the history.
        state.history.push(Message {
            role: Role::User,
            content,
        });

        let response = match self.planner.respond(state).await {
            Ok(response) => response,
            Err(e) => {
                let message = "Failed to generate a response";
                tracing::error!("{message}: {e}");
                return format!("ERROR: {message}.").into();
            },
        };

        response.content.into()
    }
}
