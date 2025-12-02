use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI-compatible chat completion request
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[allow(dead_code)]
    id: Option<String>,
    choices: Vec<Choice>,
    #[allow(dead_code)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    #[allow(dead_code)]
    index: Option<usize>,
    message: ChatMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// LLM Client for OpenAI-compatible APIs
pub struct LlmClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl LlmClient {
    /// Create a new LLM client with the given configuration
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for long generations
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    /// Create a new LLM client from the app's configuration
    pub fn from_config() -> Result<Self, String> {
        let (_, base_url, model, api_key) = super::config_service::get_effective_config()?;

        if api_key.is_empty() {
            return Err("No API key configured. Please add your API key in Settings.".to_string());
        }

        Ok(Self::new(&base_url, &api_key, &model))
    }

    /// Send a chat completion request
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
    ) -> Result<String, String> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature,
        };

        // Build the full URL - append /chat/completions if base_url doesn't already include it
        let url = if self.base_url.contains("/chat/completions") {
            self.base_url.clone()
        } else {
            format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, error_text));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response content".to_string())
    }

    /// Helper to create a system message
    pub fn system_message(content: &str) -> ChatMessage {
        ChatMessage {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    /// Helper to create a user message
    pub fn user_message(content: &str) -> ChatMessage {
        ChatMessage {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    /// Helper to create an assistant message
    pub fn assistant_message(content: &str) -> ChatMessage {
        ChatMessage {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}
