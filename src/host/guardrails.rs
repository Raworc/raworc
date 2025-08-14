use super::error::{HostError, Result};
use tracing::{debug, warn};

pub struct Guardrails {
    max_message_length: usize,
    max_messages_per_minute: u32,
}

impl Guardrails {
    pub fn new() -> Self {
        Self {
            max_message_length: 100_000,
            max_messages_per_minute: 30,
        }
    }
    
    /// Check if content contains sensitive information
    pub fn check_sensitive_content(&self, content: &str) -> Result<()> {
        // Simplified checks without regex for now
        let lower = content.to_lowercase();
        
        let sensitive_keywords = vec![
            "api_key", "apikey", "api-key",
            "secret", "token", "password", "passwd",
            "private_key", "private key",
        ];
        
        for keyword in sensitive_keywords {
            if lower.contains(keyword) {
                // Check if it looks like an actual secret (long string after keyword)
                if let Some(idx) = lower.find(keyword) {
                    let after = &content[idx + keyword.len()..];
                    let has_value = after.chars()
                        .skip_while(|c| c.is_whitespace() || *c == ':' || *c == '=')
                        .take(20)
                        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                        .count() > 15;
                    
                    if has_value {
                        warn!("Sensitive content detected in message");
                        return Err(HostError::Guardrail(
                            "Message contains potentially sensitive information".to_string()
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if content is within size limits
    pub fn check_message_size(&self, content: &str) -> Result<()> {
        if content.len() > self.max_message_length {
            return Err(HostError::Guardrail(format!(
                "Message exceeds maximum length of {} characters",
                self.max_message_length
            )));
        }
        Ok(())
    }
    
    /// Sanitize content before sending
    pub fn sanitize_output(&self, content: &str) -> String {
        let mut sanitized = content.to_string();
        
        // Simple redaction without regex
        let sensitive_keywords = vec![
            "api_key", "apikey", "api-key",
            "secret", "token", "password", "passwd",
        ];
        
        for keyword in sensitive_keywords {
            if sanitized.to_lowercase().contains(keyword) {
                // Find and replace the pattern
                let lower = sanitized.to_lowercase();
                if let Some(idx) = lower.find(keyword) {
                    let end_idx = idx + keyword.len();
                    // Find the value part (after : or =)
                    let mut value_start = end_idx;
                    let chars: Vec<char> = sanitized[end_idx..].chars().collect();
                    for (i, c) in chars.iter().enumerate() {
                        if !c.is_whitespace() && *c != ':' && *c != '=' {
                            value_start = end_idx + i;
                            break;
                        }
                    }
                    
                    // Find end of value
                    let mut value_end = value_start;
                    let value_chars: Vec<char> = sanitized[value_start..].chars().collect();
                    for (i, c) in value_chars.iter().enumerate() {
                        if c.is_whitespace() || *c == ',' || *c == ';' || *c == '}' {
                            value_end = value_start + i;
                            break;
                        }
                        if i > 50 {
                            value_end = value_start + 50;
                            break;
                        }
                    }
                    
                    if value_end > value_start {
                        let before = &sanitized[..value_start];
                        let after = &sanitized[value_end..];
                        sanitized = format!("{}[REDACTED]{}", before, after);
                    }
                }
            }
        }
        
        // Trim excessive whitespace
        sanitized = sanitized
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Ensure reasonable length
        if sanitized.len() > self.max_message_length {
            sanitized.truncate(self.max_message_length);
            sanitized.push_str("\n[Message truncated due to length]");
        }
        
        sanitized
    }
    
    /// Check if the input is asking for harmful actions
    pub fn check_harmful_intent(&self, content: &str) -> Result<()> {
        let harmful_patterns = [
            "rm -rf /",
            "format c:",
            "delete system32",
            ":(){:|:&};:",  // Fork bomb
        ];
        
        let lower_content = content.to_lowercase();
        
        for pattern in harmful_patterns {
            if lower_content.contains(pattern) {
                warn!("Potentially harmful command detected: {}", pattern);
                return Err(HostError::Guardrail(
                    "Request contains potentially harmful commands".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate all guardrails for input
    pub fn validate_input(&self, content: &str) -> Result<()> {
        debug!("Validating input with guardrails");
        
        self.check_message_size(content)?;
        self.check_harmful_intent(content)?;
        
        Ok(())
    }
    
    /// Validate all guardrails for output
    pub fn validate_output(&self, content: &str) -> Result<String> {
        debug!("Validating output with guardrails");
        
        self.check_message_size(content)?;
        self.check_sensitive_content(content)?;
        
        let sanitized = self.sanitize_output(content);
        Ok(sanitized)
    }
}