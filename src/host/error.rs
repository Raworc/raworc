use thiserror::Error;

#[derive(Error, Debug)]
pub enum HostError {
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Claude API error: {0}")]
    Claude(String),
    
    #[error("Todo management error: {0}")]
    Todo(String),
    
    #[error("Guardrail violation: {0}")]
    Guardrail(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, HostError>;