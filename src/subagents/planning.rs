use super::*;

pub struct PlanningAgent {
    model: Arc<dyn LanguageModel>,
}

impl PlanningAgent {
    /// Creates a new instance of the planning agent.
    pub fn new(model: &Arc<dyn LanguageModel>) -> Self {
        Self {
            model: model.clone(),
        }
    }

    /// Generates a response based on the session state.
    pub async fn respond(&self, state: &mut SessionState) -> Message {
        let response = self.model.infer(&state.history).await.unwrap();
        state.history.push(response.clone());
        response
    }
}
