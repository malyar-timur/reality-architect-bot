use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Clone)]
pub struct AiClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl AiClient {
    pub fn new(base_url: &str, api_key: &str, model: &str, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to build reqwest HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    /// Генерация толкования расклада
    pub async fn generate_reading(&self, system_prompt: &str, user_context: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_context.to_string(),
            },
        ];

        let payload = ChatCompletionRequest {
            model: &self.model,
            messages,
            temperature: 0.75,
            max_tokens: 1500,
        };

        info!("Sending AI request to {} for model {}", url, self.model);

        let res = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send HTTP request to AI endpoint")?;

        let status = res.status();
        if !status.is_success() {
            let error_body = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("AI API error (HTTP {}): {}", status, error_body);
            anyhow::bail!("AI API returned HTTP {}: {}", status, error_body);
        }

        let body: ChatCompletionResponse = res.json().await
            .context("Failed to deserialize AI JSON response")?;

        let interpretation = body
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_else(|| "Оракул сохранил молчание... Попробуйте повторить запрос позже.".to_string());

        Ok(interpretation)
    }
}
