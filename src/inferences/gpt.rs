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
        ["gpt-4o", "gpt-4o-mini", "o1", "o1-mini"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

#[async_trait]
impl LanguageModel for InferenceGPT {
    async fn infer(&self, messages: &[Message]) -> Result<Message> {
        let mut body = json!({ "model": self.model, "messages": [] });
        for message in messages {
            body["messages"].as_array_mut().unwrap().push(json!({
                "role": match message.role {
                    Role::System => "developer",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": message.content,
            }));
        }

        let response = Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(self.secret.as_str())
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let response_body: Value = response.json::<Value>().await?;

        Ok(Message {
            role: Role::Assistant,
            content: response_body["choices"][0]["message"]["content"]
                .as_str()
                .unwrap()
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_infer() {
        dotenv().ok();

        let inference = InferenceGPT::new();
        let messages = vec![Message {
            role: Role::System,
            content: "This is a test. So, only say: Hello".to_string(),
        }];

        let message = inference.infer(&messages).await.unwrap();
        assert_eq!(&message.content, "Hello");
    }
}
