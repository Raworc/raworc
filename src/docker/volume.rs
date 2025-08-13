use anyhow::{Result, Context};
use std::path::PathBuf;
use tracing::{info, warn};
use uuid::Uuid;

pub struct VolumeManager {
    base_path: PathBuf,
}

impl VolumeManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
    
    pub async fn create_session_volume(&self, session_id: Uuid) -> Result<PathBuf> {
        let volume_path = self.base_path.join(session_id.to_string());
        
        tokio::fs::create_dir_all(&volume_path)
            .await
            .context("Failed to create volume directory")?;
        
        info!("Created volume for session {} at {:?}", session_id, volume_path);
        Ok(volume_path)
    }
    
    pub async fn remove_session_volume(&self, session_id: Uuid) -> Result<()> {
        let volume_path = self.base_path.join(session_id.to_string());
        
        if volume_path.exists() {
            tokio::fs::remove_dir_all(&volume_path)
                .await
                .context("Failed to remove volume directory")?;
            
            info!("Removed volume for session {} at {:?}", session_id, volume_path);
        } else {
            warn!("Volume for session {} not found at {:?}", session_id, volume_path);
        }
        
        Ok(())
    }
    
    pub async fn volume_exists(&self, session_id: Uuid) -> bool {
        let volume_path = self.base_path.join(session_id.to_string());
        volume_path.exists()
    }
    
    pub async fn get_volume_size(&self, session_id: Uuid) -> Result<u64> {
        let volume_path = self.base_path.join(session_id.to_string());
        
        if !volume_path.exists() {
            return Ok(0);
        }
        
        let mut size = 0u64;
        let mut entries = tokio::fs::read_dir(&volume_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                size += metadata.len();
            }
        }
        
        Ok(size)
    }
    
    pub fn get_volume_path(&self, session_id: Uuid) -> PathBuf {
        self.base_path.join(session_id.to_string())
    }
}