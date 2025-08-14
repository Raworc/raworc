mod docker_manager;
mod session_manager;

pub use session_manager::SessionManager;

use anyhow::Result;

pub async fn run() -> Result<()> {
    tracing::info!("Starting Raworc Operator...");
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let manager = SessionManager::new(&database_url).await?;
    manager.run().await?;
    
    Ok(())
}