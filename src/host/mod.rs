// Host agent modules - placeholder for future implementation
// mod api;
// mod claude;
// mod config;
// mod error;
// mod guardrails;
// mod message_handler;
// mod todo;


use anyhow::Result;

pub async fn run(api_url: &str, session_id: &str, _api_key: &str) -> Result<()> {
    tracing::info!("Starting Raworc Host Agent...");
    tracing::info!("Connecting to API: {}", api_url);
    tracing::info!("Session ID: {}", session_id);
    
    // For now, just loop and wait
    // TODO: Implement actual host agent logic
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        tracing::debug!("Host agent running...");
    }
}