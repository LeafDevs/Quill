use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use futures::stream::Stream;
use futures::stream::StreamExt;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub modified_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<Model>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let models_response: ModelsResponse = response.json().await?;
            Ok(models_response.models)
        } else {
            Err(anyhow::anyhow!("Failed to fetch models: {}", response.status()))
        }
    }

    pub async fn chat(&self, model_name: String, message: String, system_prompt: &str) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            model: model_name,
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: message,
                }
            ],
            stream: false,
        };

        let response = self.client.post(&url).json(&request).send().await?;
        
        if response.status().is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response.message.content)
        } else {
            Err(anyhow::anyhow!("Failed to get response: {}", response.status()))
        }
    }

    pub async fn chat_stream(&self, model_name: String, messages: Vec<(String, String)>) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let url = format!("{}/api/chat", self.base_url);
        let request_messages: Vec<ChatMessage> = messages
            .into_iter()
            .map(|(role, content)| ChatMessage { role, content })
            .collect();
        let request = ChatRequest {
            model: model_name,
            messages: request_messages,
            stream: true,
        };
        let response = self.client.post(&url).json(&request).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get streaming response: {}", response.status()));
        }
        let stream = response.bytes_stream();
        let mapped = stream.map(|chunk| {
            let chunk = chunk?;
            let s = String::from_utf8_lossy(&chunk).to_string();
            Ok(s)
        });
        Ok(Box::pin(mapped))
    }
} 