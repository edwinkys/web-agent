use super::*;
use std::time::Duration;
use tokio::time::sleep;

pub struct PlanningAgent {
    instruction: Message,
    model: Arc<dyn LanguageModel>,
}

impl PlanningAgent {
    /// Creates a new instance of the planning agent.
    pub fn new(model: &Arc<dyn LanguageModel>) -> Self {
        let filepath = Path::new("static/instructions/planning.txt");
        let instruction = Message {
            role: Role::Assistant,
            content: read_instruction(&filepath),
        };

        Self {
            model: model.clone(),
            instruction,
        }
    }

    /// Generates a response based on the session state.
    pub async fn respond(&self, state: &mut SessionState) -> Result<Message> {
        const MAX_ATTEMPTS: u8 = 3;
        let mut count = 0;

        // This is a placeholder for the result.
        // It will be updated with the actual result by the infer method.
        let mut result = Err(anyhow!("Failed to generate a response."));

        while count < MAX_ATTEMPTS {
            result = self.model.infer(&self.instruction, &state.history).await;
            if let Ok(message) = result {
                state.history.push(message.clone());
                return Ok(message);
            }

            count += 1;
            sleep(Duration::from_secs(3)).await;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Configuration, SessionState};

    #[tokio::test]
    async fn test_respond() {
        let config = Configuration::from_env();
        let agent = PlanningAgent::new(&config.language_model());
        let mut state = SessionState::default();

        state.history.push(Message {
            role: Role::User,
            content: "Can you give me a cupcake recipe? Yes or no only."
                .to_string(),
        });

        let response = agent.respond(&mut state).await.unwrap();
        assert_eq!(response.content, "No.");
    }
}
