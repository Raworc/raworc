use super::config::Config;
use super::error::{HostError, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageRole {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateMessageRequest {
    pub role: MessageRole,
    pub content: String,
    pub agent_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    Pending,
    Ready,
    Busy,
    Idle,
    Terminated,
}

#[derive(Debug, Serialize)]
pub struct UpdateSessionStateRequest {
    pub state: SessionState,
}

pub struct RaworcClient {
    client: Client,
    config: Arc<Config>,
}

impl RaworcClient {
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    /// Get messages for the current session
    pub async fn get_messages(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Message>> {
        let mut url = format!(
            "{}/api/v1/sessions/{}/messages",
            self.config.api_url,
            self.config.session_id
        );
        
        let mut params = vec![];
        if let Some(limit) = limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = offset {
            params.push(format!("offset={}", offset));
        }
        
        if !params.is_empty() {
            url.push_str("?");
            url.push_str(&params.join("&"));
        }
        
        debug!("Fetching messages from: {}", url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .send()
            .await?;
        
        match response.status() {
            StatusCode::OK => {
                let messages = response.json::<Vec<Message>>().await?;
                debug!("Fetched {} messages", messages.len());
                Ok(messages)
            }
            StatusCode::UNAUTHORIZED => {
                Err(HostError::Api("Unauthorized - check API token".to_string()))
            }
            StatusCode::NOT_FOUND => {
                Err(HostError::Api(format!("Session {} not found", self.config.session_id)))
            }
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(HostError::Api(format!("API error ({}): {}", status, error_text)))
            }
        }
    }
    
    /// Send a message as the agent
    pub async fn send_message(
        &self,
        content: String,
        agent_id: Option<Uuid>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Message> {
        let url = format!(
            "{}/api/v1/sessions/{}/messages",
            self.config.api_url,
            self.config.session_id
        );
        
        let request = CreateMessageRequest {
            role: MessageRole::Agent,
            content,
            agent_id,
            metadata,
        };
        
        debug!("Sending message to: {}", url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .json(&request)
            .send()
            .await?;
        
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let message = response.json::<Message>().await?;
                info!("Message sent successfully: {}", message.id);
                Ok(message)
            }
            StatusCode::UNAUTHORIZED => {
                Err(HostError::Api("Unauthorized - check API token".to_string()))
            }
            StatusCode::NOT_FOUND => {
                Err(HostError::Api(format!("Session {} not found", self.config.session_id)))
            }
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(HostError::Api(format!("Failed to send message ({}): {}", status, error_text)))
            }
        }
    }
    
    /// Update session state
    pub async fn update_session_state(&self, state: SessionState) -> Result<()> {
        let url = format!(
            "{}/api/v1/sessions/{}/state",
            self.config.api_url,
            self.config.session_id
        );
        
        let request = UpdateSessionStateRequest { state: state.clone() };
        
        debug!("Updating session state to: {:?}", state);
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .json(&request)
            .send()
            .await?;
        
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => {
                info!("Session state updated to: {:?}", state);
                Ok(())
            }
            StatusCode::UNAUTHORIZED => {
                Err(HostError::Api("Unauthorized - check API token".to_string()))
            }
            StatusCode::NOT_FOUND => {
                Err(HostError::Api(format!("Session {} not found", self.config.session_id)))
            }
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(HostError::Api(format!("Failed to update state ({}): {}", status, error_text)))
            }
        }
    }
}