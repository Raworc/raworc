use anyhow::{Result, Context};
use bollard::container::Config;
use bollard::models::{ContainerStateStatusEnum, HostConfig, Mount, MountTypeEnum};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use super::{DockerClient, DockerSessionConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub session_id: Uuid,
    pub status: ContainerStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Creating,
    Running,
    Stopped,
    Failed,
    Removed,
}

impl From<ContainerStateStatusEnum> for ContainerStatus {
    fn from(status: ContainerStateStatusEnum) -> Self {
        match status {
            ContainerStateStatusEnum::CREATED => ContainerStatus::Creating,
            ContainerStateStatusEnum::RUNNING => ContainerStatus::Running,
            ContainerStateStatusEnum::PAUSED | 
            ContainerStateStatusEnum::EXITED => ContainerStatus::Stopped,
            ContainerStateStatusEnum::DEAD => ContainerStatus::Failed,
            ContainerStateStatusEnum::REMOVING |
            ContainerStateStatusEnum::EMPTY => ContainerStatus::Removed,
            _ => ContainerStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
}

pub struct ContainerManager {
    client: Arc<DockerClient>,
    config: DockerSessionConfig,
}

impl ContainerManager {
    pub fn new(client: DockerClient, config: DockerSessionConfig) -> Self {
        Self { 
            client: Arc::new(client), 
            config 
        }
    }
    
    pub fn from_arc(client: Arc<DockerClient>, config: DockerSessionConfig) -> Self {
        Self { client, config }
    }
    
    pub async fn create_session_container(
        &self,
        session_id: Uuid,
        session_name: &str,
        starting_prompt: &str,
    ) -> Result<String> {
        let container_name = format!("raworc-session-{}", session_id);
        let volume_path = format!("{}/{}", self.config.volumes_path, session_id);
        
        // Ensure volume directory exists
        tokio::fs::create_dir_all(&volume_path)
            .await
            .context("Failed to create volume directory")?;
        
        // Create container configuration
        let env = vec![
            format!("SESSION_ID={}", session_id),
            format!("SESSION_NAME={}", session_name),
            format!("STARTING_PROMPT={}", starting_prompt),
        ];
        
        let mut labels = HashMap::new();
        labels.insert("raworc.session.id".to_string(), session_id.to_string());
        labels.insert("raworc.session.name".to_string(), session_name.to_string());
        labels.insert("raworc.managed".to_string(), "true".to_string());
        
        // Configure resource limits
        let host_config = HostConfig {
            memory: Some(self.config.memory_limit),
            memory_swap: Some(self.config.memory_limit), // Prevent swap usage
            cpu_quota: Some((self.config.cpu_limit * 100000.0) as i64), // Convert to microseconds
            cpu_period: Some(100000), // 100ms period
            mounts: Some(vec![
                Mount {
                    target: Some("/workspace".to_string()),
                    source: Some(volume_path),
                    typ: Some(MountTypeEnum::BIND),
                    read_only: Some(false),
                    ..Default::default()
                },
            ]),
            network_mode: self.config.network.clone(),
            auto_remove: Some(false), // Keep containers for debugging
            ..Default::default()
        };
        
        let config = Config {
            image: Some(self.config.image.clone()),
            hostname: Some(format!("session-{}", session_id)),
            env: Some(env),
            labels: Some(labels),
            host_config: Some(host_config),
            working_dir: Some("/workspace".to_string()),
            cmd: Some(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "echo 'Session container started'; sleep infinity".to_string(),
            ]),
            ..Default::default()
        };
        
        // Pull image if needed
        if !self.image_exists(&self.config.image).await? {
            self.client.pull_image(&self.config.image).await?;
        }
        
        // Create and start container
        let container_id = self.client.create_container(&container_name, config).await?;
        self.client.start_container(&container_id).await?;
        
        info!("Created container {} for session {}", container_id, session_id);
        Ok(container_id)
    }
    
    pub async fn stop_session_container(&self, container_id: &str) -> Result<()> {
        if self.client.container_exists(container_id).await {
            self.client.stop_container(container_id, Some(10)).await?;
            info!("Stopped container {}", container_id);
        }
        Ok(())
    }
    
    pub async fn remove_session_container(&self, container_id: &str) -> Result<()> {
        if self.client.container_exists(container_id).await {
            // Stop first if running
            if self.client.is_container_running(container_id).await? {
                self.stop_session_container(container_id).await?;
            }
            
            self.client.remove_container(container_id, true).await?;
            info!("Removed container {}", container_id);
        }
        Ok(())
    }
    
    pub async fn restart_session_container(&self, container_id: &str) -> Result<()> {
        if !self.client.container_exists(container_id).await {
            return Err(anyhow::anyhow!("Container {} does not exist", container_id));
        }
        
        // Stop if running
        if self.client.is_container_running(container_id).await? {
            self.client.stop_container(container_id, Some(5)).await?;
        }
        
        // Start again
        self.client.start_container(container_id).await?;
        info!("Restarted container {}", container_id);
        Ok(())
    }
    
    pub async fn get_container_status(&self, container_id: &str) -> Result<ContainerStatus> {
        let info = self.client.inspect_container(container_id).await?;
        
        if let Some(state) = info.state {
            if let Some(status) = state.status {
                return Ok(ContainerStatus::from(status));
            }
        }
        
        Ok(ContainerStatus::Failed)
    }
    
    pub async fn exec_in_container(
        &self,
        container_id: &str,
        command: Vec<String>,
    ) -> Result<String> {
        // Ensure container is running
        if !self.client.is_container_running(container_id).await? {
            return Err(anyhow::anyhow!("Container {} is not running", container_id));
        }
        
        self.client.exec_command(container_id, command).await
    }
    
    pub async fn get_container_logs(
        &self,
        container_id: &str,
        tail: Option<usize>,
    ) -> Result<String> {
        self.client.get_container_logs(container_id, tail).await
    }
    
    async fn image_exists(&self, image: &str) -> Result<bool> {
        // Try to inspect the image
        match self.client.docker.inspect_image(image).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    pub async fn list_session_containers(&self) -> Result<Vec<(Uuid, String, ContainerStatus)>> {
        let containers = self.client.list_containers(true).await?;
        let mut sessions = Vec::new();
        
        for container in containers {
            if let Some(labels) = container.labels {
                if labels.get("raworc.managed") == Some(&"true".to_string()) {
                    if let Some(session_id_str) = labels.get("raworc.session.id") {
                        if let Ok(session_id) = Uuid::parse_str(session_id_str) {
                            let container_id = container.id.unwrap_or_default();
                            let status = match container.state.as_deref() {
                                Some("running") => ContainerStatus::Running,
                                Some("exited") => ContainerStatus::Stopped,
                                Some("created") => ContainerStatus::Creating,
                                _ => ContainerStatus::Failed,
                            };
                            sessions.push((session_id, container_id, status));
                        }
                    }
                }
            }
        }
        
        Ok(sessions)
    }
}