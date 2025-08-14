use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub session_id: String,
    pub api_url: String,
    pub api_token: String,
    pub claude_api_key: String,
    pub polling_interval: Duration,
}