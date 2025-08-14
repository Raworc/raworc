use super::error::{HostError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

pub struct ClaudeClient {
    client: Client,
    api_key: String,
}

impl ClaudeClient {
    pub fn new(api_key: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| HostError::Claude(format!("Failed to create client: {}", e)))?;
        
        Ok(Self {
            client,
            api_key: api_key.to_string(),
        })
    }
    
    pub async fn complete(
        &self,
        messages: Vec<(String, String)>, // (role, content)
        system_prompt: Option<String>,
    ) -> Result<String> {
        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter()
            .map(|(role, content)| ClaudeMessage {
                role: match role.as_str() {
                    "user" | "USER" => "user".to_string(),
                    "assistant" | "AGENT" => "assistant".to_string(),
                    _ => "user".to_string(),
                },
                content,
            })
            .collect();
        
        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            messages: claude_messages,
            system: system_prompt,
        };
        
        debug!("Sending request to Claude API");
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| HostError::Claude(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HostError::Claude(format!("API error ({}): {}", status, error_text)));
        }
        
        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| HostError::Claude(format!("Failed to parse response: {}", e)))?;
        
        let content = claude_response
            .content
            .first()
            .ok_or_else(|| HostError::Claude("Empty response from Claude".to_string()))?
            .text
            .clone();
        
        info!("Received response from Claude (length: {})", content.len());
        
        Ok(content)
    }
}