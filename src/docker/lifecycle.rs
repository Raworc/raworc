use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::models::AppState;
use crate::models::{Session, SessionState, UpdateSessionStateRequest};
use super::{DockerClient, DockerSessionConfig, ContainerEvent, ContainerStatus};
use super::container::ContainerManager;
use super::volume::VolumeManager;

#[derive(Clone)]
pub struct ContainerLifecycleManager {
    app_state: Arc<AppState>,
    container_manager: Arc<ContainerManager>,
    volume_manager: Arc<VolumeManager>,
    event_sender: mpsc::UnboundedSender<ContainerEvent>,
}

impl ContainerLifecycleManager {
    pub async fn new(
        app_state: Arc<AppState>,
        config: DockerSessionConfig,
    ) -> Result<Self> {
        let docker_client = DockerClient::new(Default::default()).await?;
        let container_manager = Arc::new(ContainerManager::new(docker_client, config.clone()));
        let volume_manager = Arc::new(VolumeManager::new(&config.volumes_path));
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Start event handler immediately
        let app_state_clone = app_state.clone();
        tokio::spawn(async move {
            Self::handle_events(rx, app_state_clone).await;
        });
        
        Ok(Self {
            app_state,
            container_manager,
            volume_manager,
            event_sender: tx,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        // Start health check loop
        let app_state = self.app_state.clone();
        let container_manager = self.container_manager.clone();
        tokio::spawn(async move {
            Self::health_check_loop(app_state, container_manager).await;
        });
        
        // Start idle timeout loop
        let app_state = self.app_state.clone();
        let container_manager = self.container_manager.clone();
        tokio::spawn(async move {
            Self::idle_timeout_loop(app_state, container_manager).await;
        });
        
        info!("Container lifecycle manager started");
        Ok(())
    }
    
    pub async fn create_session_container(
        &self,
        session: &Session,
    ) -> Result<String> {
        info!("Creating container for session {}", session.id);
        
        // Create volume
        self.volume_manager.create_session_volume(session.id).await?;
        
        // Create container
        let container_id = self.container_manager
            .create_session_container(
                session.id,
                &session.name,
                &session.starting_prompt,
            )
            .await?;
        
        // Update session state
        let update_req = UpdateSessionStateRequest {
            state: SessionState::Ready,
            container_id: Some(container_id.clone()),
            persistent_volume_id: Some(session.id.to_string()),
            termination_reason: None,
        };
        
        Session::update_state(&self.app_state.db, session.id, update_req).await?;
        
        // Send event
        let _ = self.event_sender.send(ContainerEvent::Created {
            session_id: session.id,
            container_id: container_id.clone(),
        });
        
        Ok(container_id)
    }
    
    pub async fn stop_session_container(&self, session_id: Uuid) -> Result<()> {
        let session = Session::find_by_id(&self.app_state.db, session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
        if let Some(container_id) = &session.container_id {
            info!("Stopping container for session {}", session_id);
            
            self.container_manager.stop_session_container(container_id).await?;
            
            // Update session state to IDLE
            let update_req = UpdateSessionStateRequest {
                state: SessionState::Idle,
                container_id: Some(container_id.clone()),
                persistent_volume_id: session.persistent_volume_id.clone(),
                termination_reason: None,
            };
            
            Session::update_state(&self.app_state.db, session_id, update_req).await?;
            
            // Send event
            let _ = self.event_sender.send(ContainerEvent::Stopped {
                session_id,
                container_id: container_id.clone(),
            });
        }
        
        Ok(())
    }
    
    pub async fn remove_session_container(&self, session_id: Uuid) -> Result<()> {
        let session = Session::find_by_id(&self.app_state.db, session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
        if let Some(container_id) = &session.container_id {
            info!("Removing container for session {}", session_id);
            
            // Remove container
            self.container_manager.remove_session_container(container_id).await?;
            
            // Remove volume
            self.volume_manager.remove_session_volume(session_id).await?;
            
            // Update session state
            let update_req = UpdateSessionStateRequest {
                state: SessionState::Error,
                container_id: None,
                persistent_volume_id: None,
                termination_reason: Some("Container removed".to_string()),
            };
            
            Session::update_state(&self.app_state.db, session_id, update_req).await?;
            
            // Send event
            let _ = self.event_sender.send(ContainerEvent::Removed {
                session_id,
                container_id: container_id.clone(),
            });
        }
        
        Ok(())
    }
    
    pub async fn reactivate_session(&self, session_id: Uuid) -> Result<()> {
        let session = Session::find_by_id(&self.app_state.db, session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
        if session.state != SessionState::Idle {
            return Err(anyhow::anyhow!("Session is not idle"));
        }
        
        if let Some(container_id) = &session.container_id {
            info!("Reactivating session {}", session_id);
            
            // Restart container
            self.container_manager.restart_session_container(container_id).await?;
            
            // Update session state
            let update_req = UpdateSessionStateRequest {
                state: SessionState::Ready,
                container_id: Some(container_id.clone()),
                persistent_volume_id: session.persistent_volume_id.clone(),
                termination_reason: None,
            };
            
            Session::update_state(&self.app_state.db, session_id, update_req).await?;
            
            // Send event
            let _ = self.event_sender.send(ContainerEvent::Started {
                session_id,
                container_id: container_id.clone(),
            });
        } else {
            // No container exists, create a new one
            self.create_session_container(&session).await?;
        }
        
        Ok(())
    }
    
    async fn handle_events(
        mut receiver: mpsc::UnboundedReceiver<ContainerEvent>,
        app_state: Arc<AppState>,
    ) {
        while let Some(event) = receiver.recv().await {
            match event {
                ContainerEvent::Created { session_id, container_id } => {
                    info!("Container created for session {}: {}", session_id, container_id);
                }
                ContainerEvent::Started { session_id, container_id } => {
                    info!("Container started for session {}: {}", session_id, container_id);
                }
                ContainerEvent::Stopped { session_id, container_id } => {
                    info!("Container stopped for session {}: {}", session_id, container_id);
                }
                ContainerEvent::Removed { session_id, container_id } => {
                    info!("Container removed for session {}: {}", session_id, container_id);
                }
                ContainerEvent::Failed { session_id, container_id, reason } => {
                    error!("Container failed for session {}: {} - {}", session_id, container_id, reason);
                    
                    // Update session state to ERROR
                    let update_req = UpdateSessionStateRequest {
                        state: SessionState::Error,
                        container_id: Some(container_id),
                        persistent_volume_id: None,
                        termination_reason: Some(reason),
                    };
                    
                    let _ = Session::update_state(&app_state.db, session_id, update_req).await;
                }
            }
        }
    }
    
    async fn health_check_loop(
        app_state: Arc<AppState>,
        container_manager: Arc<ContainerManager>,
    ) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Get all active sessions
            match Session::find_all(&app_state.db, None, None).await {
                Ok(sessions) => {
                    for session in sessions {
                        if session.state == SessionState::Ready || session.state == SessionState::Busy {
                            if let Some(container_id) = &session.container_id {
                                // Check container status
                                match container_manager.get_container_status(container_id).await {
                                    Ok(status) => {
                                        if status != ContainerStatus::Running {
                                            warn!("Container {} for session {} is not running", 
                                                container_id, session.id);
                                            
                                            // Update session state
                                            let update_req = UpdateSessionStateRequest {
                                                state: SessionState::Error,
                                                container_id: Some(container_id.clone()),
                                                persistent_volume_id: session.persistent_volume_id.clone(),
                                                termination_reason: Some(format!("Container status: {:?}", status)),
                                            };
                                            
                                            let _ = Session::update_state(&app_state.db, session.id, update_req).await;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to check container status: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch sessions for health check: {}", e);
                }
            }
        }
    }
    
    async fn idle_timeout_loop(
        app_state: Arc<AppState>,
        container_manager: Arc<ContainerManager>,
    ) {
        let mut interval = interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            // Find sessions that should timeout
            match Session::find_waiting_sessions_to_timeout(&app_state.db).await {
                Ok(sessions) => {
                    for session in sessions {
                        if let Some(container_id) = &session.container_id {
                            info!("Session {} idle timeout reached, stopping container", session.id);
                            
                            // Stop container
                            if let Err(e) = container_manager.stop_session_container(container_id).await {
                                error!("Failed to stop idle container: {}", e);
                                continue;
                            }
                            
                            // Update session state to IDLE
                            let update_req = UpdateSessionStateRequest {
                                state: SessionState::Idle,
                                container_id: Some(container_id.clone()),
                                persistent_volume_id: session.persistent_volume_id.clone(),
                                termination_reason: None,
                            };
                            
                            let _ = Session::update_state(&app_state.db, session.id, update_req).await;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch sessions for idle timeout: {}", e);
                }
            }
        }
    }
}