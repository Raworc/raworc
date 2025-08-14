use super::api::{RaworcClient, Message, MessageRole, SessionState};
use super::claude::ClaudeClient;
use super::error::Result;
use super::guardrails::Guardrails;
use super::todo::TodoManager;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct MessageHandler {
    api_client: Arc<RaworcClient>,
    claude_client: Arc<ClaudeClient>,
    todo_manager: Arc<Mutex<TodoManager>>,
    guardrails: Arc<Guardrails>,
    processed_message_ids: Arc<Mutex<HashSet<String>>>,
    agent_id: Option<Uuid>,
}

impl MessageHandler {
    pub fn new(
        api_client: Arc<RaworcClient>,
        claude_client: Arc<ClaudeClient>,
        todo_manager: Arc<Mutex<TodoManager>>,
        guardrails: Arc<Guardrails>,
    ) -> Self {
        Self {
            api_client,
            claude_client,
            todo_manager,
            guardrails,
            processed_message_ids: Arc::new(Mutex::new(HashSet::new())),
            agent_id: None, // Can be set from environment or config
        }
    }
    
    pub async fn poll_and_process(&self) -> Result<usize> {
        // Get recent messages
        let messages = self.api_client.get_messages(Some(50), None).await?;
        
        if messages.is_empty() {
            return Ok(0);
        }
        
        // Find unprocessed user messages
        let mut processed_ids = self.processed_message_ids.lock().await;
        let mut new_messages = Vec::new();
        
        for message in messages.iter() {
            if !processed_ids.contains(&message.id) {
                if message.role == MessageRole::User {
                    new_messages.push(message.clone());
                }
                processed_ids.insert(message.id.clone());
            }
        }
        
        if new_messages.is_empty() {
            return Ok(0);
        }
        
        info!("Found {} new user messages to process", new_messages.len());
        
        // Update session state to BUSY
        if let Err(e) = self.api_client.update_session_state(SessionState::Busy).await {
            warn!("Failed to update session state to BUSY: {}", e);
        }
        
        // Process each new message
        for message in new_messages.iter() {
            if let Err(e) = self.process_message(message, &messages).await {
                error!("Failed to process message {}: {}", message.id, e);
            }
        }
        
        // Update session state back to READY
        if let Err(e) = self.api_client.update_session_state(SessionState::Ready).await {
            warn!("Failed to update session state to READY: {}", e);
        }
        
        Ok(new_messages.len())
    }
    
    async fn process_message(&self, message: &Message, all_messages: &[Message]) -> Result<()> {
        info!("Processing message: {}", message.id);
        
        // Validate input with guardrails
        self.guardrails.validate_input(&message.content)?;
        
        // Check for todo commands first
        if let Some(response) = self.handle_todo_command(&message.content).await? {
            // Send todo response
            self.api_client.send_message(
                response,
                self.agent_id,
                Some(serde_json::json!({
                    "type": "todo_response"
                })),
            ).await?;
            return Ok(());
        }
        
        // Prepare conversation history for Claude
        let conversation = self.prepare_conversation_history(all_messages, &message.id);
        
        // Get Claude's response
        let system_prompt = self.build_system_prompt();
        let claude_response = self.claude_client
            .complete(conversation, Some(system_prompt))
            .await?;
        
        // Validate and sanitize output
        let sanitized_response = self.guardrails.validate_output(&claude_response)?;
        
        // Send response back via API
        self.api_client.send_message(
            sanitized_response,
            self.agent_id,
            Some(serde_json::json!({
                "type": "claude_response",
                "model": "claude-3-5-sonnet-20241022"
            })),
        ).await?;
        
        Ok(())
    }
    
    async fn handle_todo_command(&self, content: &str) -> Result<Option<String>> {
        let lower = content.to_lowercase();
        
        // Add todo
        if lower.starts_with("/todo add ") || lower.starts_with("/add ") {
            let description = content.split_once(' ')
                .and_then(|(_cmd, rest)| rest.split_once(' '))
                .map(|(_subcmd, desc)| desc)
                .unwrap_or("")
                .trim();
            
            if !description.is_empty() {
                let mut manager = self.todo_manager.lock().await;
                let todo = manager.add(description.to_string(), None).await?;
                return Ok(Some(format!("✓ Added todo #{}: {}", todo.id, todo.description)));
            }
        }
        
        // List todos
        if lower.starts_with("/todo list") || lower.starts_with("/todos") {
            let manager = self.todo_manager.lock().await;
            let todos = manager.list(false).await;
            
            if todos.is_empty() {
                return Ok(Some("No pending todos.".to_string()));
            }
            
            let mut response = String::from("📋 Pending todos:\n");
            for todo in todos {
                let priority = todo.priority.map(|p| format!("({}) ", p)).unwrap_or_default();
                response.push_str(&format!("  #{} {}{}\n", todo.id, priority, todo.description));
            }
            return Ok(Some(response));
        }
        
        // Complete todo
        if lower.starts_with("/todo done ") || lower.starts_with("/done ") {
            if let Some(id_str) = content.split_whitespace().last() {
                if let Ok(id) = id_str.parse::<usize>() {
                    let mut manager = self.todo_manager.lock().await;
                    manager.complete(id).await?;
                    return Ok(Some(format!("✓ Completed todo #{}", id)));
                }
            }
        }
        
        // Update todo
        if lower.starts_with("/todo update ") {
            let parts: Vec<&str> = content.splitn(4, ' ').collect();
            if parts.len() == 4 {
                if let Ok(id) = parts[2].parse::<usize>() {
                    let mut manager = self.todo_manager.lock().await;
                    manager.update(id, parts[3].to_string()).await?;
                    return Ok(Some(format!("✓ Updated todo #{}", id)));
                }
            }
        }
        
        Ok(None)
    }
    
    fn prepare_conversation_history(&self, messages: &[Message], current_id: &str) -> Vec<(String, String)> {
        let mut conversation = Vec::new();
        
        // Add recent message history (last 10 messages before current)
        let mut history: Vec<_> = messages
            .iter()
            .filter(|m| m.id != current_id)
            .filter(|m| m.role == MessageRole::User || m.role == MessageRole::Agent)
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => "user",
                    MessageRole::Agent => "assistant",
                    _ => "user",
                };
                (role.to_string(), m.content.clone())
            })
            .collect();
        
        // Keep only last 10 messages for context
        if history.len() > 10 {
            history = history.split_off(history.len() - 10);
        }
        
        conversation.extend(history);
        
        // Add current message
        if let Some(current) = messages.iter().find(|m| m.id == current_id) {
            conversation.push(("user".to_string(), current.content.clone()));
        }
        
        conversation
    }
    
    fn build_system_prompt(&self) -> String {
        format!(
            r#"You are a helpful AI assistant operating within a Raworc session.

Key capabilities:
- You can help users with various tasks and answer questions
- You have access to a todo list system (users can use /todo commands)
- You maintain conversation context within this session

Guidelines:
- Be helpful, accurate, and concise
- Respect user privacy and security
- Do not execute or suggest harmful commands
- If asked to perform actions outside your capabilities, explain your limitations

Available todo commands (for users):
- /todo add <description> - Add a new todo
- /todo list - List pending todos
- /todo done <id> - Mark todo as completed
- /todo update <id> <description> - Update todo description

Current session context:
- This is an isolated session environment
- Messages are persisted in the Raworc system
- You're operating as an agent within this session"#
        )
    }
}