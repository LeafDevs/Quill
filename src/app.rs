use crate::ollama::{OllamaClient, Model};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use futures::StreamExt;
use std::pin::Pin;
use std::env;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    User {
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Assistant {
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    PendingToolCall {
        tool_call: ToolCall,
        original_message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ToolCallResult {
        result: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ToolCallDenied {
        tool_call: ToolCall,
        original_message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCall {
    ReadFile { path: String },
    ReadDirectory { path: String },
}

#[derive(Debug, Clone)]
pub struct PendingToolCall {
    pub tool_call: ToolCall,
    pub original_message: String, // The raw [tool_call: ...] string
}

pub struct App {
    pub models: Vec<Model>,
    pub selected_model_index: usize,
    pub input: String,
    pub input_cursor_position: usize,
    pub messages: VecDeque<Message>,
    pub ollama_client: OllamaClient,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub streaming_message: Option<String>, // For in-progress assistant message
    pub stream: Option<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>>,
    pub working_directory: String,
    pub system_prompt: String,
    pub scroll_offset: usize,
    pub memories: Vec<(String, String)>, // (user, assistant)
    pub chat_history: Vec<(String, String)>, // (role, content)
}

impl App {
    pub async fn new(system_prompt: String) -> Result<Self> {
        let ollama_client = OllamaClient::new();
        let models = ollama_client.list_models().await.unwrap_or_else(|_| {
            vec![Model {
                name: "llama2".to_string(),
                modified_at: chrono::Utc::now(),
                size: 0,
            }]
        });
        let cwd = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Ok(App {
            models,
            selected_model_index: 0,
            input: String::new(),
            input_cursor_position: 0,
            messages: VecDeque::new(),
            ollama_client,
            is_loading: false,
            error_message: None,
            streaming_message: None,
            stream: None,
            working_directory: cwd.display().to_string(),
            system_prompt: system_prompt.clone(),
            scroll_offset: 0,
            memories: Vec::new(),
            chat_history: vec![("system".to_string(), system_prompt)],
        })
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }

    // After each assistant response, push the (user, assistant) pair to memories
    fn add_memory(&mut self, user: &str, assistant: &str) {
        self.memories.push((user.to_string(), assistant.to_string()));
    }

    // After each user/assistant message, push to chat_history
    fn add_user_message(&mut self, content: &str) {
        self.chat_history.push(("user".to_string(), content.to_string()));
    }
    fn add_assistant_message(&mut self, content: &str) {
        self.chat_history.push(("assistant".to_string(), content.to_string()));
    }

    // Build the messages array for the API call
    fn build_messages(&self, user_message: &str) -> Vec<(String, String)> {
        let mut messages = self.chat_history.clone();
        messages.push(("user".to_string(), user_message.to_string()));
        messages
    }

    // When constructing the prompt for a new message, include all memories
    fn build_prompt(&self, user_message: &str) -> String {
        let mut prompt = String::new();
        prompt.push_str(&self.system_prompt);
        prompt.push_str("\n\n");
        for (user, assistant) in &self.memories {
            prompt.push_str(&format!("USER: {}\nASSISTANT: {}\n", user, assistant));
        }
        prompt.push_str(&format!("USER: {}\nASSISTANT:", user_message));
        prompt
    }

    pub async fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        // If the last message is a pending tool call, handle accept/deny
        if let Some(Message::PendingToolCall { tool_call, original_message, .. }) = self.messages.back().cloned() {
            match key.code {
                KeyCode::Right => {
                    // Accept: execute the tool call and send the result as a new user message to the AI
                    let result = self.execute_tool_call(tool_call.clone()).await?;
                    self.messages.pop_back();
                    // Do NOT push a user message with the result; instead, send it as a hidden user message to the AI
                    self.start_message_sending_with_content(result).await?;
                    return Ok(());
                }
                KeyCode::Left => {
                    // Deny: replace the message with a denial
                    self.messages.pop_back();
                    self.messages.push_back(Message::ToolCallDenied {
                        tool_call: tool_call.clone(),
                        original_message: original_message.clone(),
                        timestamp: chrono::Utc::now(),
                    });
                    return Ok(());
                }
                _ => {}
            }
        }

        // Don't process input if we're currently loading
        if self.is_loading {
            return Ok(());
        }

        match key.code {
            KeyCode::Up => {
                if self.selected_model_index > 0 {
                    self.selected_model_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_model_index < self.models.len().saturating_sub(1) {
                    self.selected_model_index += 1;
                }
            }
            KeyCode::Char(c) => {
                // Allow all printable characters except when Control is held
                use crossterm::event::KeyModifiers;
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.insert(self.input_cursor_position, c);
                    self.input_cursor_position += 1;
                }
            }
            KeyCode::Backspace => {
                if self.input_cursor_position > 0 {
                    self.input.remove(self.input_cursor_position - 1);
                    self.input_cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                if self.input_cursor_position < self.input.len() {
                    self.input.remove(self.input_cursor_position);
                }
            }
            KeyCode::Left => {
                if self.input_cursor_position > 0 {
                    self.input_cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.input_cursor_position < self.input.len() {
                    self.input_cursor_position += 1;
                }
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.start_message_sending().await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn start_message_sending(&mut self) -> Result<()> {
        let user_message = self.input.clone();
        self.input.clear();
        self.input_cursor_position = 0;

        self.add_user_message(&user_message);
        self.messages.push_back(Message::User {
            content: user_message.clone(),
            timestamp: chrono::Utc::now(),
        });

        if self.messages.len() > 50 {
            self.messages.pop_front();
        }

        self.is_loading = true;
        self.error_message = None;
        self.streaming_message = Some(String::new());

        let selected_model = &self.models[self.selected_model_index];
        let messages = self.build_messages("");
        match self.ollama_client.chat_stream(selected_model.name.clone(), messages).await {
            Ok(stream) => {
                self.stream = Some(stream);
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
                self.is_loading = false;
                self.streaming_message = None;
            }
        }
        Ok(())
    }

    pub async fn start_message_sending_with_content(&mut self, content: String) -> Result<()> {
        let user_message = content;
        self.input.clear();
        self.input_cursor_position = 0;

        self.add_user_message(&user_message);
        if self.messages.len() > 50 {
            self.messages.pop_front();
        }

        self.is_loading = true;
        self.error_message = None;
        self.streaming_message = Some(String::new());

        let selected_model = &self.models[self.selected_model_index];
        let messages = self.build_messages("");
        match self.ollama_client.chat_stream(selected_model.name.clone(), messages).await {
            Ok(stream) => {
                self.stream = Some(stream);
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
                self.is_loading = false;
                self.streaming_message = None;
            }
        }
        Ok(())
    }

    pub async fn process_streaming(&mut self) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            // Try to get the next chunk with a very short timeout
            match tokio::time::timeout(std::time::Duration::from_millis(10), stream.next()).await {
                Ok(Some(Ok(chunk))) => {
                    // Process each line of the response
                    for line in chunk.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        // Try to parse as JSON
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            // Only append if there is actual assistant message content
                            if let Some(content) = json.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    if let Some(ref mut streaming) = self.streaming_message {
                                        streaming.push_str(content);
                                    }
                                }
                            } else if let Some(content) = json.get("response").and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    if let Some(ref mut streaming) = self.streaming_message {
                                        streaming.push_str(content);
                                    }
                                }
                            } else if let Some(done) = json.get("done").and_then(|d| d.as_bool()) {
                                if done {
                                    self.finish_streaming();
                                    return Ok(());
                                }
                            }
                        }
                        // Ignore lines that are not valid JSON or do not contain a message
                    }
                }
                Ok(Some(Err(_e))) => {
                    self.error_message = Some("Stream error".to_string());
                    self.finish_streaming();
                }
                Ok(None) => {
                    // Stream finished
                    self.finish_streaming();
                }
                Err(_) => {
                    // Timeout - this is expected, just continue
                }
            }
        }
        Ok(())
    }

    fn finish_streaming(&mut self) {
        // When done, push the full message to history
        if let Some(content) = self.streaming_message.take() {
            if !content.trim().is_empty() {
                self.add_assistant_message(&content);
                self.messages.push_back(Message::Assistant {
                    content: content.clone(),
                    timestamp: chrono::Utc::now(),
                });
                self.scroll_to_bottom();
                // Parse for tool calls in the assistant message
                self.parse_tool_calls(&content);
                // Add memory after each assistant response
                self.add_memory("USER", &content);
            }
        }
        self.is_loading = false;
        self.stream = None;
    }

    pub fn parse_tool_calls(&mut self, message: &str) {
        use regex::Regex;
        // Only allow one pending tool call at a time
        let re = Regex::new(r#"\[tool_call:\s*(read_file|read_directory)\((?:path\s*=\s*)?['"](.*?)['"]\)\]"#).unwrap();
        if let Some(cap) = re.captures(message) {
            let tool = &cap[1];
            let path = cap[2].trim();
            let tc = match tool {
                "read_file" => ToolCall::ReadFile { path: path.to_string() },
                "read_directory" => ToolCall::ReadDirectory { path: path.to_string() },
                _ => return,
            };
            self.messages.push_back(Message::PendingToolCall {
                tool_call: tc,
                original_message: format!("{}(\"{}\")", tool, path),
                timestamp: chrono::Utc::now(),
            });
            self.scroll_to_bottom();
        }
    }

    pub async fn execute_tool_call(&self, tool_call: ToolCall) -> Result<String> {
        use std::fs;
        use std::path::PathBuf;
        match tool_call {
            ToolCall::ReadFile { path } => {
                let mut pb = PathBuf::from(&self.working_directory);
                pb.push(&path);
                match fs::read_to_string(&pb) {
                    Ok(content) => Ok(format!("[TOOL RESULT: read_file]\nPath: {}\n---\n{}", pb.display(), content)),
                    Err(e) => Ok(format!("[TOOL ERROR: read_file]\nPath: {}\nError: {}", pb.display(), e)),
                }
            }
            ToolCall::ReadDirectory { path } => {
                let mut pb = PathBuf::from(&self.working_directory);
                pb.push(&path);
                match fs::read_dir(&pb) {
                    Ok(entries) => {
                        let mut list = Vec::new();
                        for entry in entries.flatten() {
                            let file_type = entry.file_type().ok();
                            let name = entry.file_name().to_string_lossy().to_string();
                            let kind = if let Some(ft) = file_type {
                                if ft.is_dir() { "[DIR]" } else { "[FILE]" }
                            } else { "[?]" };
                            list.push(format!("{} {}", kind, name));
                        }
                        Ok(format!("[TOOL RESULT: read_directory]\nPath: {}\n---\n{}", pb.display(), list.join("\n")))
                    }
                    Err(e) => Ok(format!("[TOOL ERROR: read_directory]\nPath: {}\nError: {}", pb.display(), e)),
                }
            }
        }
    }

    fn deny_tool_call(&mut self, pending: PendingToolCall) {
        let msg = format!(
            "[TOOL DENIED]\nTool call was denied by the user:\n[tool_call: {}]",
            pending.original_message
        );
        self.messages.push_back(Message::Assistant {
            content: msg,
            timestamp: chrono::Utc::now(),
        });
        self.scroll_to_bottom();
    }

    pub fn get_selected_model(&self) -> Option<&Model> {
        self.models.get(self.selected_model_index)
    }
}