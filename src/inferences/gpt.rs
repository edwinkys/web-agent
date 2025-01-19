use super::*;

pub struct InferenceGPT {
    secret: String,
    model: String,
}

impl ModelProvider for InferenceGPT {
    fn new() -> Self {
        Self {
            secret: utils::get_env("OPENAI_API_KEY"),
            model: Self::model(),
        }
    }

    fn models() -> Vec<String> {
        vec!["gpt-4o", "gpt-4o-mini", "o1", "o1-mini"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl LanguageModel for InferenceGPT {
    fn infer(&self, messages: &[Message]) -> Result<Message> {
        unimplemented!()
    }
}
